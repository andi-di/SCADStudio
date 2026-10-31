//! File baselines, explicit conflict resolution and atomic per-file replacement.
use crate::document::{ChangeSet, DocumentId, Documents, Revision, TextEdit};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Disk baseline stays separate from authoritative unsaved text.
#[derive(Clone)]
pub struct FileSession {
    pub documents: Documents,
    pub files: BTreeMap<DocumentId, FileState>,
}
#[derive(Clone)]
pub struct FileState {
    pub path: PathBuf,
    baseline: String,
}
#[derive(Clone, Debug)]
pub struct Conflict {
    pub document: DocumentId,
    pub baseline: String,
    pub local: String,
    pub disk: String,
}
#[derive(Clone, Copy)]
pub enum Resolution {
    TakeDisk,
    KeepLocal,
}

impl FileSession {
    pub fn open(paths: &[PathBuf]) -> Result<Self> {
        let mut files = BTreeMap::new();
        let mut texts = BTreeMap::new();
        for (index, path) in paths.iter().enumerate() {
            let path = path
                .canonicalize()
                .with_context(|| format!("open {}", path.display()))?;
            if files.values().any(|f: &FileState| f.path == path) {
                bail!("duplicate document");
            }
            let text = read_source(&path).with_context(|| format!("read {}", path.display()))?;
            let id = DocumentId(index as u64 + 1);
            texts.insert(id, text.clone());
            files.insert(
                id,
                FileState {
                    path,
                    baseline: text,
                },
            );
        }
        Ok(Self {
            documents: Documents::new(texts),
            files,
        })
    }
    /// Reconciles completed background I/O without replacing text entered meanwhile.
    pub fn merge_io(&mut self, base: Revision, completed: Self) -> Result<()> {
        if completed.documents.revision() != base && self.documents.revision() != base {
            bail!("revision_conflict: source changed while resolving disk conflict");
        }
        if completed.documents.revision() != base {
            self.documents = completed.documents;
        }
        self.files = completed.files;
        Ok(())
    }

    pub fn dirty(&self, id: DocumentId) -> bool {
        self.files
            .get(&id)
            .is_some_and(|file| self.documents.text(id) != Some(file.baseline.as_str()))
    }
    /// Poll from a background task. Detection never overwrites local source.
    pub fn scan(&self) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        for (&id, file) in &self.files {
            let disk = read_source(&file.path)
                .with_context(|| format!("check {}", file.path.display()))?;
            if disk != file.baseline {
                conflicts.push(Conflict {
                    document: id,
                    baseline: file.baseline.clone(),
                    local: self.documents.text(id).context("missing document")?.into(),
                    disk,
                });
            }
        }
        Ok(conflicts)
    }
    /// A reviewed conflict is bound to both current memory and current disk contents.
    pub fn resolve(&mut self, conflict: &Conflict, resolution: Resolution) -> Result<()> {
        let file = self
            .files
            .get_mut(&conflict.document)
            .context("unknown document")?;
        if file.baseline != conflict.baseline
            || self.documents.text(conflict.document) != Some(&conflict.local)
            || read_source(&file.path)? != conflict.disk
        {
            bail!("conflict changed; compare again");
        }
        if matches!(resolution, Resolution::TakeDisk) {
            self.documents.apply(ChangeSet {
                base: self.documents.revision(),
                origin: "external-file".into(),
                description: "Take reviewed disk version".into(),
                edits: vec![TextEdit {
                    document: conflict.document,
                    range: 0..conflict.local.len(),
                    replacement: conflict.disk.clone(),
                }],
            })?;
        }
        file.baseline = conflict.disk.clone();
        Ok(())
    }
    /// Saves independently; every result names its file. Failures retain dirty state.
    pub fn save(&mut self) -> Vec<(DocumentId, Result<()>)> {
        let mut results = Vec::new();
        for (&id, file) in &mut self.files {
            let result = (|| {
                let text = self.documents.text(id).context("missing document")?;
                if read_source(&file.path)? != file.baseline {
                    bail!("external_change: compare before saving");
                }
                if fs::metadata(&file.path)?.permissions().readonly() {
                    bail!("read_only: {}", file.path.display());
                }
                atomic_write_checked(
                    &file.path,
                    text.as_bytes(),
                    Expected::Existing(file.baseline.as_bytes()),
                )?;
                file.baseline = text.into();
                Ok(())
            })();
            results.push((id, result));
        }
        results
    }
}

