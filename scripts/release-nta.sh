#!/usr/bin/env bash
# Release of the NTA 8800 program (BRL 9501 §5.3, §6.1–6.3).
#
#   scripts/release-nta.sh [--dry-run] [--skip-gate] [--skip-build] [--date yyyy-mm-dd]
#
# A release:
#  1. checks the release notes against KERNEL_VERSION and, against the notes
#     of the previous release tag, that no released section was changed; it
#     prepares the released notes (unreleased items under "Rekenkern
#     <KERNEL_VERSION>") in the archive, without touching the worktree yet;
#  2. runs the full gate (scripts/verify-nta.sh), keeping its log and the
#     reference-suite report;
#  3. builds the desktop packages (Tauri, .deb) offline;
#  4. exports the user manual as one HTML file (and a PDF when chromium or
#     wkhtmltopdf is installed), stamped with both versions (BRL 9501 §4.4);
#  5. only then commits the released notes (one commit), so a failing gate or
#     build leaves no release commit behind;
#  6. writes the leveringsdocument, a manifest and SHA256SUMS;
#  7. creates the annotated git tag oes-v<program>-kernel-v<kernel>.
# When a release fails before its tag, its archive is moved aside to
# release/<tag>.mislukt-<time>/ so the release can simply be run again.
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
tagged=0
on_exit() {
  local status=$?
  if [[ $status -ne 0 && $dry_run -eq 0 && $tagged -eq 0 && -d "$archive" ]]; then
    local aside
    aside="${archive}.mislukt-$(date +%Y%m%d-%H%M%S)"
    mv "$archive" "$aside"
    printf 'Vrijgave mislukt; het archief staat apart in %s.\n' "$aside" >&2
  fi
}
trap on_exit EXIT

step "release notes against KERNEL_VERSION ${kernel_version}"
node scripts/nta-kernel-version.mjs check
# Released sections are history: compare them with the previous release tag.
previous_tag="$(git tag -l 'oes-v*-kernel-v*' --sort=-creatordate | head -n 1)"
if [[ -n "$previous_tag" ]] && git cat-file -e "${previous_tag}:docs/nta8800-releasenotes.md" 2>/dev/null; then
  git show "${previous_tag}:docs/nta8800-releasenotes.md" > "$archive/releasenotes-vorige-vrijgave.md"
  node scripts/nta-kernel-version.mjs check-released "$archive/releasenotes-vorige-vrijgave.md"
