"""Check version agreement before packaging an approved release tag."""
import json
from pathlib import Path
import re
import sys
import tomllib


def verify(tag, root=Path('.')):
    if not re.fullmatch(r'v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)', tag):
        raise ValueError('Expected a stable vX.Y.Z tag')
    version = tag[1:]
    package = tomllib.loads((root / 'Cargo.toml').read_text())['package']
    locked = tomllib.loads((root / 'Cargo.lock').read_text())['package']
    versions = [entry['version'] for entry in locked if entry['name'] == package['name']]
    manifest = json.loads((root / '.release-please-manifest.json').read_text())
    if package['version'] != version or versions != [version] or manifest['.'] != version:
        raise ValueError('Tag, Cargo.toml, Cargo.lock and release manifest must agree')


if __name__ == '__main__':
    verify(sys.argv[1])
