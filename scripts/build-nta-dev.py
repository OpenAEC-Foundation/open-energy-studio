#!/usr/bin/env python3
"""Build an unverified NTA desktop package and record its exact inputs.

The manifest is build evidence, never a reference-case result or BRL attest.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CORE_LOCK = ROOT / "crates/nta8800-core/Cargo.lock"
SERVICE_LOCK = ROOT / "crates/nta8800-service/Cargo.lock"
TAURI_LOCK = ROOT / "src-tauri/Cargo.lock"
PACKAGE_LOCK = ROOT / "package-lock.json"
SERVICE_BIN = ROOT / "crates/nta8800-service/target/debug"


def command(*args: str) -> str:
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def run(*args: str) -> None:
    print("+", " ".join(args), flush=True)
    subprocess.run(args, cwd=ROOT, check=True)


def digest(path: Path) -> str:
    sha = hashlib.sha256()
    with path.open("rb") as data:
        for chunk in iter(lambda: data.read(1024 * 1024), b""):
            sha.update(chunk)
    return sha.hexdigest()


def clean_commit() -> str:
    if command("git", "status", "--porcelain", "--untracked-files=normal"):
        raise ValueError("Git worktree must be clean before the NTA dev build")
    return command("git", "rev-parse", "HEAD")


def package_field(path: Path, field: str) -> str:
    return command("dpkg-deb", "--field", str(path), field)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument(
        "--skip-npm-ci",
        action="store_true",
        help="Use existing node_modules for a faster local build; recorded in manifest",
    )
    args = parser.parse_args()
    source_commit = clean_commit()
    package = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))
    core = tomllib.loads((ROOT / "crates/nta8800-core/Cargo.toml").read_text(encoding="utf-8"))
    norm_source = (ROOT / "crates/nta8800-core/src/lib.rs").read_text(encoding="utf-8")
    norm = re.search(r'pub const TARGET_NORM_VERSION: &str = "([^"]+)";', norm_source)
    if norm is None:
        raise ValueError("Could not read target norm version from the Rust kernel")
    expected_deb = ROOT / f"src-tauri/target/debug/bundle/deb/Open Energy Studio_{package['version']}_amd64.deb"
    output = args.output_dir.resolve()
    short = source_commit[:7]
    filenames = {
        "desktop": f"oes-nta8800-dev_{short}_linux-amd64.deb",
        "api": f"oes-nta8800-api_{short}_linux-amd64",
        "mcp": f"oes-nta8800-mcp_{short}_linux-amd64",
        "referenceGate": f"oes-nta8800-reference-gate_{short}_linux-amd64",
    }
    manifest_path = output / f"oes-nta8800-build_{short}_linux-amd64.json"
    for candidate in [manifest_path, *(output / name for name in filenames.values())]:
        if candidate.exists():
            raise FileExistsError(f"Existing build evidence would be overwritten: {candidate}")

    if not args.skip_npm_ci:
        run("npm", "ci")
    run("cargo", "build", "--locked", "--manifest-path", "crates/nta8800-service/Cargo.toml", "--bins")
    run("npm", "run", "tauri", "build", "--", "--debug", "--bundles", "deb")
    if clean_commit() != source_commit:
        raise ValueError("Git source changed during the NTA dev build")
    if not expected_deb.is_file():
        raise FileNotFoundError(expected_deb)
    if package_field(expected_deb, "Version") != package["version"]:
        raise ValueError("Desktop package version differs from package.json")
    if package_field(expected_deb, "Architecture") != "amd64":
        raise ValueError("Desktop package is not Linux amd64")

    output.mkdir(parents=True, exist_ok=True)
    sources = {
        "desktop": expected_deb,
        "api": SERVICE_BIN / "api",
        "mcp": SERVICE_BIN / "mcp",
        "referenceGate": SERVICE_BIN / "reference_gate",
    }
    artifacts = {}
    for name, source in sources.items():
        if not source.is_file():
            raise FileNotFoundError(source)
        target = output / filenames[name]
        shutil.copy2(source, target)
        artifacts[name] = {
            "file": target.name,
            "sha256": digest(target),
            "bytes": target.stat().st_size,
        }

    manifest = {
        "schemaVersion": 1,
        "sourceCommit": source_commit,
        "targetNormVersion": norm.group(1),
        "kernelVersion": core["package"]["version"],
        "desktopPackage": {
            "name": package_field(expected_deb, "Package"),
            "version": package_field(expected_deb, "Version"),
            "architecture": package_field(expected_deb, "Architecture"),
            "profile": "debug",
        },
        "toolchain": {
            "rustc": command("rustc", "--version"),
            "cargo": command("cargo", "--version"),
            "node": command("node", "--version"),
            "npm": command("npm", "--version"),
            "npmInstall": "preexisting_node_modules" if args.skip_npm_ci else "npm_ci",
        },
        "dependencyLocks": {
            "packageLockSha256": digest(PACKAGE_LOCK),
            "coreCargoLockSha256": digest(CORE_LOCK),
            "serviceCargoLockSha256": digest(SERVICE_LOCK),
            "tauriCargoLockSha256": digest(TAURI_LOCK),
        },
        "artifacts": artifacts,
        "referenceVerified": False,
        "attestStatus": "unattested",
    }
    manifest_path.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(manifest_path)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        print(f"NTA dev build failed: {error}", file=sys.stderr)
        sys.exit(1)
