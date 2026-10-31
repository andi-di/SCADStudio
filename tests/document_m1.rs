use anyhow::Result;
use scadstudio::{
    document::{ChangeSet, DocumentId, Documents, Revision, TextEdit},
    files::{FileSession, ProjectMetadata, Resolution},
    language::plan_numeric_literal,
    syntax::Syntax,
};
use std::{collections::BTreeMap, fs};

fn replace(documents: &Documents, id: DocumentId, text: &str) -> ChangeSet {
    ChangeSet {
        base: documents.revision(),
        origin: "code".into(),
        description: "Edit source".into(),
        edits: vec![TextEdit {
            document: id,
            range: 0..documents.text(id).unwrap().len(),
            replacement: text.into(),
        }],
    }
}
#[test]
fn history_is_atomic_monotonic_and_branches() -> Result<()> {
    let a = DocumentId(1);
    let b = DocumentId(2);
    let mut docs = Documents::new(BTreeMap::from([(a, "10".into()), (b, "20".into())]));
    let plan = ChangeSet {
        base: Revision(0),
        origin: "form".into(),
        description: "Two files".into(),
        edits: vec![
            TextEdit {
                document: a,
                range: 0..2,
                replacement: "12".into(),
            },
            TextEdit {
                document: b,
                range: 0..2,
                replacement: "24".into(),
            },
        ],
    };
    docs.apply(plan.clone())?;
    docs.undo(Revision(1))?;
    assert_eq!(docs.text(a), Some("10"));
    assert_eq!(docs.text(b), Some("20"));
    docs.redo(Revision(2))?;
    assert_eq!(docs.text(b), Some("24"));
    assert!(docs.apply(plan).is_err());
    assert!(docs.undo(Revision(1)).is_err());
    docs.undo(Revision(3))?;
    docs.apply(replace(&docs, a, "30"))?;
    assert!(docs.redo(docs.revision()).is_err());
    Ok(())
}
#[test]
fn at_007_safe_independent_region_and_complete_tokens() -> Result<()> {
    let id = DocumentId(1);
    let source = "// Größe\r\nh = 10;\r\ncube([h, ;";
    let mut docs = Documents::new(BTreeMap::from([(id, source.into())]));
    assert!(!Syntax::parse(source).issues.is_empty());
    let start = source.find("10").unwrap();
    docs.apply(plan_numeric_literal(&docs, id, start..start + 2, 12.)?)?;
    assert_eq!(docs.text(id), Some(source.replace("10", "12").as_str()));
    assert!(plan_numeric_literal(&docs, id, start..start + 1, 3.).is_err());
    let comment = Documents::new(BTreeMap::from([(id, "// 10\ncube(2);".into())]));
    assert!(plan_numeric_literal(&comment, id, 3..5, 12.).is_err());
    Ok(())
}
#[test]
fn at_016_conflict_requires_explicit_review_and_is_undoable() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("maße.scad");
    fs::write(&path, "cube(10);\r\n")?;
    let mut session = FileSession::open(std::slice::from_ref(&path))?;
    let id = DocumentId(1);
    session
        .documents
        .apply(replace(&session.documents, id, "cube(12);\r\n"))?;
    fs::write(&path, "cube(20);\r\n")?;
    let conflict = session.scan()?.remove(0);
    assert_eq!(conflict.baseline, "cube(10);\r\n");
    assert!(session.save().remove(0).1.is_err());
    assert!(session.dirty(id));
    session.resolve(&conflict, Resolution::KeepLocal)?;
    assert!(session.dirty(id));
    session.save().remove(0).1?;
    assert!(!session.dirty(id));
    assert_eq!(fs::read_to_string(&path)?, "cube(12);\r\n");
    fs::write(&path, "sphere(3);\r\n")?;
    let conflict = session.scan()?.remove(0);
    session.resolve(&conflict, Resolution::TakeDisk)?;
    session.documents.undo(session.documents.revision())?;
    assert_eq!(session.documents.text(id), Some("cube(12);\r\n"));
    assert!(session.dirty(id));
    Ok(())
}
#[test]
fn stale_conflict_cannot_overwrite_new_inputs() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("a.scad");
    fs::write(&path, "a=1;")?;
    let mut session = FileSession::open(std::slice::from_ref(&path))?;
    fs::write(&path, "a=2;")?;
    let conflict = session.scan()?.remove(0);
    fs::write(&path, "a=3;")?;
    assert!(session.resolve(&conflict, Resolution::TakeDisk).is_err());
    assert_eq!(session.documents.text(DocumentId(1)), Some("a=1;"));
    Ok(())
}
#[test]
fn at_017_never_overwrites_corrupt_or_future_metadata() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("model.scadstudio.json");
    let valid = r#"{"schema_version":1,"name":"Test","entrypoint":"main.scad","engine":{"reference_id":"9ba0cee7c94785fac0ca8dfe040b965a12a25ed1"},"libraries":[],"search_paths":[],"external_resources":[],"customizer":null}"#;
    fs::write(&path, valid)?;
    let metadata = ProjectMetadata::read(&path)?;
    let source = dir.path().join("main.scad");
    fs::write(&source, "cube(1);")?;
    for damaged in [
        "{broken",
        &valid.replace("\"schema_version\":1", "\"schema_version\":2"),
    ] {
        fs::write(&path, damaged)?;
        assert!(ProjectMetadata::read(&path).is_err());
        assert!(metadata.save(&path).is_err());
        assert_eq!(fs::read_to_string(&path)?, damaged);
        let session = FileSession::open(std::slice::from_ref(&source))?;
        assert_eq!(session.documents.text(DocumentId(1)), Some("cube(1);"));
    }
    Ok(())
}

