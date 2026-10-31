//! One isolated OpenSCAD process per immutable snapshot; bounded artifacts and deadline.
use crate::{
    document::Revision,
    persistence::{EvaluationSnapshot, WorkerView, validate_relative},
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

/// Original parser coordinates: one-based, end-exclusive byte columns (mapped separately).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EngineSource {
    pub file: String,
    pub first_line: i64,
    pub first_column: i64,
    pub last_line: i64,
    pub last_column: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// One actual evaluated call, with a shared immutable lexical-context snapshot.
pub struct OriginFrame {
    pub event: u64,
    pub call: String,
    pub source: EngineSource,
    pub context: Arc<BTreeMap<String, String>>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// A calculation-local tree node; its identity must never cross revisions unchecked.
pub struct Instance {
    pub id: u64,
    pub parent: u64,
    pub kind: String,
    pub evaluated: String,
    pub origin: Vec<OriginFrame>,
    pub source: EngineSource,
    pub world_matrix: [[f64; 4]; 4],
    pub subtracted: bool,
    pub mesh: Option<String>,
}
#[derive(Clone, Debug)]
/// Validated provenance with contexts shared across repeated call paths.
pub struct Manifest {
    pub protocol_version: u32,
    pub snapshot_id: String,
    pub revision: u64,
    pub nodes: Vec<Instance>,
}

/// Keeps the temporary artifact directory alive; not a durable document identity.
pub struct EvaluationResult {
    pub revision: Revision,
    pub snapshot_id: String,
    pub manifest: Manifest,
    pub view: WorkerView,
    pub artifacts: PathBuf,
}

#[derive(Clone)]
/// Explicit engine executable and library binding for isolated local jobs.
pub struct Engine {
    pub executable: PathBuf,
    pub library_root: PathBuf,
    pub timeout: Duration,
}

impl Engine {
    /// Queries the private worker protocol before scheduling real work.
    pub fn capabilities(&self) -> Result<serde_json::Value> {
        let output = Command::new(&self.executable)
            .arg("--scadstudio-capabilities")
            .output()?;
        if !output.status.success() {
            bail!("worker capability query failed");
        }
        let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        if value["protocol_version"] != 1
            || value["engine_revision"] != "9ba0cee7c94785fac0ca8dfe040b965a12a25ed1"
        {
            bail!("unsupported worker protocol or engine revision");
        }
        Ok(value)
    }

    /// Executes synchronously on a caller-owned background thread. The job owns its
    /// child and kills/reaps it on cancellation, timeout, and error.
    pub fn evaluate(
        &self,
        snapshot: EvaluationSnapshot,
        cancel: Arc<AtomicBool>,
    ) -> Result<EvaluationResult> {
        let mut view = snapshot.materialize()?;
        let mut identity = Sha256::new();
        identity.update(view.fingerprint.as_bytes());
        identity.update(read_bounded(&self.executable, 256 * 1024 * 1024)?);
        identity.update(include_bytes!("../engine/reference.lock.json"));
        let library = self.library_root.join("BOSL2");
        if library.is_dir() {
            identity.update(crate::persistence::library_fingerprint(&library)?.as_bytes());
        }
        if let Some(root) = self.executable.parent().and_then(Path::parent) {
            for resource in ["shaders", "color-schemes", "fonts"] {
                let path = root.join(resource);
                if path.is_dir() {
                    identity.update(crate::persistence::library_fingerprint(&path)?.as_bytes());
                }
            }
        }
        view.fingerprint = format!("{:x}", identity.finalize());
        let artifacts = view.directory.path().join("artifacts");
        fs::create_dir(&artifacts)?;
        fs::write(
            artifacts.join("request.json"),
            serde_json::to_vec(&serde_json::json!({
                "protocol_version": 1, "snapshot_id": view.fingerprint, "revision": snapshot.revision.0
            }))?,
        )?;
        self.run(
            &view.entrypoint,
            &artifacts,
            "result.stl",
            Some(&artifacts),
            cancel,
        )?;
        let data = read_bounded(&artifacts.join("manifest.json"), 32 * 1024 * 1024)?;
        let manifest: Manifest = Manifest::decode(&data).context("decode worker provenance")?;
        if manifest.protocol_version != 1
            || manifest.nodes.len() > 4096
            || manifest.snapshot_id != view.fingerprint
            || manifest.revision != snapshot.revision.0
        {
            bail!("unsupported worker manifest");
        }
        let ids: std::collections::BTreeSet<_> =
            manifest.nodes.iter().map(|node| node.id).collect();
        if ids.len() != manifest.nodes.len() {
            bail!("duplicate worker instance ids");
        }
        for node in &manifest.nodes {
            if node.parent != 0 && !ids.contains(&node.parent) {
                bail!("missing worker parent");
            }
            if let Some(mesh) = &node.mesh {
                validate_relative(Path::new(mesh))?;
                if mesh != &format!("{}.stl", node.id) {
                    bail!("unbound mesh artifact");
                }
                read_bounded(&artifacts.join(mesh), 64 * 1024 * 1024)?;
            }
        }
        Ok(EvaluationResult {
            revision: snapshot.revision,
            snapshot_id: view.fingerprint.clone(),
            manifest,
            view,
            artifacts,
        })
    }

    /// Standard unmodified engine export, also used for reference and headless PNG.
    pub fn run(
        &self,
        source: &Path,
        directory: &Path,
        output: &str,
        trace: Option<&Path>,
        cancel: Arc<AtomicBool>,
    ) -> Result<()> {
        validate_relative(Path::new(output))?;
        let log_path = directory.join(format!("{output}.log"));
        let log = fs::File::create(&log_path)?;
        let mut cmd = Command::new(&self.executable);
        cmd.arg("--backend=Manifold")
            .arg("--export-format")
            .arg(if output.ends_with(".png") {
                "png"
            } else {
                "binstl"
            })
            .arg("-o")
            .arg(directory.join(output))
            .arg(source)
            .env("OPENSCADPATH", &self.library_root)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("SCADSTUDIO_TRACE_DIR")
            .env("EGL_PLATFORM", "surfaceless")
            .env("LIBGL_ALWAYS_SOFTWARE", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::from(log));
        if output.ends_with(".png") {
            cmd.args(["--render", "--autocenter", "--viewall", "--imgsize=640,480"]);
        }
        if let Some(path) = trace {
            cmd.env("SCADSTUDIO_TRACE_DIR", path);
        }
        let mut child = cmd
            .spawn()
            .with_context(|| format!("start {}", self.executable.display()))?;
        let start = Instant::now();
        loop {
            if cancel.load(Ordering::Acquire) || start.elapsed() > self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                bail!(
                    "worker cancelled or timed out: {}",
                    diagnostic_excerpt(&log_path)
                );
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        bail!(
                            "worker failed ({status}): {}",
                            diagnostic_excerpt(&log_path)
                        );
                    }
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error).context("poll worker");
                }
            }
        }
        if output.ends_with(".png") && diagnostic_excerpt(&log_path).contains("ERROR:") {
            bail!(
                "engine image diagnostics: {}",
                diagnostic_excerpt(&log_path)
            );
        }
        if !directory.join(output).is_file() {
            bail!("worker produced no {output}");
        }
        Ok(())
    }
}

