//! Small technical selection viewer; not the M2 production modelling workspace.
use crate::{
    document::{DocumentId, Documents},
    engine::Manifest,
    language::plan_numeric_literal,
    render::{self, Mesh},
};
use anyhow::{Context, Result};
use eframe::egui;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Technical M0 viewer with temporary UI state and explicit source-literal editing.
pub struct ProofViewer {
    manifest: Manifest,
    meshes: BTreeMap<u64, Mesh>,
    result: Mesh,
    selected: Option<u64>,
    origin_view: bool,
    yaw: f32,
    documents: Documents,
    value: f64,
    status: String,
    screenshot_requested: bool,
    hits: Vec<u64>,
    probe: Option<NativeProbe>,
    targets: [Option<egui::Pos2>; 3],
    show_internal: bool,
}

impl ProofViewer {
    fn empty() -> Self {
        Self {
            manifest: Manifest {
                protocol_version: 1,
                snapshot_id: String::new(),
                revision: 0,
                nodes: Vec::new(),
            },
            meshes: BTreeMap::new(),
            result: Mesh::default(),
            selected: None,
            origin_view: false,
            yaw: -0.6,
            documents: Documents::default(),
            value: 12.0,
            status:
                "Kein Modell geladen · M0-Nachweise mit prove-m0 erzeugen; Anleitung in README.md"
                    .into(),
            screenshot_requested: false,
            hits: Vec::new(),
            probe: None,
            targets: [None; 3],
            show_internal: false,
        }
    }
    /// Called by the background loader, before creating the window.
    pub fn load(directory: &Path) -> Result<Self> {
        let manifest: Manifest = Manifest::decode(&crate::engine::read_bounded(
            &directory.join("manifest.json"),
            32 * 1024 * 1024,
        )?)?;
        let mut meshes = BTreeMap::new();
        for node in &manifest.nodes {
            if let Some(path) = &node.mesh {
                crate::persistence::validate_relative(Path::new(path))?;
                meshes.insert(
                    node.id,
                    Mesh::from_stl(&crate::engine::read_bounded(
                        &directory.join(path),
                        64 * 1024 * 1024,
                    )?)?,
                );
            }
        }
        let result = Mesh::from_stl(&std::fs::read(directory.join("result.stl"))?)?;
        let source =
            std::fs::read_to_string(directory.join("source.scad")).context("load proof source")?;
        Ok(Self {
            manifest,
            meshes,
            result,
            selected: None,
            origin_view: false,
            yaw: -0.6,
            documents: Documents::new(BTreeMap::from([(DocumentId(1), source)])),
            value: 12.0,
            status: "M0: evaluated origins; source edits invalidate this preview".into(),
            screenshot_requested: false,
            hits: Vec::new(),
            probe: NativeProbe::from_environment()?,
            targets: [None; 3],
            show_internal: false,
        })
    }
}

