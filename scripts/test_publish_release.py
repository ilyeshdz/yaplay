import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("publisher", Path(__file__).with_name("publish-release.py"))
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)


class PublishTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.dist = Path(self.temp.name)
        self.names = [*publisher.PACKAGES, *publisher.ARCHIVES,
                      *(name + ".minisig" for name in publisher.PACKAGES)]
        for name in self.names:
            (self.dist / name).write_bytes(name.encode())
        (self.dist / "SHA256SUMS").write_text("".join(
            f"{publisher.digest(self.dist / name)}  ./{name}\n"
            for name in (*publisher.PACKAGES, *publisher.ARCHIVES)
        ))
        self.existing = {}
        self.uploaded = []

    def run_gh(self, command, **kwargs):
        action = command[2]
        if action == "view":
            return subprocess.CompletedProcess(command, 0, json.dumps({
                "assets": [{"name": name} for name in self.existing]
            }))
        if action == "download":
            name = command[command.index("--pattern") + 1]
            target = Path(command[command.index("--dir") + 1]) / name
            target.write_bytes(self.existing[name])
        elif action == "upload":
            self.uploaded.append(Path(command[4]).name)
        return subprocess.CompletedProcess(command, 0)

    def publish(self):
        with patch.object(publisher.subprocess, "run", side_effect=self.run_gh):
            publisher.publish("v1.3.0", self.dist, "owner/repo")

    def test_checksum_readiness_marker_is_last(self):
        self.publish()
        self.assertEqual(self.uploaded, [*self.names, "SHA256SUMS"])

    def test_retry_skips_identical_assets(self):
        name = publisher.PACKAGES[0]
        self.existing[name] = (self.dist / name).read_bytes()
        self.publish()
        self.assertNotIn(name, self.uploaded)
        self.assertEqual(self.uploaded[-1], "SHA256SUMS")

    def test_changed_asset_aborts_before_any_upload(self):
        self.existing[publisher.PACKAGES[-1]] = b"previous build"
        with self.assertRaisesRegex(ValueError, "Refusing to replace"):
            self.publish()
        self.assertEqual(self.uploaded, [])

    def test_missing_signature_aborts_before_upload(self):
        (self.dist / (publisher.PACKAGES[0] + ".minisig")).unlink()
        with self.assertRaisesRegex(ValueError, "Missing release file"):
            self.publish()
        self.assertEqual(self.uploaded, [])

    def test_stale_checksum_aborts_before_upload(self):
        (self.dist / publisher.PACKAGES[0]).write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "incorrect checksum"):
            self.publish()
        self.assertEqual(self.uploaded, [])


if __name__ == "__main__":
    unittest.main()