/// Reads at most `limit` artifact bytes and rejects larger files.
pub fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        bail!("artifact exceeds limit: {}", path.display());
    }
    Ok(bytes)
}

fn diagnostic_excerpt(path: &Path) -> String {
    use std::io::Read;
    let mut bytes = Vec::new();
    if let Ok(file) = fs::File::open(path) {
        let _ = file.take(64 * 1024).read_to_end(&mut bytes);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[derive(Deserialize, Serialize)]
struct WireFrame {
    event: u64,
    call: String,
    source: EngineSource,
    context: u64,
}
#[derive(Deserialize, Serialize)]
struct WireInstance {
    id: u64,
    parent: u64,
    kind: String,
    evaluated: String,
    origin: Vec<u64>,
    source: EngineSource,
    world_matrix: [[f64; 4]; 4],
    subtracted: bool,
    mesh: Option<String>,
}
#[derive(Deserialize, Serialize)]
struct WireManifest {
    protocol_version: u32,
    snapshot_id: String,
    revision: u64,
    contexts: BTreeMap<u64, Arc<BTreeMap<String, String>>>,
    frames: BTreeMap<u64, WireFrame>,
    nodes: Vec<WireInstance>,
}

impl Manifest {
    /// Resolves validated shared references. Contexts stay shared in memory, too.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let wire: WireManifest = serde_json::from_slice(bytes)?;
        if wire.nodes.len() > 4096
            || wire
                .nodes
                .iter()
                .map(|node| node.origin.len())
                .sum::<usize>()
                > 262144
        {
            bail!("worker provenance exceeds node/path reference budget");
        }
        let mut nodes = Vec::with_capacity(wire.nodes.len());
        for node in wire.nodes {
            let mut origin = Vec::with_capacity(node.origin.len());
            for event in node.origin {
                let frame = wire
                    .frames
                    .get(&event)
                    .ok_or_else(|| anyhow::anyhow!("missing origin frame"))?;
                if frame.event != event {
                    bail!("origin event mismatch");
                }
                let context = wire
                    .contexts
                    .get(&frame.context)
                    .ok_or_else(|| anyhow::anyhow!("missing origin context"))?
                    .clone();
                origin.push(OriginFrame {
                    event,
                    call: frame.call.clone(),
                    source: frame.source.clone(),
                    context,
                });
            }
            nodes.push(Instance {
                id: node.id,
                parent: node.parent,
                kind: node.kind,
                evaluated: node.evaluated,
                origin,
                source: node.source,
                world_matrix: node.world_matrix,
                subtracted: node.subtracted,
                mesh: node.mesh,
            });
        }
        Ok(Self {
            protocol_version: wire.protocol_version,
            snapshot_id: wire.snapshot_id,
            revision: wire.revision,
            nodes,
        })
    }

    /// Writes a compact table of unique frames and contexts; never duplicates the
    /// complete BOSL2 context for every occurrence along every node's call path.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut contexts = BTreeMap::new();
        let mut context_ids = BTreeMap::new();
        let mut frames = BTreeMap::new();
        let mut nodes = Vec::new();
        for node in &self.nodes {
            for frame in &node.origin {
                let next = (context_ids.len() + 1) as u64;
                let id = *context_ids.entry(frame.context.clone()).or_insert(next);
                contexts.entry(id).or_insert_with(|| frame.context.clone());
                frames.entry(frame.event).or_insert_with(|| WireFrame {
                    event: frame.event,
                    call: frame.call.clone(),
                    source: frame.source.clone(),
                    context: id,
                });
            }
            nodes.push(WireInstance {
                id: node.id,
                parent: node.parent,
                kind: node.kind.clone(),
                evaluated: node.evaluated.clone(),
                origin: node.origin.iter().map(|frame| frame.event).collect(),
                source: node.source.clone(),
                world_matrix: node.world_matrix,
                subtracted: node.subtracted,
                mesh: node.mesh.clone(),
            });
        }
        Ok(serde_json::to_vec_pretty(&WireManifest {
            protocol_version: self.protocol_version,
            snapshot_id: self.snapshot_id.clone(),
            revision: self.revision,
            contexts,
            frames,
            nodes,
        })?)
    }
}
