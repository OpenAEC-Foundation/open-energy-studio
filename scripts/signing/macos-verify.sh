#!/usr/bin/env bash
set -euo pipefail
# Verify the app actually carried by the downloaded DMG, not a build-tree copy.
dmg="$1"
verify_identity() {
  local identity
  identity="$(codesign -dv --verbose=4 "$1" 2>&1)"
  grep -q '^Authority=Developer ID Application:' <<< "$identity"
  grep -qxF "TeamIdentifier=$APPLE_TEAM_ID" <<< "$identity"
}
codesign --verify --strict --verbose=2 "$dmg"
verify_identity "$dmg"
xcrun stapler validate "$dmg"
mountpoint="$(mktemp -d "$RUNNER_TEMP/oes-dmg.XXXXXX")"
trap 'hdiutil detach "$mountpoint" >/dev/null || true; rmdir "$mountpoint" || true' EXIT
hdiutil attach "$dmg" -readonly -nobrowse -mountpoint "$mountpoint"
shopt -s nullglob
apps=("$mountpoint/"*.app)
[[ ${#apps[@]} == 1 ]]
app="${apps[0]}"
codesign --verify --deep --strict --verbose=2 "$app"
verify_identity "$app"
xcrun stapler validate "$app"
spctl --assess --type execute --verbose=4 "$app"
