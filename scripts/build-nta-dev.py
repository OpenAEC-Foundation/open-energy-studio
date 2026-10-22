#!/usr/bin/env python3
"""Build an unverified NTA desktop package and record its exact inputs.

The manifest is build evidence, never a reference-case result or BRL attest.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
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


def run(*args: str, env: dict[str, str] | None = None) -> None:
    print("+", " ".join(args), flush=True)
    subprocess.run(args, cwd=ROOT, check=True, env=env)


def rust_build_env() -> dict[str, str]:
    """Give debug information stable source paths on different build hosts."""
    env = os.environ.copy()
    if env.get("RUSTFLAGS") or env.get("CARGO_ENCODED_RUSTFLAGS"):
        raise ValueError("NTA build requires unset RUSTFLAGS and CARGO_ENCODED_RUSTFLAGS")
    # Keep a deliberate symlink spelling: the fixed-layout runner gives Rustup
    # the same path on each host, although its actual toolchain lives elsewhere.
    cargo_home = Path(os.path.abspath(env.get("CARGO_HOME", Path.home() / ".cargo")))
    rustup_home = Path(os.path.abspath(env.get("RUSTUP_HOME", Path.home() / ".rustup")))
    remaps = (
        (ROOT, "/workspace"),
        (cargo_home, "/cargo"),
        (rustup_home, "/rustup"),
    )
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(
        f"--remap-path-prefix={source}={target}" for source, target in remaps
    )
    return env


def digest(path: Path) -> str:
    sha = hashlib.sha256()
    with path.open("rb") as data:
        for chunk in iter(lambda: data.read(1024 * 1024), b""):
            sha.update(chunk)
    return sha.hexdigest()


def copy_exclusive(source: Path, target: Path) -> None:
    """Never replace existing build evidence, including a dangling symlink."""
    with source.open("rb") as input_file, target.open("xb") as output_file:
        shutil.copyfileobj(input_file, output_file)


def clean_commit() -> str:
    if command("git", "status", "--porcelain", "--untracked-files=normal"):
        raise ValueError("Git worktree must be clean before the NTA dev build")
    return command("git", "rev-parse", "HEAD")


def package_field(path: Path, field: str) -> str:
    return command("dpkg-deb", "--field", str(path), field)


def normalize_deb(path: Path, source_date_epoch: int) -> None:
    """Repack the Tauri bundle with stable ownership, timestamps and compression."""
    with tempfile.TemporaryDirectory(prefix="oes-nta-deb-", dir=path.parent) as temporary:
        staging = Path(temporary)
        tree = staging / "tree"
        normalized = staging / "normalized.deb"
        env = {**os.environ, "SOURCE_DATE_EPOCH": str(source_date_epoch)}
        run("dpkg-deb", "-R", str(path), str(tree))
        # Tauri may enumerate files in a different order when writing this
        # control file, even when every packaged payload byte is identical.
        md5sums = tree / "DEBIAN/md5sums"
        if md5sums.exists():
            lines = md5sums.read_text(encoding="utf-8").splitlines()
            if any("  " not in line for line in lines):
                raise ValueError("Malformed deb md5sums")
            paths = [line.split("  ", 1)[1] for line in lines]
            if len(paths) != len(set(paths)):
                raise ValueError("Duplicate path in deb md5sums")
            md5sums.write_text(
                "\n".join(line for _, line in sorted(zip(paths, lines))) + "\n",
                encoding="utf-8",
            )
        # SOURCE_DATE_EPOCH only caps newer mtimes; normalize older inputs too.
        entries = list(tree.rglob("*"))
        for entry in entries:
            os.utime(entry, (source_date_epoch, source_date_epoch), follow_symlinks=False)
        os.utime(tree, (source_date_epoch, source_date_epoch))
        run(
            "dpkg-deb", "--build", "--root-owner-group", "-Zgzip", "-z1",
            str(tree), str(normalized), env=env,
        )
        normalized.replace(path)


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
    source_date_epoch = int(command("git", "show", "-s", "--format=%ct", source_commit))
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
        if candidate.exists() or candidate.is_symlink():
            raise FileExistsError(f"Existing build evidence would be overwritten: {candidate}")

    if not args.skip_npm_ci:
        run("npm", "ci")
    build_env = rust_build_env()
    run("cargo", "build", "--locked", "--manifest-path", "crates/nta8800-service/Cargo.toml", "--bins", env=build_env)
    # dpkg-deb clamps archive and tar entry times to this reproducible bound.
    run(
        "npm", "run", "tauri", "build", "--", "--debug", "--bundles", "deb",
        env={**build_env, "SOURCE_DATE_EPOCH": str(source_date_epoch)},
    )
    if clean_commit() != source_commit:
        raise ValueError("Git source changed during the NTA dev build")
    if not expected_deb.is_file():
        raise FileNotFoundError(expected_deb)
    normalize_deb(expected_deb, source_date_epoch)
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
        copy_exclusive(source, target)
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
            "sourceDateEpoch": source_date_epoch,
            "debugPathRemap": "workspace:/workspace,cargo:/cargo,rustup:/rustup",
            "buildLayout": "fixed_tmp_v1" if os.environ.get("NTA_EVIDENCE_FIXED_LAYOUT") == "1" else "caller_checkout",
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
    with manifest_path.open("x", encoding="utf-8") as manifest_file:
        manifest_file.write(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(manifest_path)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        print(f"NTA dev build failed: {error}", file=sys.stderr)
        sys.exit(1)
