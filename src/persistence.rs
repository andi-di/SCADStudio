//! Immutable worker filesystem views, including unsaved includes and resource bytes.
use crate::document::Revision;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};
use tempfile::TempDir;

/// A complete explicit file set. Source bytes may differ from the on-disk originals.
#[derive(Clone)]
pub struct EvaluationSnapshot {
    pub revision: Revision,
    pub entrypoint: PathBuf,
    pub files: BTreeMap<PathBuf, Vec<u8>>,
}

/// Owned private view; removed only after its associated job and artifacts are consumed.
pub struct WorkerView {
    pub directory: TempDir,
    pub entrypoint: PathBuf,
    pub fingerprint: String,
}

pub fn validate_relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("invalid snapshot path: {}", path.display());
    }
    Ok(())
}

impl EvaluationSnapshot {
    /// Materializes a fresh private tree, never writes to original project files.
    /// M0 accepts complete explicit snapshots, not implicit filesystem discovery.
    pub fn materialize(&self) -> Result<WorkerView> {
        validate_relative(&self.entrypoint)?;
        if !self.files.contains_key(&self.entrypoint) {
            bail!("entrypoint missing from snapshot");
        }
        for path in self.files.keys() {
            validate_relative(path)?;
        }
        let directory = tempfile::Builder::new()
            .prefix("scadstudio-job-")
            .tempdir()?;
        let mut hash = Sha256::new();
        hash.update(self.revision.0.to_le_bytes());
        let entry = self.entrypoint.as_os_str().as_encoded_bytes();
        hash.update((entry.len() as u64).to_le_bytes());
        hash.update(entry);
        for (path, bytes) in &self.files {
            let encoded = path.as_os_str().as_encoded_bytes();
            hash.update((encoded.len() as u64).to_le_bytes());
            hash.update(encoded);
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
            let target = directory.path().join(path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&target, bytes).with_context(|| format!("stage {}", path.display()))?;
        }
        let entrypoint = directory.path().join(&self.entrypoint);
        Ok(WorkerView {
            directory,
            entrypoint,
            fingerprint: format!("{:x}", hash.finalize()),
        })
    }
}

/// Fingerprints actual dependency files, including untracked resources. Internal
/// symlinks are refused in M0; the explicitly configured library root may be a link.
pub fn library_fingerprint(root: &Path) -> Result<String> {
    fn collect(
        root: &Path,
        directory: &Path,
        files: &mut BTreeMap<PathBuf, Vec<u8>>,
    ) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if entry.file_name() == ".git" {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                collect(root, &entry.path(), files)?;
            } else if kind.is_file() {
                files.insert(
                    entry.path().strip_prefix(root)?.to_path_buf(),
                    fs::read(entry.path())?,
                );
            } else {
                bail!("unsupported library resource: {}", entry.path().display());
            }
        }
        Ok(())
    }
    let root = root.canonicalize()?;
    let mut files = BTreeMap::new();
    collect(&root, &root, &mut files)?;
    let mut hash = Sha256::new();
    for (path, bytes) in files {
        let encoded = path.as_os_str().as_encoded_bytes();
        hash.update((encoded.len() as u64).to_le_bytes());
        hash.update(encoded);
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Ok(format!("sha256:{:x}", hash.finalize()))
}
