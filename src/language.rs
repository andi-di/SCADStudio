//! M0 explicit numeric-token change, not a substitute OpenSCAD parser/interpreter.
use crate::document::{ChangeSet, DocumentId, Documents, TextEdit};
use anyhow::{Result, bail};
use std::ops::Range;

/// Edits an explicitly chosen literal without replacing surrounding expressions.
/// The caller must supply a confirmed current source range, never a mesh-derived offset.
pub fn plan_numeric_literal(
    documents: &Documents,
    id: DocumentId,
    range: Range<usize>,
    value: f64,
) -> Result<ChangeSet> {
    if !value.is_finite() {
        bail!("non-finite literal");
    }
    let source = documents
        .text(id)
        .ok_or_else(|| anyhow::anyhow!("unknown document"))?;
    let token = source
        .get(range.clone())
        .ok_or_else(|| anyhow::anyhow!("invalid range"))?;
    if token.trim() != token || !token.parse::<f64>().is_ok_and(f64::is_finite) || token.is_empty()
    {
        bail!("target is not an explicit numeric literal");
    }
    if !crate::syntax::Syntax::parse(source).numeric_token(&range) {
        bail!("stale_target: not a complete current numeric token in a safe region");
    }
    Ok(ChangeSet {
        base: documents.revision(),
        origin: "parameter-form".into(),
        description: "Change explicitly selected source literal".into(),
        edits: vec![TextEdit {
            document: id,
            range,
            replacement: value.to_string(),
        }],
    })
}

/// Converts OpenSCAD's lexer coordinates to half-open UTF-8 bytes. CR counts as a
/// byte; LF starts a new line. Bounds and code point boundaries are always checked.
pub fn source_range(
    source: &str,
    first_line: usize,
    first_column: usize,
    last_line: usize,
    last_column: usize,
) -> Result<Range<usize>> {
    let offset = |line: usize, column: usize| -> Result<usize> {
        if line == 0 || column == 0 {
            bail!("invalid source coordinate");
        }
        let start = if line == 1 {
            0
        } else {
            source
                .match_indices('\n')
                .nth(line - 2)
                .map(|(index, _)| index + 1)
                .ok_or_else(|| anyhow::anyhow!("line outside document"))?
        };
        let result = start
            .checked_add(column - 1)
            .ok_or_else(|| anyhow::anyhow!("coordinate overflow"))?;
        let end = source[start..]
            .find('\n')
            .map_or(source.len(), |index| start + index);
        if result > end {
            bail!("column outside line");
        }
        Ok(result)
    };
    let start = offset(first_line, first_column)?;
    let end = offset(last_line, last_column)?;
    if source.get(start..end).is_none() {
        bail!("source coordinates split UTF-8 or exceed document");
    }
    Ok(start..end)
}

/// Plans a literal edit against an explicitly revision-bound source target.
pub fn plan_numeric_target(
    documents: &Documents,
    target: &crate::document::SourceRange,
    value: f64,
) -> Result<ChangeSet> {
    if target.revision != documents.revision() {
        bail!("stale_target: source revision changed; select the current token again");
    }
    plan_numeric_literal(documents, target.document, target.bytes.clone(), value)
}
