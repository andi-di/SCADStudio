use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.first().and_then(|arg| arg.to_str()) {
        None => scadstudio::ui::run(
            Path::new("target/m0-proof/manifest.json")
                .exists()
                .then_some(Path::new("target/m0-proof")),
        ),
        Some("prove-m0") if args.len() == 5 => scadstudio::proof::run(
            PathBuf::from(&args[1]),
            PathBuf::from(&args[2]),
            PathBuf::from(&args[3]),
            Path::new(&args[4]),
        ),
        Some("view-m0") if args.len() == 2 => scadstudio::ui::run(Some(Path::new(&args[1]))),
        Some("edit") if args.len() > 1 => {
            scadstudio::workbench::run(args[1..].iter().map(PathBuf::from).collect())
        }
        Some(_) if args.len() == 1 && Path::new(&args[0]).is_file() => {
            scadstudio::workbench::run(vec![PathBuf::from(&args[0])])
        }
        _ => bail!(
            "Usage:\n  scadstudio <file.scad|project.scadstudio.json>\n  scadstudio edit <file.scad> [more-files.scad ...]\n  scadstudio prove-m0 <worker> <reference> <library-root> <output-directory>\n  scadstudio view-m0 <proof-directory>"
        ),
    }
}
