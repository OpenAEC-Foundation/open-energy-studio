#!/usr/bin/env python3
"""Fail before building a release; print missing NAMES only, never values."""
import os
import json
from pathlib import Path

REQUIRED = '''AZURE_CLIENT_ID AZURE_CLIENT_SECRET AZURE_TENANT_ID AZURE_ENDPOINT
AZURE_TRUSTED_SIGNING_ACCOUNT_NAME AZURE_CERTIFICATE_PROFILE_NAME WINDOWS_SIGNING_SUBJECT
APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY KEYCHAIN_PASSWORD
APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID RELEASE_GPG_PRIVATE_KEY'''.split()

if __name__ == '__main__':
    missing = [name for name in REQUIRED if not os.environ.get(name, '').strip()]
    if missing:
        raise SystemExit('Release signing configuration missing: ' + ', '.join(missing))
    if not os.environ['APPLE_SIGNING_IDENTITY'].startswith('Developer ID Application:'):
        raise SystemExit('APPLE_SIGNING_IDENTITY must be a Developer ID Application identity.')
    config = json.loads((Path(__file__).resolve().parents[2] / 'src-tauri/tauri.conf.json').read_text())
    if config.get('bundle', {}).get('createUpdaterArtifacts') or 'updater' in config.get('plugins', {}):
        raise SystemExit('Updater configured: add explicit updater signing and verification before releasing.')
    print('Required signing configuration is present; cryptographic checks still follow.')
