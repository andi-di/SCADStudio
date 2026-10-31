//! Executable M0 acceptance probes; any failed assertion returns a nonzero status.
use crate::{
    application::{ModelState, SelectionMode},
    document::{DocumentId, Documents, Revision},
    engine::{Engine, EvaluationResult},
    language::plan_numeric_literal,
    offscreen,
    persistence::EvaluationSnapshot,
    render::Mesh,
};
use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

fn cancel() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

/// Standard, unannotated fixture covering loops, modules, children and BOSL2.
pub fn origins_snapshot() -> EvaluationSnapshot {
    EvaluationSnapshot {
        revision: Revision(0),
        entrypoint: "main.scad".into(),
        files: BTreeMap::from([(
            PathBuf::from("main.scad"),
            include_bytes!("../assets/m0/origins.scad").to_vec(),
        )]),
    }
}

/// Immutable overlay with nested Unicode includes, CRLF and an adjacent SVG resource.
pub fn resources_snapshot() -> EvaluationSnapshot {
    EvaluationSnapshot {
        revision: Revision(3),
        entrypoint: "main.scad".into(),
        files: BTreeMap::from([
            (
                PathBuf::from("main.scad"),
                include_bytes!("../assets/m0/resources/main.scad").to_vec(),
            ),
            (
                PathBuf::from("nested/maße.scad"),
                include_bytes!("../assets/m0/resources/nested/maße.scad").to_vec(),
            ),
            (
                PathBuf::from("nested/deeper/value.scad"),
                "// Ungespeicherte Höhe\r\nheight_value = 7;\r\n"
                    .as_bytes()
                    .to_vec(),
            ),
            (
                PathBuf::from("nested/shape.svg"),
                include_bytes!("../assets/m0/resources/nested/shape.svg").to_vec(),
            ),
        ]),
    }
}

fn mesh(path: &Path) -> Result<Mesh> {
    Mesh::from_stl(&fs::read(path)?)
}

fn compare(
    reference: &Engine,
    result: &EvaluationResult,
    output: &str,
) -> Result<serde_json::Value> {
    reference.run(
        &result.view.entrypoint,
        &result.artifacts,
        output,
        None,
        cancel(),
    )?;
    let actual = mesh(&result.artifacts.join("result.stl"))?;
    let expected = mesh(&result.artifacts.join(output))?;
    let a = actual.bounds().context("empty actual geometry")?;
    let b = expected.bounds().context("empty reference geometry")?;
    let size = (0..3)
        .map(|axis| f64::from(b[1][axis] - b[0][axis]))
        .fold(0.0_f64, f64::max);
    let length_tolerance = 1e-6_f64.max(1e-6 * size);
    for side in 0..2 {
        for axis in 0..3 {
            ensure!(
                f64::from((a[side][axis] - b[side][axis]).abs()) <= length_tolerance,
                "reference bounds differ"
            );
        }
    }
    let actual_topology = actual.topology();
    let reference_topology = expected.topology();
    ensure!(
        actual_topology == reference_topology && actual_topology.1,
        "reference components or closed-edge topology differ"
    );
    let expected_volume = expected.volume();
    ensure!(
        (actual.volume() - expected_volume).abs() <= 1e-9_f64.max(1e-6 * expected_volume),
        "reference volume differs"
    );
    Ok(
        json!({"volume": actual.volume(), "reference_volume": expected_volume, "bounds": a, "triangles": actual.triangles.len(), "components": actual_topology.0, "closed_edges": actual_topology.1}),
    )
}

