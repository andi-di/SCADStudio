//! M1 document workbench. Disk and engine work run on owned background jobs.
use crate::{
    document::{ChangeSet, DocumentId, Revision, TextEdit},
    files::{Conflict, FileSession, Resolution},
    render::Mesh,
};
use anyhow::Result;
use eframe::egui;
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

#[path = "workbench_probe.rs"]
mod probe;

type DiskResult = (FileSession, Vec<Conflict>, String);
pub struct Workbench {
    session: Option<FileSession>,
    disk: Option<mpsc::Receiver<DiskResult>>,
    conflicts: Vec<Conflict>,
    active: DocumentId,
    status: String,
    checked: Instant,
    offset: usize,
    length: usize,
    value: f64,
    preview: Mesh,
    preview_revision: Option<Revision>,
    evaluation: Option<(Revision, mpsc::Receiver<Result<Mesh>>)>,
    yaw: f32,
    probe: Option<probe::Probe>,
    targets: [Option<egui::Pos2>; 7],
    disk_base: Revision,
    literal_target: Option<crate::document::SourceRange>,
    syntax: Option<(DocumentId, Revision, crate::syntax::Syntax)>,
}
impl Workbench {
    fn new(session: FileSession) -> Self {
        Self {
            session: Some(session),
            disk: None,
            conflicts: vec![],
            active: DocumentId(1),
            status: String::new(),
            checked: Instant::now(),
            offset: 0,
            length: 1,
            value: 12.,
            preview: Mesh::default(),
            preview_revision: None,
            evaluation: None,
            yaw: -0.6,
            probe: None,
            targets: [None; 7],
            syntax: None,
            disk_base: Revision(0),
            literal_target: None,
        }
    }
    fn disk_job(
        &mut self,
        action: impl FnOnce(&mut FileSession) -> Result<()> + Send + 'static,
        context: egui::Context,
    ) {
        if self.disk.is_some() {
            self.status = "Dateiauftrag läuft bereits; bitte erneut auslösen".into();
            return;
        }
        let Some(mut session) = self.session.clone() else {
            return;
        };
        self.disk_base = session.documents.revision();
        let (tx, rx) = mpsc::sync_channel(1);
        self.disk = Some(rx);
        std::thread::spawn(move || {
            let status = action(&mut session)
                .err()
                .map_or(String::new(), |e| format!("{e:#}"));
            let (conflicts, status) = match session.scan() {
                Ok(c) => (c, status),
                Err(e) => (vec![], format!("{status} · Dateiprüfung: {e:#}")),
            };
            let _ = tx.send((session, conflicts, status));
            context.request_repaint();
        });
    }
    fn evaluate(&mut self, context: egui::Context) {
        if self.evaluation.is_some() {
            return;
        }
        let Some(session) = &self.session else { return };
        let Some(entry) = session.files.get(&DocumentId(1)) else {
            return;
        };
        let Some(root) = entry.path.parent().map(std::path::Path::to_path_buf) else {
            return;
        };
        let entrypoint = entry
            .path
            .strip_prefix(&root)
            .unwrap_or(&entry.path)
            .to_path_buf();
        let overlays: Vec<_> = session
            .files
            .iter()
            .filter_map(|(id, file)| {
                Some((
                    file.path.strip_prefix(&root).ok()?.to_path_buf(),
                    session.documents.text(*id)?.as_bytes().to_vec(),
                ))
            })
            .collect();
        if overlays.len() != session.files.len() {
            self.status =
                "Vorschau: Dateien müssen unter dem Einstiegspunkt-Verzeichnis liegen".into();
            return;
        }
        let revision = session.documents.revision();
        let (tx, rx) = mpsc::sync_channel(1);
        self.evaluation = Some((revision, rx));
        self.status = "Berechnung läuft".into();
        std::thread::spawn(move || {
            let result = (|| -> Result<Mesh> {
                let mut files = std::collections::BTreeMap::new();
                collect(&root, &root, &mut files, &mut 0)?;
                files.extend(overlays);
                let snapshot = crate::persistence::EvaluationSnapshot {
                    revision,
                    entrypoint,
                    files,
                };
                let engine = crate::engine::Engine {
                    executable: PathBuf::from("target/m0-engine/worker/openscad").canonicalize()?,
                    library_root: PathBuf::from("target/m0-engine/libraries").canonicalize()?,
                    timeout: Duration::from_secs(60),
                };
                let result = engine.evaluate(snapshot, Default::default())?;
                Mesh::from_stl(&crate::engine::read_bounded(
                    &result.artifacts.join("result.stl"),
                    64 * 1024 * 1024,
                )?)
            })();
            let _ = tx.send(result);
            context.request_repaint();
        });
    }
}
fn collect(
    root: &std::path::Path,
    directory: &std::path::Path,
    files: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>,
    size: &mut usize,
) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        if matches!(entry.file_name().to_str(), Some(".git" | "target")) {
            continue;
        }
        let kind = entry.file_type()?;
        anyhow::ensure!(
            !kind.is_symlink(),
            "Snapshot enthält Symlink: {}",
            entry.path().display()
        );
        if kind.is_dir() {
            collect(root, &entry.path(), files, size)?;
        } else if kind.is_file() {
            let bytes = crate::engine::read_bounded(&entry.path(), 64 * 1024 * 1024)?;
            *size += bytes.len();
            anyhow::ensure!(
                *size <= 128 * 1024 * 1024 && files.len() < 4096,
                "Snapshot-Limit überschritten"
            );
            files.insert(entry.path().strip_prefix(root)?.to_path_buf(), bytes);
        }
    }
    Ok(())
}
impl eframe::App for Workbench {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        if let Some(probe) = &mut self.probe {
            input.events.append(&mut probe.events);
        }
    }
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.disk {
            match rx.try_recv() {
                Ok((session, conflicts, status)) => {
                    if let Some(current) = &mut self.session {
                        match current.merge_io(self.disk_base, session) {
                            Ok(()) => self.conflicts = conflicts,
                            Err(error) => {
                                self.status = error.to_string();
                                self.conflicts.clear();
                            }
                        }
                    }
                    if !status.is_empty() {
                        self.status = status;
                    }
                    self.disk = None;
                    self.checked = Instant::now();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = "Datei-Worker unerwartet beendet".into();
                    self.disk = None;
                }
                _ => {}
            }
        }
        if self.session.is_some()
            && let Some((revision, rx)) = &self.evaluation
        {
            match rx.try_recv() {
                Ok(result) => {
                    if self
                        .session
                        .as_ref()
                        .is_some_and(|s| s.documents.revision() == *revision)
                    {
                        match result {
                            Ok(mesh) => {
                                self.preview = mesh;
                                self.preview_revision = Some(*revision);
                                self.status = "Vorschau aktuell".into();
                            }
                            Err(e) => {
                                self.status = format!("Engine: Revision {} · {e:#}", revision.0)
                            }
                        }
                    }
                    self.evaluation = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = "Engine-Worker unerwartet beendet".into();
                    self.evaluation = None;
                }
                _ => {}
            }
        }
        let mut save = false;
        let mut preview = false;
        let mut resolution = None;
        egui::CentralPanel::default().show(root,|ui|{
            egui::ScrollArea::vertical().show(ui,|ui|{
            ui.heading("SCADStudio · Dokumente");ui.label(&self.status);
            ui.horizontal(|ui|{
                save=ui.button("Speichern · Ctrl+S").clicked();
                let response = ui.button("Vorschau · F5"); self.targets[0] = Some(response.rect.center()); preview=response.clicked();
                if let Some(session)=&mut self.session {
                    for (label,redo) in [("Undo · Ctrl+Z",false),("Redo · Ctrl+Shift+Z",true)] {
                        let shortcut=ui.input_mut(|i|i.consume_key(egui::Modifiers {ctrl:true,shift:redo,..Default::default()},egui::Key::Z));
                        let response=ui.button(label); self.targets[if redo {4} else {3}]=Some(response.rect.center());
                        if response.clicked()||shortcut {let rev=session.documents.revision();let result=if redo {session.documents.redo(rev)}else{session.documents.undo(rev)};if let Err(e)=result{self.status=e.to_string();}}
                    }
                }
            });
            save|=ui.input_mut(|i|i.consume_key(egui::Modifiers::CTRL,egui::Key::S));
            preview|=ui.input_mut(|i|i.consume_key(egui::Modifiers::NONE,egui::Key::F5));
            if let Some(session)=&mut self.session {
                ui.horizontal_wrapped(|ui|{for (&id,file) in &session.files {ui.selectable_value(&mut self.active,id,format!("{}{}",file.path.file_name().unwrap_or_default().to_string_lossy(),if session.dirty(id){" *"}else{""}));}});
                ui.label(format!("Revision {} · UTF-8-Bytebereiche",session.documents.revision().0));
                ui.columns(2,|columns|{
                    let original=session.documents.text(self.active).unwrap_or_default().to_string();let mut text=original.clone();
                    egui::ScrollArea::both().id_salt(("source",self.active.0)).show(&mut columns[0],|ui|{let response=ui.add(egui::TextEdit::multiline(&mut text).id(egui::Id::unique(("editor",self.active.0))).code_editor().desired_width(ui.available_width()).desired_rows(11)); self.targets[1]=Some(response.rect.center());});
                    if text!=original {let result=session.documents.apply(ChangeSet {base:session.documents.revision(),origin:"code".into(),description:"Edit source".into(),edits:vec![TextEdit {document:self.active,range:0..original.len(),replacement:text.clone()}]});if let Err(e)=result {self.status=e.to_string();}}
                    columns[1].label(if self.preview_revision==Some(session.documents.revision()) {"Aktuelle Vorschau"} else {"Veraltete / fehlende Vorschau · alte Geometrie bleibt erhalten; keine alten Auswahlziele"});
                    columns[1].scope(|ui| {ui.set_max_height(250.0);crate::render::draw(ui,&self.preview,None,&mut self.yaw);});
                    let revision=session.documents.revision();
                    if !self.syntax.as_ref().is_some_and(|(id,rev,_)| *id==self.active && *rev==revision) {self.syntax=Some((self.active,revision,crate::syntax::Syntax::parse(&text)));}
                    let syntax=&self.syntax.as_ref().map(|(_,_,s)|s);
                    if let Some(syntax)=syntax {
                        columns[0].collapsing("Syntaxstruktur / Literal wählen", |ui| {
                            if let Some(range)=syntax_tree(ui, &syntax.nodes, &text, syntax) {
                                self.offset=range.start;self.length=range.len();
                                self.literal_target=Some(crate::document::SourceRange {document:self.active,revision,bytes:range});
                            }
                        });
                    }
                    for issue in syntax.iter().flat_map(|s|&s.issues) {columns[0].label(format!("Syntaxstruktur {}..{}: {}",issue.range.start,issue.range.end,issue.message));}
                });
                ui.horizontal(|ui|{
                    ui.label("Explizites Zahlenliteral: Bytebeginn / Länge / Wert");if ui.add(egui::DragValue::new(&mut self.offset)).changed() | ui.add(egui::DragValue::new(&mut self.length)).changed() {self.literal_target=None;}ui.add(egui::DragValue::new(&mut self.value));
                    let response=ui.button("Literal ändern");self.targets[2]=Some(response.rect.center());
                    if response.clicked(){let result=self.offset.checked_add(self.length).ok_or_else(||anyhow::anyhow!("Bereichsüberlauf")).and_then(|end|crate::language::plan_numeric_target(&session.documents,
                        &self.literal_target.clone().unwrap_or(crate::document::SourceRange {document:self.active,revision:session.documents.revision(),bytes:self.offset..end}),self.value)).and_then(|p|session.documents.apply(p));if let Err(e)=result{self.status=e.to_string();}}
                });
                for conflict in &self.conflicts {
                    egui::CollapsingHeader::new(format!("Externer Konflikt · Datei {}",conflict.document.0)).default_open(true).show(ui,|ui|{
                        ui.columns(3,|cols|{for (col,(label,text)) in cols.iter_mut().zip([("Basis",&conflict.baseline),("Lokal",&conflict.local),("Datenträger",&conflict.disk)]){col.label(label);egui::ScrollArea::both().max_height(85.).id_salt((conflict.document.0,label)).show(col,|ui|{ui.monospace(text);});}});
                        ui.horizontal(|ui|{let response=ui.button("Datenträger übernehmen");self.targets[5]=Some(response.rect.center());if response.clicked(){resolution=Some((conflict.clone(),Resolution::TakeDisk));}
let response=ui.button("Lokal behalten / manuell zusammenführen");self.targets[6]=Some(response.rect.center());if self.probe.is_some(){response.scroll_to_me(Some(egui::Align::Center));}
if response.clicked(){resolution=Some((conflict.clone(),Resolution::KeepLocal));}});
                        ui.label("Zum manuellen Zusammenführen lokal behalten, im Editor zusammenführen und speichern.");
                    });
                }
            } else {ui.label("Dateizugriff läuft …");}
        });
        });
        if save {
            self.disk_job(
                |s| {
                    let failures: Vec<_> = s
                        .save()
                        .into_iter()
                        .filter_map(|(id, r)| r.err().map(|e| format!("Datei {}: {e:#}", id.0)))
                        .collect();
                    anyhow::ensure!(failures.is_empty(), "{}", failures.join("; "));
                    Ok(())
                },
                root.ctx().clone(),
            );
        }
        if let Some((conflict, choice)) = resolution {
            self.disk_job(move |s| s.resolve(&conflict, choice), root.ctx().clone());
        }
        if preview {
            self.evaluate(root.ctx().clone());
        }
        if self.checked.elapsed() > Duration::from_secs(1) && self.disk.is_none() {
            self.disk_job(|_| Ok(()), root.ctx().clone());
        }
        if let Some(mut probe) = self.probe.take() {
            probe.advance(self, root.ctx());
            self.probe = Some(probe);
        }
        root.ctx().request_repaint_after(Duration::from_secs(1));
    }
}
fn syntax_tree(
    ui: &mut egui::Ui,
    nodes: &[crate::syntax::Node],
    source: &str,
    syntax: &crate::syntax::Syntax,
) -> Option<std::ops::Range<usize>> {
    let mut selected = None;
    for node in nodes {
        if node.kind == crate::syntax::Kind::Trivia {
            continue;
        }
        ui.push_id((node.range.start, node.range.end), |ui| {
            let label = format!(
                "{:?} {}..{} · {}",
                node.kind,
                node.range.start,
                node.range.end,
                source[node.range.clone()]
                    .chars()
                    .take(45)
                    .collect::<String>()
            );
            if node.children.is_empty() {
                if node.kind == crate::syntax::Kind::Number {
                    if ui
                        .add_enabled(syntax.numeric_token(&node.range), egui::Button::new(label))
                        .clicked()
                    {
                        selected = Some(node.range.clone());
                    }
                } else {
                    ui.label(label);
                }
            } else {
                ui.collapsing(label, |ui| {
                    selected = syntax_tree(ui, &node.children, source, syntax).or(selected.take());
                });
            }
        });
    }
    selected
}

