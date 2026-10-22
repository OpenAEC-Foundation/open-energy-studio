#!/usr/bin/env python3
"""Verify the stored bytes and package metadata of an NTA dev build."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path


ARTIFACTS = {"desktop", "api", "mcp", "referenceGate"}
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
COMMIT = re.compile(r"[0-9a-f]{40}\Z")
LOCKS = {
    "packageLockSha256": "package-lock.json",
    "coreCargoLockSha256": "crates/nta8800-core/Cargo.lock",
    "serviceCargoLockSha256": "crates/nta8800-service/Cargo.lock",
    "tauriCargoLockSha256": "src-tauri/Cargo.lock",
}


def digest(path: Path) -> str:
    sha = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            sha.update(chunk)
    return sha.hexdigest()


def verify(manifest_path: Path) -> dict:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if not isinstance(manifest, dict) or manifest.get("schemaVersion") != 1:
        raise ValueError("Unsupported or missing build manifest schema")
    if not isinstance(manifest.get("sourceCommit"), str) or not COMMIT.fullmatch(manifest["sourceCommit"]):
        raise ValueError("Invalid source commit")
    if manifest.get("referenceVerified") is not False or manifest.get("attestStatus") != "unattested":
        raise ValueError("Build manifest must not claim reference verification or attestation")
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, dict) or set(artifacts) != ARTIFACTS:
        raise ValueError("Expected exactly desktop, API, MCP and reference gate artifacts")

    names = set()
    for key, item in artifacts.items():
        if not isinstance(item, dict):
            raise ValueError(f"Invalid {key} artifact metadata")
        name, expected_sha, expected_bytes = (item.get(field) for field in ("file", "sha256", "bytes"))
        if not isinstance(name, str) or name in ("", ".", "..") or Path(name).name != name:
            raise ValueError(f"Invalid {key} artifact filename")
        if name in names:
            raise ValueError(f"Duplicate artifact filename: {name}")
        names.add(name)
        if not isinstance(expected_sha, str) or not SHA256.fullmatch(expected_sha):
            raise ValueError(f"Invalid {key} SHA-256")
        if type(expected_bytes) is not int or expected_bytes <= 0:
            raise ValueError(f"Invalid {key} byte count")
        path = manifest_path.parent / name
        # A basename alone does not confine a symlink to the manifest directory.
        if path.is_symlink() or not path.is_file() or path.stat().st_size != expected_bytes or digest(path) != expected_sha:
            raise ValueError(f"Artifact differs from manifest: {path}")

    desktop = manifest.get("desktopPackage")
    if not isinstance(desktop, dict) or desktop.get("architecture") != "amd64" or desktop.get("profile") != "debug":
        raise ValueError("Expected a Linux amd64 debug desktop package")
    path = manifest_path.parent / artifacts["desktop"]["file"]
    for field, key in (("Package", "name"), ("Version", "version"), ("Architecture", "architecture")):
        actual = subprocess.check_output(["dpkg-deb", "--field", str(path), field], text=True).strip()
        if actual != desktop.get(key):
            raise ValueError(f"Desktop package {field} differs from manifest")
    return manifest


def verify_source(manifest: dict, repo: Path) -> None:
    commit = manifest["sourceCommit"]
    object_type = subprocess.check_output(
        ["git", "-C", str(repo), "cat-file", "-t", commit],
        text=True,
        stderr=subprocess.DEVNULL,
    ).strip()
    if object_type != "commit":
        raise ValueError("Claimed source object is not a Git commit")
    toolchain = manifest.get("toolchain")
    if not isinstance(toolchain, dict):
        raise ValueError("Missing build toolchain metadata")
    source_date_epoch = toolchain.get("sourceDateEpoch")
    if type(source_date_epoch) is not int:
        raise ValueError("Missing or invalid SOURCE_DATE_EPOCH")
    actual_epoch = int(subprocess.check_output(
        ["git", "-C", str(repo), "show", "-s", "--format=%ct", commit],
        text=True,
    ).strip())
    if source_date_epoch != actual_epoch:
        raise ValueError("SOURCE_DATE_EPOCH differs from source commit time")

    def source_bytes(path: str) -> bytes:
        return subprocess.check_output(
            ["git", "-C", str(repo), "show", f"{commit}:{path}"],
            stderr=subprocess.DEVNULL,
        )

    locks = manifest.get("dependencyLocks")
    if not isinstance(locks, dict) or set(locks) != set(LOCKS):
        raise ValueError("Missing or unexpected source lockfile hashes")
    for key, path in LOCKS.items():
        actual = hashlib.sha256(source_bytes(path)).hexdigest()
        if actual != locks[key]:
            raise ValueError(f"Source lockfile differs from manifest: {path}")

    package = json.loads(source_bytes("package.json"))
    if package["name"] != manifest["desktopPackage"]["name"] or package["version"] != manifest["desktopPackage"]["version"]:
        raise ValueError("Source package metadata differs from manifest")
    core = tomllib.loads(source_bytes("crates/nta8800-core/Cargo.toml").decode("utf-8"))
    if core["package"]["version"] != manifest.get("kernelVersion"):
        raise ValueError("Source kernel version differs from manifest")
    norm_source = source_bytes("crates/nta8800-core/src/lib.rs").decode("utf-8")
    norm = re.search(r'pub const TARGET_NORM_VERSION: &str = "([^"]+)";', norm_source)
    if norm is None or norm.group(1) != manifest.get("targetNormVersion"):
        raise ValueError("Source target norm version differs from manifest")


def compare_builds(first: dict, second: dict) -> None:
    """Require the same source and exact bytes for all four artifacts."""
    if first["sourceCommit"] != second["sourceCommit"]:
        raise ValueError("Builds use different source commits")
    for field in ("targetNormVersion", "kernelVersion", "desktopPackage", "dependencyLocks"):
        if first.get(field) != second.get(field):
            raise ValueError(f"Build metadata differs: {field}")
    first_toolchain = first.get("toolchain")
    second_toolchain = second.get("toolchain")
    if not isinstance(first_toolchain, dict) or not isinstance(second_toolchain, dict):
        raise ValueError("Build metadata differs: sourceDateEpoch")
    first_epoch = first_toolchain.get("sourceDateEpoch")
    second_epoch = second_toolchain.get("sourceDateEpoch")
    if type(first_epoch) is not int or first_epoch != second_epoch:
        raise ValueError("Build metadata differs: sourceDateEpoch")
    if first_toolchain.get("buildLayout") != second_toolchain.get("buildLayout"):
        raise ValueError("Build metadata differs: buildLayout")
    for key in sorted(ARTIFACTS):
        a, b = first["artifacts"][key], second["artifacts"][key]
        if (a["sha256"], a["bytes"]) != (b["sha256"], b["bytes"]):
            raise ValueError(f"Build artifact differs: {key}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--source-repo", type=Path, help="Check the claimed commit and lockfiles against this Git repository")
    parser.add_argument("--compare-with", type=Path, help="Verify a second manifest and require byte-identical artifacts from the same commit")
    args = parser.parse_args()
    manifest = verify(args.manifest)
    if args.source_repo is not None:
        verify_source(manifest, args.source_repo)
    if args.compare_with is not None:
        second = verify(args.compare_with)
        if args.source_repo is not None:
            verify_source(second, args.source_repo)
        compare_builds(manifest, second)
        print("Both manifests verify and all four artifacts are byte-identical for the same commit. Build-host independence, normative reference and attest remain unverified.")
        return 0
    if args.source_repo is not None:
        print(f"Build bytes and source metadata match commit {manifest['sourceCommit']} in the supplied repository. Build provenance, normative reference and attest remain unverified.")
    else:
        print(f"Build bytes verified; claimed source {manifest['sourceCommit']} was not checked against a repository. Normative reference and attest remain unverified.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        print(f"NTA build verification failed: {error}", file=sys.stderr)
        sys.exit(1)