impl eframe::App for ProofViewer {
    fn raw_input_hook(&mut self, _context: &egui::Context, input: &mut egui::RawInput) {
        if let Some(probe) = &mut self.probe
            && let Some(pos) = probe.pending.take()
        {
            input.events.push(egui::Event::PointerMoved(pos));
            for pressed in [true, false] {
                input.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                });
            }
        }
    }

    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(root, |ui| {
            ui.heading("SCADStudio · M0 Integrationsnachweis");
            ui.label(&self.status);
            let previous_mode = self.origin_view;
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.origin_view, false, "Ergebnis");
                self.targets[0] = Some(ui.selectable_value(&mut self.origin_view, true, "Ursprünge / abgezogene Operanden").rect.center());
                ui.label("Ansicht zum Drehen ziehen");
                ui.checkbox(&mut self.show_internal, "Alle Zwischenknoten");
            });
            if self.origin_view != previous_mode { self.selected = None; self.hits.clear(); }
            if self.documents.revision().0 > 0 {
                ui.colored_label(egui::Color32::YELLOW, "Veraltete Vorschau · Quelländerung übernommen; Auswahl gesperrt");
            }
            ui.columns(3, |columns| {
                columns[0].heading("Ausgewertete Instanzen");
                if self.manifest.nodes.is_empty() { columns[0].label("Noch keine Instanzen vorhanden."); }
                egui::ScrollArea::vertical().id_salt("instances").show(&mut columns[0], |ui| {
                    for node in &self.manifest.nodes {
                        if !self.origin_view && node.subtracted { continue; }
                        if !self.show_internal && !matches!(node.kind.as_str(), "cube" | "cylinder" | "sphere" | "polyhedron" | "difference" | "union" | "intersection" | "linear_extrude" | "rotate_extrude") { continue; }
                        let label = format!("{} · {}{}", node.id, node.kind, if node.subtracted { " [−]" } else { "" });
                        let response = ui.add_enabled(self.documents.revision().0 == 0, egui::Button::selectable(self.selected == Some(node.id), label));
                        if node.subtracted && node.kind == "cylinder" && self.targets[1].is_none() { self.targets[1] = Some(response.rect.center()); }
                        if response.clicked() {
                            self.selected = Some(node.id);
                        }
                    }
                });
                columns[1].heading("Geometrie / Auswahl");
                let selected = self.selected.and_then(|id| self.meshes.get(&id));
                let (rect, clicked) = render::draw(&mut columns[1], &self.result, selected, &mut self.yaw);
                if let Some(point) = clicked && self.documents.revision().0 == 0
                    && let Some(bounds) = self.result.bounds() {
                    self.hits = self.manifest.nodes.iter().filter(|node| self.origin_view || !node.subtracted)
                        .filter(|node| self.meshes.get(&node.id).is_some_and(|mesh| render::hit_test(mesh, bounds, rect, self.yaw, point)))
                        .map(|node| node.id).collect();
                    self.selected = self.hits.last().copied();
                }
                columns[2].heading("Herkunft / Parameter");
                egui::ScrollArea::vertical().id_salt("properties").show(&mut columns[2], |ui| {
                    if self.manifest.nodes.is_empty() {
                        ui.label("Nach einem erfolgreichen M0-Lauf sind hier Aufrufpfade, Quellstellen und Parameter verfügbar.");
                        return;
                    }
                    if !self.hits.is_empty() {
                        ui.label("Treffer / Verschachtelung am Bildpunkt:");
                        ui.horizontal_wrapped(|ui| { for id in &self.hits {
                            if ui.selectable_label(self.selected == Some(*id), id.to_string()).clicked() { self.selected = Some(*id); }
                        }});
                    }
                    if let Some(node) = self.selected.and_then(|id| self.manifest.nodes.iter().find(|n| n.id == id)) {
                        ui.monospace(&node.evaluated);
                        for frame in node.origin.iter().filter(|frame| !frame.source.file.contains("BOSL2")) {
                            let file = Path::new(&frame.source.file).file_name().unwrap_or_default().to_string_lossy();
                            ui.label(format!("{} · {}:{}:{}", frame.call, file, frame.source.first_line, frame.source.first_column));
                            for key in ["i", "r", "base_height"] {
                                if let Some(value) = frame.context.get(key) { ui.monospace(format!("{key} = {value}")); }
                            }
                        }
                        ui.collapsing("Vollständige Bibliotheksaufrufe", |ui| {
                            for frame in node.origin.iter().filter(|frame| frame.source.file.contains("BOSL2")) {
                                ui.label(format!("{} · {}:{}", frame.call, Path::new(&frame.source.file).file_name().unwrap_or_default().to_string_lossy(), frame.source.first_line));
                            }
                        });
                    } else { ui.label("Instanz im Baum wählen. Gelb zeigt die ausgewählte Ursprungsgeometrie."); }
                    ui.separator();
                    ui.label("Beispielziel: base_height. Betroffen: Grundkörper und fünf Zylinder; der BOSL2-Körper bleibt unverändert.");
                    ui.add(egui::DragValue::new(&mut self.value).range(1.0..=100.0));
                    let response = ui.add_enabled(!self.manifest.nodes.is_empty(), egui::Button::new("Quellparameter ändern"));
                    self.targets[2] = Some(response.rect.center());
                    if response.clicked() {
                        let operation = (|| -> Result<()> {
                            let source = self.documents.text(DocumentId(1)).context("missing source")?;
                            let start = source.find("base_height = ").context("parameter not present")? + "base_height = ".len();
                            let end = start + source[start..].find(';').context("parameter terminator missing")?;
                            let plan = plan_numeric_literal(&self.documents, DocumentId(1), start..end, self.value)?;
                            self.documents.apply(plan)?;
                            Ok(())
                        })();
                        self.status = operation.map_or_else(|error| error.to_string(), |()| "Parameter geändert; Kommentar und Formel erhalten".into());
                        self.selected = None;
                    }
                    if let Some(mut source) = self.documents.text(DocumentId(1)) {
                        ui.add(egui::TextEdit::multiline(&mut source).code_editor().desired_width(ui.available_width()).desired_rows(10).interactive(false));
                    }
                });
            });
        });
        if let Some(mut probe) = self.probe.take() {
            probe.advance(self, root.ctx());
            self.probe = Some(probe);
        }
        if let Some(path) = std::env::var_os("SCADSTUDIO_SCREENSHOT")
            && !self.screenshot_requested
        {
            let context = root.ctx().clone();
            root.ctx().request_screenshot(move |capture| {
                std::thread::spawn(move || {
                    if let Err(error) = crate::offscreen::save_png(
                        &PathBuf::from(path),
                        capture.width() as u32,
                        capture.height() as u32,
                        capture.as_raw(),
                    ) {
                        eprintln!("Screenshot failed: {error}");
                    }
                    context.send_viewport_cmd(egui::ViewportCommand::Close);
                });
            });
            self.screenshot_requested = true;
        }
    }
}