#[test]
fn syntax_retains_every_byte_and_limits_hostile_nesting() {
    use scadstudio::syntax::{Kind, Node, StatementKind};
    fn leaves(nodes: &[Node], source: &str, output: &mut String) {
        for node in nodes {
            if node.kind == Kind::Group {
                output.push_str(&source[node.range.start..node.range.start + 1]);
                leaves(&node.children, source, output);
                if matches!(source.as_bytes()[node.range.end - 1], b')' | b']' | b'}') {
                    output.push_str(&source[node.range.end - 1..node.range.end]);
                }
            } else {
                output.push_str(&source[node.range.clone()]);
            }
        }
    }
    let source = "// ä\r\nmodule example(x=2) { translate([x,0,0]) cube(2); }\r\nh=10;\r\ncube([";
    let syntax = Syntax::parse(source);
    let mut restored = String::new();
    leaves(&syntax.nodes, source, &mut restored);
    assert_eq!(restored, source);
    assert!(!syntax.issues.is_empty());
    let statements = syntax.statements(source);
    assert_eq!(statements[0].kind, StatementKind::Module);
    assert_eq!(statements[1].kind, StatementKind::Assignment);
    let hostile = "(".repeat(100_000);
    assert!(!Syntax::parse(&hostile).issues.is_empty());
}

#[test]
fn partial_save_identifies_failed_file_and_preserves_dirty_content() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let a = dir.path().join("a.scad");
    let b = dir.path().join("b.scad");
    fs::write(&a, "a=1;")?;
    fs::write(&b, "b=1;")?;
    let mut session = FileSession::open(&[a.clone(), b.clone()])?;
    session
        .documents
        .apply(replace(&session.documents, DocumentId(1), "a=2;"))?;
    session
        .documents
        .apply(replace(&session.documents, DocumentId(2), "b=2;"))?;
    fs::remove_file(&b)?;
    fs::create_dir(&b)?;
    let results = session.save();
    assert!(results[0].1.is_ok());
    assert!(results[1].1.is_err());
    assert_eq!(fs::read_to_string(&a)?, "a=2;");
    assert!(!session.dirty(DocumentId(1)));
    assert!(session.dirty(DocumentId(2)));
    assert_eq!(session.documents.text(DocumentId(2)), Some("b=2;"));
    Ok(())
}

