"""Small fixture checks for the stored NTA build manifest reader."""

import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("verify-nta-build.py")
SPEC = importlib.util.spec_from_file_location("verify_nta_build", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class BuildManifestTests(unittest.TestCase):
    def test_source_object_must_be_a_commit(self):
        with patch.object(MODULE.subprocess, "check_output", return_value="tree\n") as check:
            with self.assertRaisesRegex(ValueError, "not a Git commit"):
                MODULE.verify_source({"sourceCommit": "a" * 40}, Path("/unused"))
            check.assert_called_once()

    def test_artifacts_must_be_files_in_manifest_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / "output"
            output.mkdir()
            outside = root / "outside-api"
            outside.write_bytes(b"api")
            artifacts = {}
            for key in sorted(MODULE.ARTIFACTS):
                name = f"{key}.bin"
                data = key.encode("ascii")
                (output / name).write_bytes(data)
                artifacts[key] = {
                    "file": name,
                    "sha256": hashlib.sha256(data).hexdigest(),
                    "bytes": len(data),
                }
            manifest = {
                "schemaVersion": 1,
                "sourceCommit": "a" * 40,
                "referenceVerified": False,
                "attestStatus": "unattested",
                "desktopPackage": {
                    "name": "oes",
                    "version": "1.0.0",
                    "architecture": "amd64",
                    "profile": "debug",
                },
                "artifacts": artifacts,
            }
            manifest_path = output / "build.json"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            with patch.object(MODULE.subprocess, "check_output", side_effect=["oes", "1.0.0", "amd64"]):
                self.assertEqual(MODULE.verify(manifest_path), manifest)

            (output / "api.bin").unlink()
            (output / "api.bin").symlink_to(outside)
            manifest["artifacts"]["api"] = {
                "file": "api.bin",
                "sha256": hashlib.sha256(b"api").hexdigest(),
                "bytes": 3,
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "Artifact differs from manifest"):
                MODULE.verify(manifest_path)


if __name__ == "__main__":
    unittest.main()