/// Flushes and replaces only after the complete new content is written.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    atomic_write_checked(path, bytes, Expected::Unchecked)
}
enum Expected<'a> {
    Unchecked,
    Existing(&'a [u8]),
    Absent,
}
fn atomic_write_checked(path: &Path, bytes: &[u8], expected: Expected<'_>) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).context("create temporary sibling")?;
    if let Ok(metadata) = fs::metadata(path) {
        temporary
            .as_file()
            .set_permissions(metadata.permissions())?;
    }
    temporary
        .write_all(bytes)
        .context("write temporary sibling")?;
    temporary
        .as_file()
        .sync_all()
        .context("flush temporary sibling")?;
    if let Expected::Existing(expected) = expected {
        anyhow::ensure!(
            crate::engine::read_bounded(path, 16 * 1024 * 1024)? == expected,
            "external_change: target changed during save"
        );
    }
    let replacement = if matches!(expected, Expected::Absent) {
        temporary.persist_noclobber(path)
    } else {
        temporary.persist(path)
    };
    replacement.map_err(|e| anyhow::anyhow!("replace {}: {}", path.display(), e.error))?;
    Ok(())
}

/// Shared metadata contains no local layout, history, tokens or recovery state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectMetadata {
    pub schema_version: u32,
    pub name: String,
    pub entrypoint: PathBuf,
    pub engine: EngineBinding,
    pub libraries: Vec<LibraryBinding>,
    pub search_paths: Vec<PathBuf>,
    pub external_resources: Vec<PathBuf>,
    pub customizer: Option<CustomizerBinding>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineBinding {
    pub reference_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryBinding {
    pub name: String,
    pub revision: String,
    pub content_hash: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomizerBinding {
    pub file: PathBuf,
    pub preset: String,
}
impl ProjectMetadata {
    pub fn read(path: &Path) -> Result<Self> {
        let value: Self = serde_json::from_slice(&crate::engine::read_bounded(path, 1024 * 1024)?)
            .context("invalid project metadata; open .scad separately")?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            bail!("unsupported project schema; open .scad separately");
        }
        anyhow::ensure!(
            !self.name.trim().is_empty() && !self.engine.reference_id.trim().is_empty(),
            "missing project name or engine reference"
        );
        for library in &self.libraries {
            anyhow::ensure!(
                !library.name.is_empty()
                    && library.revision.len() == 40
                    && library.revision.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid library revision"
            );
            let hash = library
                .content_hash
                .strip_prefix("sha256:")
                .context("invalid library hash")?;
            anyhow::ensure!(
                hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid library hash"
            );
        }
        if let Some(customizer) = &self.customizer {
            crate::persistence::validate_relative(&customizer.file)?;
        }
        crate::persistence::validate_relative(&self.entrypoint)?;
        for path in &self.search_paths {
            crate::persistence::validate_relative(path)?;
        }
        Ok(())
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        // An unreadable or future existing format must never be silently replaced.
        let baseline = if path.exists() {
            let bytes = crate::engine::read_bounded(path, 1024 * 1024)?;
            let existing: Self = serde_json::from_slice(&bytes)?;
            existing.validate()?;
            Some(bytes)
        } else {
            None
        };
        atomic_write_checked(
            path,
            &serde_json::to_vec_pretty(self)?,
            baseline
                .as_deref()
                .map_or(Expected::Absent, Expected::Existing),
        )
    }
}

fn read_source(path: &Path) -> Result<String> {
    String::from_utf8(crate::engine::read_bounded(path, 16 * 1024 * 1024)?)
        .with_context(|| format!("invalid UTF-8 source: {}", path.display()))
}
