#!/usr/bin/env bash
set -euo pipefail
keychain="$RUNNER_TEMP/oes-signing.keychain-db"
cert="$RUNNER_TEMP/oes-signing.p12"
trap 'rm -f "$cert"' EXIT
printf '%s' "$APPLE_CERTIFICATE" | base64 --decode > "$cert"
security create-keychain -p "$KEYCHAIN_PASSWORD" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$keychain"
security import "$cert" -k "$keychain" -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$keychain" >/dev/null
security list-keychains -d user -s "$keychain" "$HOME/Library/Keychains/login.keychain-db"
security find-identity -v -p codesigning "$keychain"
