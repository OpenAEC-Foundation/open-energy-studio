"""Small fixture checks for the stored NTA build manifest reader."""

import hashlib
import importlib.util
import io
import json
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("verify-nta-build.py")
SPEC = importlib.util.spec_from_file_location("verify_nta_build", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class BuildManifestTests(unittest.TestCase):
    def test_source_checked_message_only_after_source_verification(self):
        manifest = {"sourceCommit": "a" * 40}
        with patch.object(MODULE.sys, "argv", ["verify-nta-build.py", "build.json", "--source-repo", "."]), \
                patch.object(MODULE, "verify", return_value=manifest), \
                patch.object(MODULE, "verify_source") as verify_source, \
                redirect_stdout(io.StringIO()) as output:
            self.assertEqual(MODULE.main(), 0)
        verify_source.assert_called_once_with(manifest, Path("."))
        self.assertIn("source metadata match commit", output.getvalue())
        self.assertNotIn("was not checked", output.getvalue())

    def test_compare_builds_requires_same_source_and_artifact_bytes(self):
        first = {
            "sourceCommit": "a" * 40,
            "toolchain": {"sourceDateEpoch": 1700000000},
            "artifacts": {key: {"sha256": "b" * 64, "bytes": 10} for key in MODULE.ARTIFACTS},
        }
        second = json.loads(json.dumps(first))
        MODULE.compare_builds(first, second)
        second["artifacts"]["desktop"]["sha256"] = "c" * 64
        with self.assertRaisesRegex(ValueError, "artifact differs: desktop"):
            MODULE.compare_builds(first, second)
        second = json.loads(json.dumps(first))
        second["sourceCommit"] = "d" * 40
        with self.assertRaisesRegex(ValueError, "different source commits"):
            MODULE.compare_builds(first, second)
        second = json.loads(json.dumps(first))
        second["targetNormVersion"] = "different edition"
        with self.assertRaisesRegex(ValueError, "metadata differs: targetNormVersion"):
            MODULE.compare_builds(first, second)
        second = json.loads(json.dumps(first))
        second["toolchain"]["sourceDateEpoch"] += 1
        with self.assertRaisesRegex(ValueError, "metadata differs: sourceDateEpoch"):
            MODULE.compare_builds(first, second)
        second = json.loads(json.dumps(first))
        second["toolchain"]["buildLayout"] = "fixed_tmp_v1"
        with self.assertRaisesRegex(ValueError, "metadata differs: buildLayout"):
            MODULE.compare_builds(first, second)

    def test_source_object_must_be_a_commit(self):
        with patch.object(MODULE.subprocess, "check_output", return_value="tree\n") as check:
            with self.assertRaisesRegex(ValueError, "not a Git commit"):
                MODULE.verify_source({"sourceCommit": "a" * 40}, Path("/unused"))
            check.assert_called_once()

    def test_source_date_epoch_must_match_commit(self):
        manifest = {
            "sourceCommit": "a" * 40,
            "toolchain": {"sourceDateEpoch": 1700000001},
        }
        with patch.object(MODULE.subprocess, "check_output", side_effect=["commit\n", "1700000000\n"]):
            with self.assertRaisesRegex(ValueError, "SOURCE_DATE_EPOCH differs"):
                MODULE.verify_source(manifest, Path("/unused"))

    def test_source_date_epoch_must_be_present(self):
        manifest = {"sourceCommit": "a" * 40, "toolchain": {}}
        with patch.object(MODULE.subprocess, "check_output", return_value="commit\n"):
            with self.assertRaisesRegex(ValueError, "Missing or invalid SOURCE_DATE_EPOCH"):
                MODULE.verify_source(manifest, Path("/unused"))

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
