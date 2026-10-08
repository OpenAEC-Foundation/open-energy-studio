#!/usr/bin/env bash
# Release of the NTA 8800 program (BRL 9501 §5.3, §6.1–6.3).
#
#   scripts/release-nta.sh [--dry-run] [--skip-gate] [--skip-build] [--date yyyy-mm-dd]
#
# A release:
#  1. checks the release notes against KERNEL_VERSION and moves the
#     unreleased items under "Rekenkern <KERNEL_VERSION>" (one commit);
#  2. runs the full gate (scripts/verify-nta.sh), keeping its log and the
#     reference-suite report;
#  3. builds the desktop packages (Tauri, .deb) offline;
#  4. writes the leveringsdocument, a manifest and SHA256SUMS;
#  5. creates the annotated git tag oes-v<program>-kernel-v<kernel>.
# Everything lands in release/<tag>/ (ignored by git). Nothing is pushed:
# publishing the tag and the archive is a separate, manual step.
#
# --dry-run changes nothing in git (no release-notes commit, no tag) and
# accepts a dirty worktree; it writes the archive to release/dry-run-<tag>/.
# --skip-gate and --skip-build are only allowed with --dry-run.
set -euo pipefail
cd "$(dirname "$0")/.."

dry_run=0
skip_gate=0
skip_build=0
release_date="$(date +%F)"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) dry_run=1 ;;
    --skip-gate) skip_gate=1 ;;
    --skip-build) skip_build=1 ;;
    --date)
      [[ $# -ge 2 ]] || { printf -- '--date vraagt jjjj-mm-dd\n' >&2; exit 2; }
      release_date="$2"
      shift
      ;;
    *) printf 'Onbekende optie: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done
if [[ $dry_run -eq 0 && ( $skip_gate -eq 1 || $skip_build -eq 1 ) ]]; then
  printf -- '--skip-gate en --skip-build mogen alleen met --dry-run.\n' >&2
  exit 2
fi

step() { printf '\n==> %s\n' "$1"; }

program_version="$(node -p "require('./package.json').version")"
kernel_version="$(node scripts/nta-kernel-version.mjs version)"
tag="oes-v${program_version}-kernel-v${kernel_version}"
if [[ $dry_run -eq 1 ]]; then
  archive="release/dry-run-${tag}"
else
  archive="release/${tag}"
fi

step "release ${tag} (dry run: ${dry_run})"
if [[ $dry_run -eq 0 ]]; then
  if [[ -n "$(git status --porcelain)" ]]; then
    printf 'De werkmap heeft lokale wijzigingen; een vrijgave vraagt een schone werkmap.\n' >&2
    exit 1
  fi
  if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    printf 'Tag %s bestaat al: verhoog de programma- of rekenkernversie.\n' "$tag" >&2
    exit 1
  fi
  if [[ -e "$archive" ]]; then
    printf 'Archief %s bestaat al.\n' "$archive" >&2
    exit 1
  fi
fi
rm -rf "$archive"
mkdir -p "$archive"

step "release notes against KERNEL_VERSION ${kernel_version}"
node scripts/nta-kernel-version.mjs check
# Unreleased items move under a new section only with a new kernel version;
# a program release without one (UI or report only) leaves them unreleased.
read -r unreleased new_kernel < <(node --input-type=module -e "
  import { parseReleaseNotes, compareVersions } from './scripts/nta-kernel-version.mjs';
  import { readFileSync } from 'node:fs';
  const notes = parseReleaseNotes(readFileSync('docs/nta8800-releasenotes.md', 'utf8'));
  console.log(notes.unreleased.length, compareVersions('$kernel_version', notes.sections[0].version) > 0 ? 1 : 0);
")
if [[ "$unreleased" -gt 0 && "$new_kernel" -eq 0 ]]; then
  printf 'Rekenkern %s is al uitgebracht: %s onuitgebrachte items blijven onder "Onuitgebracht" staan.\n' \
    "$kernel_version" "$unreleased"
  cp docs/nta8800-releasenotes.md "$archive/releasenotes.md"
elif [[ "$unreleased" -gt 0 ]]; then
  if [[ $dry_run -eq 1 ]]; then
    cp docs/nta8800-releasenotes.md "$archive/releasenotes-voor.md"
    node --input-type=module -e "
      import { releaseNotes, dutchDate } from './scripts/nta-kernel-version.mjs';
      import { readFileSync, writeFileSync } from 'node:fs';
      const text = readFileSync('docs/nta8800-releasenotes.md', 'utf8');
      writeFileSync('$archive/releasenotes.md', releaseNotes(text, '$kernel_version', dutchDate('$release_date')));
    "
    printf 'Proef: %s onuitgebrachte items zouden onder Rekenkern %s komen (%s/releasenotes.md).\n' \
      "$unreleased" "$kernel_version" "$archive"
  else
    node scripts/nta-kernel-version.mjs release "$release_date"
    git add docs/nta8800-releasenotes.md
    git commit -q -m "Release NTA kernel ${kernel_version} (program ${program_version})"
    cp docs/nta8800-releasenotes.md "$archive/releasenotes.md"
  fi
else
  cp docs/nta8800-releasenotes.md "$archive/releasenotes.md"
fi
commit="$(git rev-parse HEAD)"
if [[ -n "$(git status --porcelain)" ]]; then
  commit="${commit} (met lokale wijzigingen)"
fi

if [[ $skip_gate -eq 1 ]]; then
  step "gate overgeslagen (proef)"
  gate_status="overgeslagen"
else
  step "gate (log in ${archive}/gate.log)"
  NTA_EVIDENCE_DIR="$PWD/$archive/evidence" scripts/verify-nta.sh 2>&1 | tee "$archive/gate.log"
  gate_status="geslaagd"
fi

packages=()
if [[ $skip_build -eq 1 ]]; then
  step "pakketten overgeslagen (proef)"
else
  step "desktoppakketten (Tauri, offline)"
  CARGO_NET_OFFLINE=true npm run tauri:build -- --bundles deb
  mkdir -p "$archive/pakketten"
  while IFS= read -r -d '' package; do
    cp "$package" "$archive/pakketten/"
    packages+=("$archive/pakketten/$(basename "$package")")
  done < <(find src-tauri/target/release/bundle -maxdepth 2 -type f -name '*.deb' -print0)
  if [[ ${#packages[@]} -eq 0 ]]; then
    printf 'De build leverde geen pakket op.\n' >&2
    exit 1
  fi
fi

step "leveringsdocument"
node scripts/nta-leveringsdocument.mjs --out "$archive/leveringsdocument.md" \
  --commit "$commit" --tag "$tag" --date "$release_date" "${packages[@]}"

step "manifest en SHA256SUMS"
node --input-type=module -e "
  import { readIdentity } from './scripts/nta-leveringsdocument.mjs';
  import { writeFileSync } from 'node:fs';
  const manifest = {
    ...readIdentity('.'),
    tag: '$tag',
    commit: '$commit',
    releaseDate: '$release_date',
    dryRun: $dry_run === 1,
    gate: '$gate_status',
    packagesBuilt: $skip_build === 0,
  };
  writeFileSync('$archive/manifest.json', JSON.stringify(manifest, null, 2) + '\n');
"
(cd "$archive" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)

if [[ $dry_run -eq 1 ]]; then
  step "proef klaar: geen commit, geen tag (${archive})"
else
  step "tag ${tag}"
  git tag -a "$tag" -m "Open Energy Studio ${program_version}, NTA kernel ${kernel_version}"
  printf 'Vrijgave klaar in %s. De tag is lokaal; publiceren gaat apart (git push origin %s).\n' "$archive" "$tag"
fi
