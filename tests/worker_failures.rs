use scadstudio::{
    document::Revision,
    engine::{Engine, read_bounded},
    persistence::EvaluationSnapshot,
    render::Mesh,
};
use std::{
    collections::BTreeMap,
    fs,
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

#[test]
fn rejects_truncated_and_nonfinite_meshes() {
    assert!(Mesh::from_stl(&[0; 20]).is_err());
    let mut bytes = vec![0; 134];
    bytes[80..84].copy_from_slice(&1_u32.to_le_bytes());
    bytes[96..100].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(Mesh::from_stl(&bytes).is_err());
}

#[test]
fn bounds_artifact_reads() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("artifact");
    fs::write(&path, [0; 128])?;
    assert!(read_bounded(&path, 64).is_err());
    assert_eq!(read_bounded(&path, 128)?.len(), 128);
    Ok(())
}

#[cfg(unix)]
#[test]
fn cancellation_and_timeout_kill_and_reap_isolated_worker() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir()?;
    let worker = directory.path().join("blocked-worker");
    fs::write(&worker, "#!/bin/sh\nexec sleep 30\n")?;
    fs::set_permissions(&worker, fs::Permissions::from_mode(0o700))?;
    let engine = Engine {
        executable: worker,
        library_root: directory.path().into(),
        timeout: Duration::from_millis(30),
    };
    let snapshot = EvaluationSnapshot {
        revision: Revision(0),
        entrypoint: "main.scad".into(),
        files: BTreeMap::from([("main.scad".into(), b"cube(1);".to_vec())]),
    };
    for cancelled in [true, false] {
        let start = Instant::now();
        assert!(
            engine
                .evaluate(snapshot.clone(), Arc::new(AtomicBool::new(cancelled)))
                .is_err()
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    Ok(())
}

#[test]
fn projected_hit_rejects_points_outside_geometry() {
    use eframe::egui::{Pos2, Rect, vec2};
    use scadstudio::render::hit_test;
    let mesh = Mesh {
        triangles: vec![[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 0.0, 2.0]]],
    };
    let bounds = [[0.0; 3], [2.0; 3]];
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(200.0, 200.0));
    assert!(hit_test(&mesh, bounds, rect, 0.0, Pos2::new(100.0, 100.0)));

    assert!(!hit_test(&mesh, bounds, rect, 0.0, Pos2::new(0.0, 0.0)));
}

#[test]
fn near_edge_on_triangles_do_not_emit_unbounded_stroke_vertices() {
    use eframe::egui::{self, Pos2, Rect, vec2};
    let mesh = Mesh {
        triangles: vec![[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 0.0, 0.000001]]],
    };
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(200.0, 200.0));
    let context = egui::Context::default();
    let mut output = context.run_pass(
        egui::RawInput {
            screen_rect: Some(rect),
            ..Default::default()
        },
        |ctx| {
            ctx.layer_painter(egui::LayerId::background())
                .extend(scadstudio::render::shapes(
                    &mesh,
                    [[0.0; 3], [10.0; 3]],
                    rect,
                    0.0,
                    false,
                ));
        },
    );
    output.textures_delta.clear();
    let primitives = context.tessellate(output.shapes, 1.0);
    assert!(!primitives.is_empty());
    for primitive in primitives {
        if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
            assert!(mesh.vertices.iter().all(|vertex| rect.contains(vertex.pos)));
        }
    }
}