/// Starts a native window, loading artifacts before the event loop or showing an empty state.
pub fn run(directory: Option<&Path>) -> Result<()> {
    let viewer = match directory {
        Some(directory) => ProofViewer::load(directory)?,
        None => ProofViewer::empty(),
    };
    let probe_directory = viewer.probe.as_ref().map(|probe| probe.directory.clone());
    eframe::run_native(
        "SCADStudio M0",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(viewer))),
    )
    .map_err(|error| anyhow::anyhow!("native UI: {error}"))?;
    if let Some(directory) = probe_directory {
        let report: serde_json::Value = serde_json::from_slice(
            &std::fs::read(directory.join("native-report.json"))
                .context("native UI probe did not finish")?,
        )?;
        anyhow::ensure!(report["passed"] == true, "native UI probe failed");
    }
    Ok(())
}

// Explicit local acceptance harness. Inactive during normal application use.
struct NativeProbe {
    directory: PathBuf,
    stage: u8,
    pending: Option<egui::Pos2>,
    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
    started: std::time::Instant,
    original: Option<String>,
}

impl NativeProbe {
    fn from_environment() -> Result<Option<Self>> {
        let Some(directory) = std::env::var_os("SCADSTUDIO_UI_PROBE") else {
            return Ok(None);
        };
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory)?;
        anyhow::ensure!(
            !directory.join("native-report.json").exists(),
            "choose a fresh native probe directory"
        );
        Ok(Some(Self {
            directory,
            stage: 0,
            pending: None,
            ready: Default::default(),
            started: std::time::Instant::now(),
            original: None,
        }))
    }

    fn screenshot(&self, context: &egui::Context, name: &str, finish: bool) {
        let path = self.directory.join(name);
        let directory = self.directory.clone();
        let ready = self.ready.clone();
        let context_copy = context.clone();
        context.request_screenshot(move |capture| {
            std::thread::spawn(move || {
                let result = (|| -> Result<()> {
                    crate::offscreen::save_png(&path, capture.width() as u32, capture.height() as u32, capture.as_raw())?;
                    if finish { anyhow::ensure!(directory.join("native-origin.png").is_file(), "origin screenshot missing"); std::fs::write(directory.join("native-report.json"), br#"{"passed":true,"actions":["origin_view","select_subtracted_operand","change_parameter"],"revision":1,"comment_and_formula_preserved":true,"stale_selection_cleared":true}"#)?; }
                    Ok(())
                })();
                if let Err(error) = result { eprintln!("Native proof capture failed: {error}"); }
                ready.store(true, std::sync::atomic::Ordering::Release);
                if finish { context_copy.send_viewport_cmd(egui::ViewportCommand::Close); }
                else { context_copy.request_repaint(); }
            });
        });
    }

    fn advance(&mut self, viewer: &ProofViewer, context: &egui::Context) {
        if self.started.elapsed() > std::time::Duration::from_secs(15) {
            eprintln!("Native UI proof timed out at stage {}", self.stage);
            context.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        match self.stage {
            0 if viewer.targets[0].is_some() => {
                self.original = viewer.documents.text(DocumentId(1)).map(String::from);
                self.pending = viewer.targets[0];
                self.stage = 1;
            }
            1 if viewer.origin_view && viewer.targets[1].is_some() => {
                self.pending = viewer.targets[1];
                self.stage = 2;
            }
            2 if viewer.selected.is_some_and(|id| {
                viewer.manifest.nodes.iter().any(|node| {
                    node.id == id
                        && node.subtracted
                        && !node.origin.is_empty()
                        && node.mesh.is_some()
                })
            }) =>
            {
                self.screenshot(context, "native-origin.png", false);
                self.stage = 3;
            }
            3 if self.ready.load(std::sync::atomic::Ordering::Acquire) => {
                self.pending = viewer.targets[2];
                self.stage = 4;
            }
            4 if viewer.documents.revision().0 == 1 && viewer.selected.is_none() => {
                let source = viewer.documents.text(DocumentId(1)).unwrap_or_default();
                if self.original.as_ref().is_some_and(|original| {
                    source == original.replacen("base_height = 10;", "base_height = 12;", 1)
                }) && source.contains("base_height = 12;")
                    && source.contains("// Keep the relation")
                    && source.contains("base_height * 2")
                {
                    self.screenshot(context, "native-edited.png", true);
                    self.stage = 5;
                }
            }
            _ => {}
        }
        context.request_repaint();
    }
}