fi
# Unreleased items move under a new section only with a new kernel version;
# a program release without one (UI or report only) leaves them unreleased.
read -r unreleased new_kernel < <(node --input-type=module -e "
  import { parseReleaseNotes, compareVersions } from './scripts/nta-kernel-version.mjs';
  import { readFileSync } from 'node:fs';
  const notes = parseReleaseNotes(readFileSync('docs/nta8800-releasenotes.md', 'utf8'));
  console.log(notes.unreleased.length, compareVersions('$kernel_version', notes.sections[0].version) > 0 ? 1 : 0);
")
notes_to_commit=0
if [[ "$unreleased" -gt 0 && "$new_kernel" -eq 0 ]]; then
  printf 'Rekenkern %s is al uitgebracht: %s onuitgebrachte items blijven onder "Onuitgebracht" staan.\n' \
    "$kernel_version" "$unreleased"
  cp docs/nta8800-releasenotes.md "$archive/releasenotes.md"
elif [[ "$unreleased" -gt 0 ]]; then
  # The released notes are prepared in the archive; the worktree changes only
  # after the gate and the build (step 5).
  cp docs/nta8800-releasenotes.md "$archive/releasenotes-voor.md"
  node --input-type=module -e "
    import { releaseNotes, dutchDate } from './scripts/nta-kernel-version.mjs';
    import { readFileSync, writeFileSync } from 'node:fs';
    const text = readFileSync('docs/nta8800-releasenotes.md', 'utf8');
    writeFileSync('$archive/releasenotes.md', releaseNotes(text, '$kernel_version', dutchDate('$release_date')));
  "
  if [[ $dry_run -eq 1 ]]; then
    printf 'Proef: %s onuitgebrachte items zouden onder Rekenkern %s komen (%s/releasenotes.md).\n' \
      "$unreleased" "$kernel_version" "$archive"
  else
    notes_to_commit=1
  fi
else
  cp docs/nta8800-releasenotes.md "$archive/releasenotes.md"
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

step "handleiding (BRL 9501 §4.4)"
node scripts/nta-manual.mjs check
mkdir -p "$archive/handleiding"
manual_html="$archive/handleiding/handleiding-nta8800-${program_version}.html"
node scripts/nta-manual.mjs html --out "$manual_html" --program "$program_version" --date "$release_date"
manual_files=("$manual_html")
manual_pdf="${manual_html%.html}.pdf"
pdf_tool=""
for candidate in chromium chromium-browser google-chrome wkhtmltopdf; do
  if command -v "$candidate" >/dev/null 2>&1; then pdf_tool="$candidate"; break; fi
done
if [[ "$pdf_tool" == wkhtmltopdf ]]; then
  wkhtmltopdf --quiet "$manual_html" "$manual_pdf" && manual_files+=("$manual_pdf")
elif [[ -n "$pdf_tool" ]]; then
  "$pdf_tool" --headless --disable-gpu --no-pdf-header-footer --print-to-pdf="$PWD/$manual_pdf" "file://$PWD/$manual_html" >/dev/null 2>&1 \
    && manual_files+=("$manual_pdf")
fi
if [[ ${#manual_files[@]} -eq 1 ]]; then
  printf 'Geen PDF-omzetter (chromium of wkhtmltopdf) gevonden: de handleiding gaat mee als HTML.\n'
fi
manual_args=()
for file in "${manual_files[@]}"; do manual_args+=(--manual "$file"); done

if [[ $notes_to_commit -eq 1 ]]; then
  step "releasenotes: onuitgebrachte items onder Rekenkern ${kernel_version} (commit)"
  cp "$archive/releasenotes.md" docs/nta8800-releasenotes.md
  node scripts/nta-kernel-version.mjs check
  git add docs/nta8800-releasenotes.md
  git commit -q -m "Release NTA kernel ${kernel_version} (program ${program_version})"
fi
commit="$(git rev-parse HEAD)"
if [[ -n "$(git status --porcelain)" ]]; then
  commit="${commit} (met lokale wijzigingen)"
fi

step "leveringsdocument"
node scripts/nta-leveringsdocument.mjs --out "$archive/leveringsdocument.md" \
  --commit "$commit" --tag "$tag" --date "$release_date" "${manual_args[@]}" "${packages[@]}"

step "manifest en SHA256SUMS"
node --input-type=module -e "
  import { readIdentity, sha256File } from './scripts/nta-leveringsdocument.mjs';
  import { writeFileSync } from 'node:fs';
  const manifest = {
    ...readIdentity('.'),
    tag: '$tag',
    commit: '$commit',
    releaseDate: '$release_date',
    dryRun: $dry_run === 1,
    gate: '$gate_status',
    packagesBuilt: $skip_build === 0,
    manual: '${manual_files[*]}'.split(' ').filter(Boolean).map((path) => ({
      file: path.slice('$archive/'.length), sha256: sha256File(path),
    })),
  };
  writeFileSync('$archive/manifest.json', JSON.stringify(manifest, null, 2) + '\n');
"
(cd "$archive" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)

if [[ $dry_run -eq 1 ]]; then
  step "proef klaar: geen commit, geen tag (${archive})"
else
  step "tag ${tag}"
  git tag -a "$tag" -m "Open Energy Studio ${program_version}, NTA kernel ${kernel_version}"
  tagged=1
  printf 'Vrijgave klaar in %s. De tag is lokaal; publiceren gaat apart (git push origin %s).\n' "$archive" "$tag"
fi
