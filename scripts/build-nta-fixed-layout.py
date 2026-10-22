#!/usr/bin/env python3
"""Build NTA debug evidence from one fixed absolute checkout and Cargo layout.

This isolates path-sensitive Rust outputs across hosts. It does not establish
normative reference verification or attestation.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


SOURCE_REPO = Path(__file__).resolve().parent.parent
BUILD_ROOT = Path("/tmp/oes-nta8800-evidence")


def command(*args: str) -> str:
    return subprocess.check_output(args, cwd=SOURCE_REPO, text=True).strip()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    output = args.output_dir.resolve()
    if output == BUILD_ROOT or BUILD_ROOT in output.parents:
        raise ValueError("Output directory must be outside the temporary build layout")
    if command("git", "status", "--porcelain", "--untracked-files=normal"):
        raise ValueError("Git worktree must be clean before the NTA dev build")
    source_commit = command("git", "rev-parse", "HEAD")
    # Exclusive creation also prevents two builds using the same fixed paths.
    BUILD_ROOT.mkdir()
    try:
        checkout = BUILD_ROOT / "source"
        cargo_home = BUILD_ROOT / "cargo"
        rustup_home = BUILD_ROOT / "rustup"
        cargo_home.mkdir()
        original_rustup = Path(os.environ.get("RUSTUP_HOME", Path.home() / ".rustup")).resolve()
        if not original_rustup.is_dir():
            raise FileNotFoundError(original_rustup)
        rustup_home.symlink_to(original_rustup, target_is_directory=True)
        subprocess.run(
            ["git", "clone", "--local", "--no-hardlinks", "--quiet", "--no-checkout", str(SOURCE_REPO), str(checkout)],
            check=True,
        )
        subprocess.run(["git", "-C", str(checkout), "checkout", "--quiet", "--detach", source_commit], check=True)
        env = os.environ.copy()
        env.update({
            "CARGO_HOME": str(cargo_home),
            "RUSTUP_HOME": str(rustup_home),
            "CARGO_INCREMENTAL": "0",
            "NTA_EVIDENCE_FIXED_LAYOUT": "1",
        })
        subprocess.run(
            [sys.executable, str(checkout / "scripts/build-nta-dev.py"), "--output-dir", str(output)],
            cwd=checkout, env=env, check=True,
        )
        if command("git", "rev-parse", "HEAD") != source_commit:
            raise ValueError("Source repository moved during the NTA dev build")
    finally:
        shutil.rmtree(BUILD_ROOT)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        print(f"Fixed-layout NTA build failed: {error}", file=sys.stderr)
        sys.exit(1)
