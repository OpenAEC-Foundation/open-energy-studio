#!/usr/bin/env bash
set -euo pipefail
# Sign the FINAL downloaded bytes of all six installers, including the Linux packages.
# A signed checksum manifest provides detached verification; no RPM/DEB embedded-signature claim.
export GNUPGHOME
GNUPGHOME="$(mktemp -d "$RUNNER_TEMP/oes-gnupg.XXXXXX")"
trap 'gpgconf --kill gpg-agent || true; rm -rf "$GNUPGHOME"' EXIT
chmod 700 "$GNUPGHOME"
fingerprint=13F6D414ADEE4ED4CD09AC60601D86C93C504D29
printf '%s' "$RELEASE_GPG_PRIVATE_KEY" | gpg --batch --import
# An explicit fingerprint prevents choosing a different available or newly generated key.
gpg --batch --list-secret-keys "$fingerprint" >/dev/null
python3 "$GITHUB_WORKSPACE/scripts/signing/manifest.py" create .
printf '%s' "$RELEASE_GPG_PASSPHRASE" | gpg --batch --yes --pinentry-mode loopback --passphrase-fd 0 \
  --local-user "$fingerprint" --armor --detach-sign SHA256SUMS-installers.txt
gpg --armor --export "$fingerprint" > OpenAEC-release-signing.asc
gpg --batch --verify SHA256SUMS-installers.txt.asc SHA256SUMS-installers.txt
python3 "$GITHUB_WORKSPACE/scripts/signing/manifest.py" verify .