#[test]
fn old_source_target_is_rejected_even_when_new_bytes_look_numeric() -> Result<()> {
    use scadstudio::{document::SourceRange, language::plan_numeric_target};
    let id = DocumentId(1);
    let mut docs = Documents::new(BTreeMap::from([(id, "a=10;".into())]));
    let target = SourceRange {
        document: id,
        revision: docs.revision(),
        bytes: 2..4,
    };
    docs.apply(replace(&docs, id, "b=20;"))?;
    assert!(plan_numeric_target(&docs, &target, 12.).is_err());
    assert_eq!(docs.text(id), Some("b=20;"));
    Ok(())
}

#[test]
fn background_save_retains_input_typed_while_writing() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("a.scad");
    fs::write(&path, "a=1;")?;
    let mut session = FileSession::open(std::slice::from_ref(&path))?;
    let id = DocumentId(1);
    session
        .documents
        .apply(replace(&session.documents, id, "a=2;"))?;
    let base = session.documents.revision();
    let mut io = session.clone();
    session
        .documents
        .apply(replace(&session.documents, id, "a=3;"))?;
    io.save().remove(0).1?;
    session.merge_io(base, io)?;
    assert_eq!(fs::read_to_string(&path)?, "a=2;");
    assert_eq!(session.documents.text(id), Some("a=3;"));
    assert!(session.dirty(id));
    Ok(())
}
#[test]
fn background_conflict_take_cannot_replace_new_input() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("a.scad");
    fs::write(&path, "a=1;")?;
    let mut session = FileSession::open(std::slice::from_ref(&path))?;
    let id = DocumentId(1);
    fs::write(&path, "a=2;")?;
    let conflict = session.scan()?.remove(0);
    let base = session.documents.revision();
    let mut io = session.clone();
    io.resolve(&conflict, Resolution::TakeDisk)?;
    session
        .documents
        .apply(replace(&session.documents, id, "a=3;"))?;
    assert!(session.merge_io(base, io).is_err());
    assert_eq!(session.documents.text(id), Some("a=3;"));
    assert_eq!(session.scan()?.len(), 1);
    Ok(())
}

#[test]
fn signed_literals_remain_editable_without_consuming_binary_operators() -> Result<()> {
    let id = DocumentId(1);
    let mut docs = Documents::new(BTreeMap::from([(id, "a=-10; b=a-10;".into())]));
    docs.apply(plan_numeric_literal(&docs, id, 2..5, -12.)?)?;
    docs.apply(plan_numeric_literal(&docs, id, 2..5, -14.)?)?;
    assert!(plan_numeric_literal(&docs, id, 10..13, 20.).is_err());
    assert_eq!(docs.text(id), Some("a=-14; b=a-10;"));
    Ok(())
}

#[test]
fn readonly_save_keeps_original_and_dirty_state() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("locked.scad");
    fs::write(&path, "cube(1);")?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions)?;
    let mut session = FileSession::open(std::slice::from_ref(&path))?;
    session
        .documents
        .apply(replace(&session.documents, DocumentId(1), "cube(2);"))?;
    assert!(session.save().remove(0).1.is_err());
    assert!(session.dirty(DocumentId(1)));
    assert_eq!(fs::read_to_string(&path)?, "cube(1);");
    Ok(())
}

#[test]
fn unclosed_outer_group_is_reported_when_last_byte_closes_inner_group() {
    let source = "cube((10)";
    let syntax = Syntax::parse(source);
    assert!(!syntax.issues.is_empty());
    assert!(!syntax.numeric_token(&(6..8)));
}

#[test]
fn versioned_metadata_roundtrips_without_local_state() -> Result<()> {
    use scadstudio::files::EngineBinding;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("project.scadstudio.json");
    let metadata = ProjectMetadata {
        schema_version: 1,
        name: "Example".into(),
        entrypoint: "src/main.scad".into(),
        engine: EngineBinding {
            reference_id: "9ba0cee7c94785fac0ca8dfe040b965a12a25ed1".into(),
        },
        libraries: vec![],
        search_paths: vec![],
        external_resources: vec![],
        customizer: None,
    };
    metadata.save(&path)?;
    assert_eq!(
        ProjectMetadata::read(&path)?.entrypoint,
        metadata.entrypoint
    );
    let json: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    for field in ["theme", "history", "layout", "token", "recovery"] {
        assert!(json.get(field).is_none());
    }
    metadata.save(&path)?;
    Ok(())
}
