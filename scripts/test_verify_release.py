import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('verifier', Path(__file__).with_name('verify-release.py'))
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


class VerifyTests(unittest.TestCase):
    def test_all_version_files_must_match_the_stable_tag(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'Cargo.toml').write_text('[package]\nname = "yaplay"\nversion = "1.4.0"\n')
            (root / 'Cargo.lock').write_text('[[package]]\nname = "yaplay"\nversion = "1.4.0"\n')
            (root / '.release-please-manifest.json').write_text(json.dumps({'.': '1.4.0'}))
            verifier.verify('v1.4.0', root)
            for tag in ['v1.4.1', 'v1.4.0-beta.1', 'v01.4.0']:
                with self.assertRaises(ValueError):
                    verifier.verify(tag, root)
            (root / 'Cargo.lock').write_text('[[package]]\nname = "yaplay"\nversion = "1.3.0"\n')
            with self.assertRaisesRegex(ValueError, 'must agree'):
                verifier.verify('v1.4.0', root)
