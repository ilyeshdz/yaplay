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
        self.draft = True
        self.latest = 'v1.2.0'
        self.finalized = []

    def run_gh(self, command, **kwargs):
        action = command[2]
        if action == "view" and "tagName" in command:
            return subprocess.CompletedProcess(command, 0, json.dumps({"tagName": self.latest}))
        if action == "edit":
            self.finalized.append(command)
        if action == "view":
            return subprocess.CompletedProcess(command, 0, json.dumps({
                "assets": [{"name": name} for name in self.existing],
                "isDraft": self.draft, "isPrerelease": False
            }))
        if action == "download":
            name = command[command.index("--pattern") + 1]
            target = Path(command[command.index("--dir") + 1]) / name
            target.write_bytes(self.existing[name])
        elif action == "upload":
            self.uploaded.append(Path(command[4]).name)
        return subprocess.CompletedProcess(command, 0)

    def publish(self):
        with patch.object(publisher.subprocess, "run", side_effect=self.run_gh), patch.object(publisher, "release_notes", return_value="### Fixed\n\n- A human note."):
            return publisher.publish("v1.3.0", self.dist, "owner/repo")

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

    def test_publishes_only_after_all_assets_and_does_not_announce_public_retry(self):
        self.assertTrue(self.publish())
        self.assertEqual(len(self.finalized), 1)
        self.assertIn('--latest=true', self.finalized[0])
        self.draft = False
        self.finalized.clear()
        self.assertFalse(self.publish())
        self.assertEqual(self.finalized, [])

    def test_old_release_cannot_replace_latest(self):
        self.latest = 'v1.4.0'
        self.publish()
        self.assertIn('--latest=false', self.finalized[0])

    def test_reads_versioned_human_notes_and_rejects_empty_notes(self):
        path = self.dist / 'CHANGELOG.md'
        path.write_text('# Changelog\n\n## [Unreleased]\n\n## [1.3.0] (2026-11-22)\n\n### Fixed\n\n- Human note.\n\n## [1.2.0]\n\n- Old note.')
        self.assertEqual(publisher.release_notes('v1.3.0', path), '### Fixed\n\n- Human note.')
        with self.assertRaisesRegex(ValueError, 'Missing hand-written'):
            publisher.release_notes('v9.0.0', path)


if __name__ == "__main__":
    unittest.main()
