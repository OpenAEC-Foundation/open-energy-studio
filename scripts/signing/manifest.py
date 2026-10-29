#!/usr/bin/env python3
"""Require all six delivery formats and check exact downloaded bytes."""
import hashlib
from pathlib import Path
import re
import sys

FORMATS = ('.deb', '.rpm', '.AppImage', '.dmg', '.exe', '.msi')
MANIFEST = 'SHA256SUMS-installers.txt'


def installers(root):
    files = sorted(p for p in root.iterdir() if p.is_file() and p.suffix in FORMATS)
    for suffix in FORMATS:
        if sum(p.suffix == suffix for p in files) != 1:
            raise ValueError(f'Expected exactly one installer ending in {suffix}')
    return files


def create(root):
    rows = []
    for path in installers(root):
        if not re.fullmatch(r'[A-Za-z0-9_.+-]+', path.name):
            raise ValueError('Installer name must be normalized as on GitHub: ' + path.name)
        rows.append(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + path.name)
    (root / MANIFEST).write_text('\n'.join(rows) + '\n')


def verify(root):
    expected = {p.name for p in installers(root)}
    seen = set()
    for line in (root / MANIFEST).read_text().splitlines():
        match = re.fullmatch(r'([0-9a-f]{64})  ([A-Za-z0-9_.+-]+)', line)
        if not match:
            raise ValueError('Malformed checksum record')
        digest, name = match.groups()
        if name in seen or name not in expected:
            raise ValueError('Unexpected or duplicate checksum record: ' + name)
        seen.add(name)
        if hashlib.sha256((root / name).read_bytes()).hexdigest() != digest:
            raise ValueError('Checksum mismatch: ' + name)
    if seen != expected:
        raise ValueError('Incomplete checksum manifest')


if __name__ == '__main__':
    {'create': create, 'verify': verify}[sys.argv[1]](Path(sys.argv[2]))
