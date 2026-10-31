//! Explicitly ignored unless the separately built foreign engines are supplied.
#[test]
#[ignore = "requires locked reference, patched worker, BOSL2, EGL and wgpu; see README"]
fn m0_real_engine_acceptance_spikes() -> anyhow::Result<()> {
    let output = tempfile::tempdir()?;
    scadstudio::proof::run(
        std::env::var_os("SCADSTUDIO_WORKER")
            .ok_or_else(|| anyhow::anyhow!("SCADSTUDIO_WORKER missing"))?
            .into(),
        std::env::var_os("SCADSTUDIO_REFERENCE")
            .ok_or_else(|| anyhow::anyhow!("SCADSTUDIO_REFERENCE missing"))?
            .into(),
        std::env::var_os("SCADSTUDIO_LIBRARIES")
            .ok_or_else(|| anyhow::anyhow!("SCADSTUDIO_LIBRARIES missing"))?
            .into(),
        output.path(),
    )
}
