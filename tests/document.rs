use scadstudio::{
    document::{ChangeSet, DocumentId, Documents, Revision, TextEdit},
    language::{plan_numeric_literal, source_range},
    persistence::EvaluationSnapshot,
};
use std::{collections::BTreeMap, path::PathBuf};

#[test]
fn at_002_preserves_comments_crlf_unicode_and_formula() -> anyhow::Result<()> {
    let source = "base_height = 10;\r\n// Größe erhalten\r\ncube([30, 20, base_height * 2]);\r\n";
    let id = DocumentId(1);
    let mut documents = Documents::new(BTreeMap::from([(id, source.into())]));
    let plan = plan_numeric_literal(&documents, id, 14..16, 12.0)?;
    documents.apply(plan.clone())?;
    assert_eq!(
        documents.text(id),
        Some(source.replacen("10", "12", 1).as_str())
    );
    assert!(
        documents.apply(plan).is_err(),
        "AT-006: stale plans must fail"
    );
    assert_eq!(documents.revision(), Revision(1));
    assert_eq!(documents.history().len(), 1);
    Ok(())
}

#[test]
fn rejects_invalid_multifile_change_without_partial_mutation() {
    let mut documents = Documents::new(BTreeMap::from([
        (DocumentId(1), "cube(1);".into()),
        (DocumentId(2), "// ä".into()),
    ]));
    let changes = ChangeSet {
        base: Revision(0),
        origin: "test".into(),
        description: "Atomic change".into(),
        edits: vec![
            TextEdit {
                document: DocumentId(1),
                range: 5..6,
                replacement: "2".into(),
            },
            TextEdit {
                document: DocumentId(2),
                range: 3..4,
                replacement: "x".into(),
            },
        ],
    };
    assert!(documents.apply(changes).is_err());
    assert_eq!(documents.text(DocumentId(1)), Some("cube(1);"));
    assert_eq!(documents.revision(), Revision(0));
}

#[test]
fn maps_engine_byte_coordinates_across_unicode_and_crlf() -> anyhow::Result<()> {
    let source = "// ä\r\ntranslate([0,0,0]) cube(2);\r\n";
    let range = source_range(source, 2, 20, 2, 27)?;
    assert_eq!(&source[range], "cube(2)");
    assert!(source_range(source, 0, 1, 1, 1).is_err());
    assert!(source_range(source, 1, 5, 1, 6).is_err());
    Ok(())
}

#[test]
fn rejects_snapshot_traversal_and_missing_entrypoint() {
    let snapshot = EvaluationSnapshot {
        revision: Revision(0),
        entrypoint: "main.scad".into(),
        files: BTreeMap::from([
            (PathBuf::from("main.scad"), b"cube(1);".to_vec()),
            (PathBuf::from("../outside"), vec![]),
        ]),
    };
    assert!(snapshot.materialize().is_err());
    let empty = EvaluationSnapshot {
        files: BTreeMap::new(),
        ..snapshot
    };
    assert!(empty.materialize().is_err());
}

#[test]
fn rejects_formula_replacement_and_nonfinite_values() {
    let documents = Documents::new(BTreeMap::from([(DocumentId(1), "height * 2".into())]));
    assert!(plan_numeric_literal(&documents, DocumentId(1), 0..10, 24.0).is_err());
    assert!(plan_numeric_literal(&documents, DocumentId(1), 9..10, f64::NAN).is_err());
}
