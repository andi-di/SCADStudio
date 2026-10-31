//! UI-independent UTF-8 byte edits. No language execution happens here.
use anyhow::{Result, bail};
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

/// Opaque identity in one session, independent of the worker's filesystem paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DocumentId(pub u64);

/// A monotonically increasing session revision.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Revision(pub u64);

/// Half-open UTF-8 byte range at the change set's base revision.
#[derive(Clone, Debug)]
pub struct TextEdit {
    pub document: DocumentId,
    pub range: Range<usize>,
    pub replacement: String,
}

/// A single atomic operation; preserves all bytes outside the explicit edits.
#[derive(Clone, Debug)]
pub struct ChangeSet {
    pub base: Revision,
    pub origin: String,
    pub description: String,
    pub edits: Vec<TextEdit>,
}

/// Authoritative unsaved source. Derived geometry is deliberately absent.
#[derive(Clone, Default)]
pub struct Documents {
    revision: Revision,
    texts: Arc<BTreeMap<DocumentId, String>>,
    history: Arc<Vec<ChangeSet>>,
    undo: Arc<Vec<Arc<BTreeMap<DocumentId, String>>>>,
    redo: Arc<Vec<Arc<BTreeMap<DocumentId, String>>>>,
}

impl Documents {
    /// Creates the initial source state without an edit transaction.
    pub fn new(texts: BTreeMap<DocumentId, String>) -> Self {
        Self {
            texts: Arc::new(texts),
            ..Self::default()
        }
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn text(&self, document: DocumentId) -> Option<&str> {
        self.texts.get(&document).map(String::as_str)
    }
    pub fn history(&self) -> &[ChangeSet] {
        &self.history
    }

    /// Validates every edit before changing any document. Rejects stale revisions,
    /// overlapping ranges and byte offsets inside a UTF-8 code point.
    pub fn apply(&mut self, changes: ChangeSet) -> Result<Revision> {
        if changes.base != self.revision {
            bail!("revision_conflict");
        }
        if changes.edits.is_empty() {
            bail!("empty_change_set");
        }
        let next = self
            .revision
            .0
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("revision_overflow"))?;
        let mut grouped: BTreeMap<DocumentId, Vec<&TextEdit>> = BTreeMap::new();
        for edit in &changes.edits {
            grouped.entry(edit.document).or_default().push(edit);
        }
        for (id, edits) in &mut grouped {
            let text = self
                .text(*id)
                .ok_or_else(|| anyhow::anyhow!("unknown_document"))?;
            edits.sort_by_key(|edit| (edit.range.start, edit.range.end));
            for (index, edit) in edits.iter().enumerate() {
                if edit.range.start > edit.range.end
                    || edit.range.end > text.len()
                    || !text.is_char_boundary(edit.range.start)
                    || !text.is_char_boundary(edit.range.end)
                {
                    bail!("invalid_utf8_range");
                }
                if index > 0 && edits[index - 1].range.end >= edit.range.start {
                    // Adjacent replacements are allowed, coincident insertions aren't.
                    if edits[index - 1].range.end > edit.range.start
                        || edits[index - 1].range.is_empty()
                        || edit.range.is_empty()
                    {
                        bail!("overlapping_edits");
                    }
                }
            }
        }
        Arc::make_mut(&mut self.undo).push(self.texts.clone());
        Arc::make_mut(&mut self.redo).clear();
        for (id, edits) in grouped {
            if let Some(text) = Arc::make_mut(&mut self.texts).get_mut(&id) {
                for edit in edits.into_iter().rev() {
                    text.replace_range(edit.range.clone(), &edit.replacement);
                }
            }
        }
        self.revision = Revision(next);
        Arc::make_mut(&mut self.history).push(changes);
        Ok(self.revision)
    }
}

impl Documents {
    /// Restores content as a new revision; stale history commands never mutate state.
    pub fn undo(&mut self, expected: Revision) -> Result<Revision> {
        self.restore(expected, false)
    }

    /// Reapplies the last undone transaction as a new revision.
    pub fn redo(&mut self, expected: Revision) -> Result<Revision> {
        self.restore(expected, true)
    }

    fn restore(&mut self, expected: Revision, redo: bool) -> Result<Revision> {
        if expected != self.revision {
            bail!("revision_conflict");
        }
        let next = self
            .revision
            .0
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("revision_overflow"))?;
        let stack = if redo { &mut self.redo } else { &mut self.undo };
        let restored = Arc::make_mut(stack)
            .pop()
            .ok_or_else(|| anyhow::anyhow!("history_empty"))?;
        let edits = restored
            .iter()
            .filter_map(|(&document, text)| {
                let current = self.texts.get(&document)?;
                (current != text).then(|| TextEdit {
                    document,
                    range: 0..current.len(),
                    replacement: text.clone(),
                })
            })
            .collect();
        let previous = std::mem::replace(&mut self.texts, restored);
        if redo {
            Arc::make_mut(&mut self.undo).push(previous);
        } else {
            Arc::make_mut(&mut self.redo).push(previous);
        }
        self.revision = Revision(next);
        Arc::make_mut(&mut self.history).push(ChangeSet {
            base: expected,
            origin: "history".into(),
            description: if redo { "Redo" } else { "Undo" }.into(),
            edits,
        });
        Ok(self.revision)
    }

    /// Immutable full source set for snapshots and persistence.
    pub fn texts(&self) -> &BTreeMap<DocumentId, String> {
        &self.texts
    }
}

/// Confirmed source position. A range from an old revision is never implicitly reused.
#[derive(Clone, Debug)]
pub struct SourceRange {
    pub document: DocumentId,
    pub revision: Revision,
    pub bytes: Range<usize>,
}