pub fn run(
    worker: PathBuf,
    reference: PathBuf,
    library_root: PathBuf,
    output: &Path,
) -> Result<()> {
    fs::create_dir_all(output)?;
    let engine = Engine {
        executable: worker,
        library_root: library_root.clone(),
        timeout: Duration::from_secs(120),
    };
    let reference = Engine {
        executable: reference,
        library_root,
        timeout: engine.timeout,
    };
    let expected_lock: serde_json::Value =
        serde_json::from_str(include_str!("../engine/reference.lock.json"))?;
    ensure!(
        crate::persistence::library_fingerprint(&engine.library_root.join("BOSL2"))?
            == expected_lock["bosl2"]["content_hash"],
        "BOSL2 differs from locked content"
    );
    let capabilities = engine.capabilities()?;
    let snapshot = origins_snapshot();
    let result = engine
        .evaluate(snapshot.clone(), cancel())
        .context("origins evaluation")?;
    let nodes = &result.manifest.nodes;
    let holes: Vec<_> = nodes
        .iter()
        .filter(|node| node.kind == "cylinder" && node.subtracted)
        .collect();
    ensure!(
        holes.len() == 3,
        "AT-003: expected three subtracted loop instances, found {}",
        holes.len()
    );
    let mut iterations = BTreeSet::new();
    let mut events = BTreeSet::new();
    for node in &holes {
        let leaf = node.origin.last().context("missing evaluated origin")?;
        iterations.insert(
            node.origin
                .iter()
                .rev()
                .find_map(|frame| frame.context.get("i"))
                .context("missing iteration context")?
                .clone(),
        );
        events.insert(leaf.event);
        ensure!(
            node.origin.iter().any(|frame| frame.call == "outer"),
            "missing outer module call"
        );
        ensure!(
            node.origin.iter().any(|frame| frame.call == "inner"),
            "missing inner module call"
        );
        ensure!(
            node.origin
                .iter()
                .any(|frame| frame.call == "cylinder" && frame.source.file.ends_with("main.scad")),
            "wrong source document"
        );
        ensure!(
            node.mesh.is_some(),
            "AT-008: subtracted operand geometry missing"
        );
    }
    ensure!(
        iterations == BTreeSet::from(["0".into(), "1".into(), "2".into()]),
        "wrong loop contexts"
    );
    ensure!(
        events.len() == 3,
        "loop instances have identical evaluated identities"
    );
    let children: Vec<_> = nodes
        .iter()
        .filter(|node| node.kind == "cylinder" && !node.subtracted)
        .collect();
    ensure!(children.len() == 2, "expected two children() instances");
    for node in &children {
        ensure!(
            node.origin.iter().any(|frame| frame.call == "children"),
            "missing children() call path"
        );
        ensure!(
            node.origin.iter().any(|frame| frame.call == "twice"),
            "missing parent module"
        );
    }
    ensure!(
        children[0].origin.last().map(|frame| frame.event)
            != children[1].origin.last().map(|frame| frame.event),
        "children() identity collision"
    );
    ensure!(
        children[0].world_matrix != children[1].world_matrix,
        "children() world transforms lost"
    );
    ensure!(
        nodes.iter().any(|node| node.mesh.is_some()
            && node.origin.iter().any(|frame| frame.call == "cuboid")
            && node.source.file.contains("BOSL2")),
        "BOSL2 evaluated geometry/source path missing"
    );
    let geometry = compare(&reference, &result, "reference.stl")?;

    for entry in fs::read_dir(&result.artifacts)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), output.join(entry.file_name()))?;
        }
    }
    fs::write(
        output.join("source.scad"),
        &snapshot.files[Path::new("main.scad")],
    )?;
    // Persist relative provenance for the viewer, never host-specific temporary paths.
    let mut manifest = result.manifest.clone();
    for node in &mut manifest.nodes {
        for source in std::iter::once(&mut node.source)
            .chain(node.origin.iter_mut().map(|frame| &mut frame.source))
        {
            if let Some(index) = source.file.find("BOSL2/") {
                source.file = source.file[index..].to_string();
            } else if let Ok(path) =
                Path::new(&source.file).strip_prefix(result.view.directory.path())
            {
                source.file = path.to_string_lossy().into();
            }
        }
    }
    fs::write(output.join("manifest.json"), manifest.encode()?)?;
    engine
        .run(
            &result.view.entrypoint,
            output,
            "engine.png",
            None,
            cancel(),
        )
        .context("display-free EGL image")?;
    let adapter = offscreen::capture(&mesh(&output.join("result.stl"))?, &output.join("wgpu.png"))
        .context("display-free wgpu image")?;

    let mut documents = Documents::new(BTreeMap::from([(
        DocumentId(1),
        String::from_utf8(snapshot.files[Path::new("main.scad")].clone())?,
    )]));
    let start = documents
        .text(DocumentId(1))
        .context("missing source")?
        .find("base_height = 10")
        .context("missing parameter")?
        + "base_height = ".len();
    let plan = plan_numeric_literal(&documents, DocumentId(1), start..start + 2, 12.0)?;
    documents.apply(plan.clone())?;
    ensure!(documents.apply(plan).is_err(), "AT-006 stale plan accepted");
    let modified = documents
        .text(DocumentId(1))
        .context("missing edited source")?;
    ensure!(
        modified.contains("// Keep the relation") && modified.contains("base_height * 2"),
        "AT-002 comment/formula lost"
    );
    let changed = EvaluationSnapshot {
        revision: documents.revision(),
        files: BTreeMap::from([(PathBuf::from("main.scad"), modified.as_bytes().to_vec())]),
        ..snapshot
    };
    let changed = engine.evaluate(changed, cancel())?;
    let edited_geometry = compare(&reference, &changed, "reference.stl")?;
    let hole = changed
        .manifest
        .nodes
        .iter()
        .find(|node| node.kind == "cylinder" && node.subtracted)
        .context("missing current operand")?
        .id;
    let mut state = ModelState::default();
    ensure!(
        !state.accept(Revision(1), result),
        "late result replaced current geometry"
    );
    ensure!(
        state.accept(Revision(1), changed),
        "current result rejected"
    );
    ensure!(
        state
            .select(Revision(1), hole, SelectionMode::Result)
            .is_err(),
        "subtracted operand visible as result"
    );
    ensure!(
        state
            .select(Revision(1), hole, SelectionMode::Origin)
            .is_ok(),
        "operand not selectable in origin view"
    );
    ensure!(
        state
            .select(Revision(2), hole, SelectionMode::Origin)
            .is_err(),
        "stale selection accepted"
    );

    let resources = engine.evaluate(resources_snapshot(), cancel())?;
    let resource_geometry = compare(&reference, &resources, "reference.stl")?;
    let bounds = mesh(&resources.artifacts.join("result.stl"))?
        .bounds()
        .context("missing imported geometry")?;
    ensure!(
        (bounds[1][2] - 7.0).abs() < 1e-6,
        "unsaved nested include was not used"
    );
    ensure!(
        include_str!("../assets/m0/resources/nested/deeper/value.scad")
            .contains("height_value = 2"),
        "original include modified"
    );

    let unknown = EvaluationSnapshot { revision: Revision(4), entrypoint: "unbekannt.scad".into(), files: BTreeMap::from([
        (PathBuf::from("unbekannt.scad"), "// Keine Metadaten\r\nmodule arbitrary(n) { /* Größe */ sphere(r=n, $fn=16); }\r\narbitrary(3);\r\n".as_bytes().to_vec())
    ]) };
    let unknown_snapshot = unknown.clone();
    let unknown = engine.evaluate(unknown, cancel())?;
    ensure!(
        unknown
            .manifest
            .nodes
            .iter()
            .any(|node| node.kind == "sphere"
                && node.origin.iter().any(|frame| frame.call == "arbitrary")),
        "AT-001 unknown standard source not mapped"
    );
    let unknown_geometry = compare(&reference, &unknown, "reference.stl")?;
    let sphere = unknown
        .manifest
        .nodes
        .iter()
        .find(|node| node.kind == "sphere")
        .context("missing sphere")?;
    let source = std::str::from_utf8(&unknown_snapshot.files[Path::new("unbekannt.scad")])?;
    let coordinates = &sphere.source;
    let sphere_range = crate::language::source_range(
        source,
        coordinates.first_line.try_into()?,
        coordinates.first_column.try_into()?,
        coordinates.last_line.try_into()?,
        coordinates.last_column.try_into()?,
    )?;
    ensure!(
        source[sphere_range].starts_with("sphere("),
        "UTF-8/CRLF engine source mapping failed"
    );
    let call = sphere
        .origin
        .iter()
        .find(|frame| frame.call == "arbitrary")
        .context("missing arbitrary call")?;
    let range = crate::language::source_range(
        source,
        call.source.first_line.try_into()?,
        call.source.first_column.try_into()?,
        call.source.last_line.try_into()?,
        call.source.last_column.try_into()?,
    )?;
    let literal = range.start
        + source[range.clone()]
            .find('3')
            .context("missing explicit argument")?;
    let mut unknown_documents = Documents::new(BTreeMap::from([(DocumentId(2), source.into())]));
    let plan = plan_numeric_literal(&unknown_documents, DocumentId(2), literal..literal + 1, 4.0)?;
    unknown_documents.apply(plan)?;
    let edited_source = unknown_documents
        .text(DocumentId(2))
        .context("missing unknown edited source")?;
    ensure!(
        edited_source == source.replacen("arbitrary(3)", "arbitrary(4)", 1),
        "unrelated unknown source bytes changed"
    );
    let edited_unknown = engine.evaluate(
        EvaluationSnapshot {
            revision: Revision(5),
            files: BTreeMap::from([(
                PathBuf::from("unbekannt.scad"),
                edited_source.as_bytes().to_vec(),
            )]),
            ..unknown_snapshot
        },
        cancel(),
    )?;
    let unknown_edited_geometry = compare(&reference, &edited_unknown, "reference.stl")?;
    let report = json!({"milestone": "M0", "automated_spikes": "passed", "not_full_product_acceptance": true,
        "worker_capabilities": capabilities, "loop_instances": 3, "children_instances": 2, "origins_from_evaluator": true,
        "geometry": geometry, "edited_geometry": edited_geometry, "resource_geometry": resource_geometry,
        "unknown_geometry": unknown_geometry, "unknown_edited_geometry": unknown_edited_geometry, "offscreen_adapter": adapter,
        "png_paths": ["engine.png", "wgpu.png"],
        "limits": ["No full language corpus", "No fresh-system offline packaging test", "No final legal release clearance", "M0 painter projection has no production depth buffer"]});
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