pub fn run(mut paths: Vec<PathBuf>) -> Result<()> {
    if paths.len() == 1 && paths[0].to_string_lossy().ends_with(".scadstudio.json") {
        let project = crate::files::ProjectMetadata::read(&paths[0])?;
        anyhow::ensure!(
            project.engine.reference_id == "9ba0cee7c94785fac0ca8dfe040b965a12a25ed1",
            "Enginebindung nicht verfügbar; .scad separat öffnen"
        );
        anyhow::ensure!(
            project.libraries.is_empty()
                && project.search_paths.is_empty()
                && project.external_resources.is_empty()
                && project.customizer.is_none(),
            "Dieses M1-Fenster unterstützt noch keine Projekt-Bibliotheks-/Variantenkonfiguration; .scad separat öffnen (vollständige Bindungen folgen in M3/M4)"
        );
        paths = vec![
            paths[0]
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join(project.entrypoint),
        ];
    }
    let session = FileSession::open(&paths)?;
    let mut app = Workbench::new(session);
    app.probe = probe::Probe::from_environment(&mut app)?;
    let report = app.probe.as_ref().map(|p| p.directory.join("report.json"));
    eframe::run_native(
        "SCADStudio",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            viewport: egui::ViewportBuilder::default().with_inner_size([1280., 800.]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("native UI: {e}"))?;
    if let Some(path) = report {
        anyhow::ensure!(path.is_file(), "Native M1 acceptance did not finish");
    }
    Ok(())
}
