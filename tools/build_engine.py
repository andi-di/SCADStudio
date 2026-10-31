#!/usr/bin/env python3
"""Explicit network/build setup, never run by the application. No system installs."""
import argparse
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
LOCK = json.loads((ROOT / "engine/reference.lock.json").read_text())


def run(*args, **kwargs):
    subprocess.run([str(arg) for arg in args], check=True, **kwargs)


def fetch(path, repository, revision):
    path.mkdir(parents=True)
    run("git", "-C", path, "init")
    run("git", "-C", path, "remote", "add", "origin", repository)
    run("git", "-C", path, "fetch", "--depth=1", "origin", revision)
    run("git", "-C", path, "checkout", "--detach", "FETCH_HEAD")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=pathlib.Path)
    parser.add_argument("--jobs", type=int, default=2)
    args = parser.parse_args()
    directory = args.directory.resolve()
    if directory.exists():
        raise RuntimeError("Choose a new empty directory; existing builds are never overwritten")
    directory.mkdir(parents=True)
    source, build = directory / "source", directory / "build"
    upstream, bosl = LOCK["openscad"], LOCK["bosl2"]
    fetch(source, upstream["repository"], upstream["revision"])
    run("git", "-C", source, "submodule", "update", "--init", "--recursive", "--depth=1")
    for path, revision in upstream["submodules"].items():
        actual = subprocess.check_output(["git", "-C", str(source / path), "rev-parse", "HEAD"], text=True).strip()
        if actual != revision:
            raise RuntimeError(f"Submodule mismatch: {path}")
    fetch(directory / "libraries/BOSL2", bosl["repository"], bosl["revision"])
    options = [f"-D{key}={value}" for key, value in upstream["options"].items()]
    run("cmake", "-S", source, "-B", build, "-G", "Ninja", *options)
    run("cmake", "--build", build, "--target", "openscad", "-j", args.jobs)
    (directory / "reference").mkdir()
    shutil.copy2(build / "openscad", directory / "reference/openscad")
    # Keep an unchanged source worktree as well as the unchanged reference binary.
    run("git", "-C", source, "worktree", "add", "--detach", directory / "reference-source", upstream["revision"])
    run(sys.executable, ROOT / "tools/prepare_worker.py", source)
    run("cmake", "-S", source, "-B", build, "-G", "Ninja", *options)
    run("cmake", "--build", build, "--target", "openscad", "-j", args.jobs)
    (directory / "worker").mkdir()
    shutil.copy2(build / "openscad", directory / "worker/openscad")
    # Upstream discovers resources next to the binary or one directory above it.
    for resource in ("shaders", "color-schemes", "locale", "fonts"):
        shutil.copytree(source / resource, directory / resource)
    for license_path in [source / "COPYING", directory / "libraries/BOSL2/LICENSE"]:
        target = directory / ("OpenSCAD-COPYING" if license_path.name == "COPYING" else "BOSL2-LICENSE")
        shutil.copy2(license_path, target)
    evidence = {"reference_sha256": hashlib.sha256((directory / "reference/openscad").read_bytes()).hexdigest(),
                "worker_sha256": hashlib.sha256((directory / "worker/openscad").read_bytes()).hexdigest(),
                "lock": LOCK}
    (directory / "build-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
