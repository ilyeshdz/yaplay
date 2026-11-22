#!/usr/bin/env python3
"""Publish complete updater assets without replacing bytes trusted by clients."""
import argparse
import hashlib
import json
import os
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
        ["gh", "release", "view", tag, "--repo", repository, "--json", "assets,isDraft,isPrerelease"],
        check=True, capture_output=True, text=True,
    )
    release = json.loads(result.stdout)
    if release["isPrerelease"]:
        raise ValueError("Only stable releases can be published")
    existing = {asset["name"] for asset in release["assets"]}
    notes = release_notes(tag) if release["isDraft"] else None
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

    if not release["isDraft"]:
        return False
    latest = subprocess.run(
        ["gh", "release", "view", "--repo", repository, "--json", "tagName"],
        check=True, capture_output=True, text=True,
    )
    latest_tag = json.loads(latest.stdout)["tagName"]
    if not re.fullmatch(r"v\d+\.\d+\.\d+", latest_tag):
        raise ValueError("Expected a stable latest release tag")
    version = lambda value: tuple(map(int, value[1:].split(".")))
    make_latest = str(version(tag) >= version(latest_tag)).lower()
    with tempfile.TemporaryDirectory(prefix="yaplay-notes-") as temporary:
        notes_path = Path(temporary) / "notes.md"
        notes_path.write_text(notes, encoding="utf-8")
        subprocess.run(
            ["gh", "release", "edit", tag, "--repo", repository,
             "--draft=false", f"--latest={make_latest}", "--notes-file", str(notes_path)],
            check=True,
        )
    return True


def release_notes(tag, path=Path("CHANGELOG.md")):
    text = path.read_text(encoding="utf-8")
    sections = re.findall(
        rf"^## \[{re.escape(tag[1:])}\][^\n]*\n(.*?)(?=^## |\Z)",
        text, re.MULTILINE | re.DOTALL,
    )
    if len(sections) != 1 or not re.search(r"^[-*] \S", sections[0], re.MULTILINE):
        raise ValueError(f"Missing hand-written notes for {tag}")
    return sections[0].strip()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--repo", required=True)
    args = parser.parse_args()
    published = publish(args.tag, args.dist, args.repo)
    if output := os.environ.get("GITHUB_OUTPUT"):
        with open(output, "a", encoding="utf-8") as stream:
            stream.write(f"published={str(published).lower()}\n")
