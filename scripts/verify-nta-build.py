#!/usr/bin/env python3
"""Verify the stored bytes and package metadata of an NTA dev build."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path


ARTIFACTS = {"desktop", "api", "mcp", "referenceGate"}
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
COMMIT = re.compile(r"[0-9a-f]{40}\Z")


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
        if not path.is_file() or path.stat().st_size != expected_bytes or digest(path) != expected_sha:
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    manifest = verify(args.manifest)
    print(f"Build bytes verified; claimed source {manifest['sourceCommit']} is not authenticated. Normative reference and attest remain unverified.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"NTA build verification failed: {error}", file=sys.stderr)
        sys.exit(1)
