use scadstudio::engine::{EngineSource, Instance, Manifest, OriginFrame};
use std::{collections::BTreeMap, sync::Arc};

#[test]
fn repeated_large_contexts_remain_bounded_and_lossless() -> anyhow::Result<()> {
    let source = EngineSource {
        file: "main.scad".into(),
        first_line: 1,
        first_column: 1,
        last_line: 1,
        last_column: 9,
    };
    let value = "x".repeat(700_000);
    let context = Arc::new(BTreeMap::from([
        ("large_value".into(), value.clone()),
        ("i".into(), "2".into()),
    ]));
    let nodes = (1..=64)
        .map(|id| Instance {
            id,
            parent: 0,
            kind: "cube".into(),
            evaluated: "cube(1)".into(),
            source: source.clone(),
            origin: vec![OriginFrame {
                event: id,
                call: "cube".into(),
                source: source.clone(),
                context: context.clone(),
            }],
            world_matrix: [[0.0; 4]; 4],
            subtracted: false,
            mesh: None,
        })
        .collect();
    let manifest = Manifest {
        protocol_version: 1,
        snapshot_id: "test".into(),
        revision: 3,
        nodes,
    };
    let encoded = manifest.encode()?;
    assert!(
        encoded.len() < 1_000_000,
        "shared 700 KB context must not become a >32 MiB manifest"
    );
    let decoded = Manifest::decode(&encoded)?;
    assert_eq!(decoded.nodes.len(), 64);
    for node in decoded.nodes {
        assert_eq!(node.origin[0].context.get("large_value"), Some(&value));
    }
    Ok(())
}

#[test]
fn rejects_dangling_context_and_frame_references() {
    let bytes = br#"{"protocol_version":1,"snapshot_id":"test","revision":0,"contexts":{},"frames":{"1":{"event":1,"call":"cube","source":{"file":"main.scad","first_line":1,"first_column":1,"last_line":1,"last_column":9},"context":99}},"nodes":[{"id":1,"parent":0,"kind":"cube","evaluated":"cube(1)","source":{"file":"main.scad","first_line":1,"first_column":1,"last_line":1,"last_column":9},"world_matrix":[[0,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,0,0]],"subtracted":false,"mesh":null,"origin":[1]}]}"#;
    assert!(Manifest::decode(bytes).is_err());
    let mut json: serde_json::Value = serde_json::from_slice(bytes).unwrap_or_default();
    json["nodes"][0]["origin"] = serde_json::json!([100]);
    assert!(Manifest::decode(json.to_string().as_bytes()).is_err());
}
