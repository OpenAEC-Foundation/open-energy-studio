import contextlib
import io
import os
from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import patch

from manifest import create, verify, FORMATS, MANIFEST
from credentials import REQUIRED


class ManifestTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        for suffix in FORMATS:
            (self.root / ('Open.Energy.Studio_1' + suffix)).write_bytes(b'installer')
        create(self.root)

    def test_roundtrip(self):
        verify(self.root)

    def test_tampering(self):
        (self.root / 'Open.Energy.Studio_1.deb').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'Checksum mismatch'):
            verify(self.root)

    def test_missing_platform(self):
        (self.root / 'Open.Energy.Studio_1.dmg').unlink()
        with self.assertRaisesRegex(ValueError, 'exactly one'):
            create(self.root)

    def test_extra_installer(self):
        (self.root / 'old.exe').write_bytes(b'old')
        with self.assertRaisesRegex(ValueError, 'exactly one'):
            verify(self.root)

    def test_duplicate_manifest_record(self):
        p = self.root / MANIFEST
        p.write_text(p.read_text() + p.read_text().splitlines()[0] + '\n')
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            verify(self.root)

    def test_omitted_record(self):
        p = self.root / MANIFEST
        p.write_text('\n'.join(p.read_text().splitlines()[1:]) + '\n')
        with self.assertRaisesRegex(ValueError, 'Incomplete'):
            verify(self.root)

    def test_path_traversal(self):
        (self.root / MANIFEST).write_text('a' * 64 + '  ../escape.deb\n')
        with self.assertRaisesRegex(ValueError, 'Malformed'):
            verify(self.root)

    def test_names_with_spaces_rejected(self):
        (self.root / 'Open.Energy.Studio_1.deb').rename(self.root / 'Open Energy Studio.deb')
        with self.assertRaisesRegex(ValueError, 'normalized'):
            create(self.root)

    def test_missing_credentials_only_print_names(self):
        with patch.dict(os.environ, {'AZURE_CLIENT_SECRET': 'do-not-print-this'}, clear=True):
            with self.assertRaises(SystemExit) as error:
                runpy.run_path(str(Path(__file__).with_name('credentials.py')), run_name='__main__')
        self.assertIn('APPLE_CERTIFICATE', str(error.exception))
        self.assertNotIn('do-not-print-this', str(error.exception))

    def test_ad_hoc_identity_rejected(self):
        values = dict.fromkeys(REQUIRED, 'configured')
        values['APPLE_SIGNING_IDENTITY'] = '-'
        with patch.dict(os.environ, values, clear=True):
            with self.assertRaisesRegex(SystemExit, 'Developer ID Application'):
                runpy.run_path(str(Path(__file__).with_name('credentials.py')), run_name='__main__')

    def test_configured_credentials_are_not_acceptance(self):
        values = dict.fromkeys(REQUIRED, 'configured')
        values['APPLE_SIGNING_IDENTITY'] = 'Developer ID Application: Test'
        with patch.dict(os.environ, values, clear=True), contextlib.redirect_stdout(io.StringIO()):
            runpy.run_path(str(Path(__file__).with_name('credentials.py')), run_name='__main__')

    def test_updater_requires_an_explicit_signing_chain(self):
        values = dict.fromkeys(REQUIRED, 'configured')
        values['APPLE_SIGNING_IDENTITY'] = 'Developer ID Application: Test'
        for config in ['{"bundle":{"createUpdaterArtifacts":true}}', '{"plugins":{"updater":{}}}']:
            with self.subTest(config=config), patch.dict(os.environ, values, clear=True):
                with patch.object(Path, 'read_text', return_value=config):
                    with self.assertRaisesRegex(SystemExit, 'Updater configured'):
                        runpy.run_path(str(Path(__file__).with_name('credentials.py')), run_name='__main__')


if __name__ == '__main__':
    unittest.main()
