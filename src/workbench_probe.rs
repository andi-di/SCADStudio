//! Explicit native acceptance driver; inactive without SCADSTUDIO_M1_PROBE.
use super::*;
use anyhow::{Context, ensure};
pub(super) struct Probe {
    pub directory: PathBuf,
    pub events: Vec<egui::Event>,
    stage: u8,
    started: Instant,
    original: String,
    captured: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl Probe {
    pub fn from_environment(app: &mut Workbench) -> Result<Option<Self>> {
        let Some(directory) = std::env::var_os("SCADSTUDIO_M1_PROBE") else {
            return Ok(None);
        };
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join("report.json").exists(),
            "choose a fresh probe output directory"
        );
        let session = app.session.as_ref().context("no probe session")?;
        let original = session
            .documents
            .text(DocumentId(1))
            .context("no probe source")?
            .to_string();
        ensure!(
            original == "h = 10;\r\n// Größe und Formel erhalten\r\ncube([h * 2, 20, 10]);\r\n",
            "use documented disposable M1 probe fixture"
        );
        app.offset = 4;
        app.length = 2;
        app.value = 12.;
        Ok(Some(Self {
            directory,
            events: vec![],
            stage: 0,
            started: Instant::now(),
            original,
            captured: Default::default(),
        }))
    }
    fn click(&mut self, point: Option<egui::Pos2>) -> bool {
        let Some(pos) = point else { return false };
        self.events.push(egui::Event::PointerMoved(pos));
        for pressed in [true, false] {
            self.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        }
        true
    }
    fn key(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        for pressed in [true, false] {
            self.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            });
        }
    }
    fn capture(&mut self, ctx: &egui::Context, name: &str, finish: bool) {
        self.captured
            .store(false, std::sync::atomic::Ordering::Release);
        let path = self.directory.join(name);
        let report = self.directory.join("report.json");
        let captured = self.captured.clone();
        let ctx2 = ctx.clone();
        ctx.request_screenshot(move|image|{std::thread::spawn(move||{
            let result=(||->Result<()>{crate::offscreen::save_png(&path,image.width() as u32,image.height() as u32,image.as_raw())?;
                if finish {std::fs::write(report,serde_json::to_vec_pretty(&serde_json::json!({"passed":true,"actions":["preview","type_invalid_source","safe_literal_edit","undo","redo","external_conflict","keep_local","take_disk","save"],"last_valid_preview_retained":true}))?)?;}Ok(())})();
            if let Err(e)=result{eprintln!("M1 native proof: {e:#}");}captured.store(true,std::sync::atomic::Ordering::Release);if finish{ctx2.send_viewport_cmd(egui::ViewportCommand::Close);}else{ctx2.request_repaint();}
        });});
    }
    pub fn advance(&mut self, app: &mut Workbench, ctx: &egui::Context) {
        if self.started.elapsed() > Duration::from_secs(60) {
            eprintln!("M1 probe timeout at stage {}: {}", self.stage, app.status);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let Some(session) = &app.session else {
            ctx.request_repaint_after(Duration::from_millis(20));
            return;
        };
        let source = session.documents.text(DocumentId(1)).unwrap_or_default();
        let revision = session.documents.revision().0;
        match self.stage {
            0 => {
                if self.click(app.targets[0]) {
                    self.stage = 1;
                }
            }
            1 if app.preview_revision == Some(Revision(0)) => {
                if self.click(app.targets[1]) {
                    self.stage = 2;
                }
            }
            2 => {
                self.key(egui::Key::End, egui::Modifiers::CTRL);
                self.events.push(egui::Event::Text("\ncube([".into()));
                self.stage = 3;
            }
            3 if revision == 1 && source.ends_with("cube([") => {
                if self.click(app.targets[0]) {
                    self.stage = 4;
                }
            }
            4 if app.evaluation.is_none() && app.status.starts_with("Engine:") => {
                if self.click(app.targets[2]) {
                    self.stage = 5;
                }
            }
            5 if revision == 2
                && source == format!("{}\ncube([", self.original.replacen("10", "12", 1))
                && app.preview_revision == Some(Revision(0))
                && !app.preview.triangles.is_empty() =>
            {
                self.capture(ctx, "stale-preview.png", false);
                self.stage = 6;
            }
            6 if self.captured.load(std::sync::atomic::Ordering::Acquire) => {
                if self.click(app.targets[3]) {
                    self.stage = 7;
                }
            }
            7 if revision == 3 && source.starts_with("h = 10;") => {
                if self.click(app.targets[4]) {
                    self.stage = 8;
                }
            }
            8 if revision == 4 && source.starts_with("h = 12;") => {
                let path = session.files[&DocumentId(1)].path.clone();
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    if let Err(e) = std::fs::write(path, "sphere(3);\r\n") {
                        eprintln!("probe external write: {e}");
                    }
                    ctx.request_repaint();
                });
                self.stage = 9;
            }
            9 if !app.conflicts.is_empty() => {
                self.capture(ctx, "conflict.png", false);
                self.stage = 10;
            }
            10 if self.captured.load(std::sync::atomic::Ordering::Acquire) => {
                if self.click(app.targets[6]) {
                    self.stage = 11;
                }
            }
            11 if app.conflicts.is_empty() && session.dirty(DocumentId(1)) => {
                self.key(egui::Key::S, egui::Modifiers::CTRL);
                self.stage = 12;
            }
            12 if !session.dirty(DocumentId(1)) => {
                let path = session.files[&DocumentId(1)].path.clone();
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let _ = std::fs::write(path, "sphere(4);\r\n");
                    ctx.request_repaint();
                });
                self.stage = 13;
            }
            13 if !app.conflicts.is_empty() => {
                if self.click(app.targets[5]) {
                    self.stage = 14;
                }
            }
            14 if source == "sphere(4);\r\n" && !session.dirty(DocumentId(1)) => {
                self.capture(ctx, "resolved.png", true);
                self.stage = 15;
            }
            _ => {}
        }
        ctx.request_repaint_after(Duration::from_millis(20));
    }
}
