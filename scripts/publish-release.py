#!/usr/bin/env python3
"""Publish complete updater assets without replacing bytes trusted by clients."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile

PACKAGES = (
    "yaplay-windows-x86_64.exe",
    "Yaplay-aarch64-apple-darwin.dmg",
)
ARCHIVES = (
    "Yaplay-aarch64-apple-darwin.app.zip",
)


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(block)
    return result.hexdigest()


def publish(tag, directory, repository):
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        raise ValueError("Expected vMAJOR.MINOR.PATCH")
    # SHA256SUMS is uploaded last. Strict clients cannot offer an incomplete
    # release. Existing assets are immutable, including on workflow retries.
    names = [*PACKAGES, *ARCHIVES, *(name + ".minisig" for name in PACKAGES), "SHA256SUMS"]
    for name in names:
        if not (directory / name).is_file():
            raise ValueError(f"Missing release file: {name}")
    sums = (directory / "SHA256SUMS").read_text()
    for name in (*PACKAGES, *ARCHIVES):
        if f"{digest(directory / name)}  ./{name}\n" not in sums:
            raise ValueError(f"Missing or incorrect checksum: {name}")
    result = subprocess.run(
        ["gh", "release", "view", tag, "--repo", repository, "--json", "assets"],
        check=True, capture_output=True, text=True,
    )
    existing = {asset["name"] for asset in json.loads(result.stdout)["assets"]}
    # Compare all existing files before uploading any missing file. A rebuild
    # is not necessarily reproducible (DMG creation, timestamps, signatures).
    with tempfile.TemporaryDirectory(prefix="yaplay-release-") as temporary:
        for name in names:
            if name not in existing:
                continue
            subprocess.run(
                ["gh", "release", "download", tag, "--repo", repository,
                 "--pattern", name, "--dir", temporary], check=True,
            )
            if digest(Path(temporary) / name) != digest(directory / name):
                raise ValueError(f"Refusing to replace published asset {name}; publish a new version")
    for name in names:
        if name not in existing:
            subprocess.run(
                ["gh", "release", "upload", tag, str(directory / name), "--repo", repository],
                check=True,
            )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--repo", required=True)
    args = parser.parse_args()
    publish(args.tag, args.dist, args.repo)
