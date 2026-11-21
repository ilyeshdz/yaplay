import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { detectOS, downloads, downloadUrl } from '../lib/downloads.ts';
import { parseReleases } from '../lib/releases.ts';

test('detects supported desktop OSes without treating mobile devices as desktops', () => {
  for (const [ua, platform, touch, expected] of [
    ['Mozilla/5.0 (Windows NT 10.0; Win64; x64)', 'Win32', 0, 'windows'],
    ['Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)', 'MacIntel', 0, 'mac'],
    ['Mozilla/5.0 (X11; Linux x86_64)', 'Linux x86_64', 0, 'linux'],
    ['Mozilla/5.0 (Linux; Android 14)', 'Linux armv8l', 5, 'mobile'],
    ['Mozilla/5.0 (iPhone; CPU iPhone OS 18_0)', 'iPhone', 5, 'mobile'],
    ['Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15)', 'MacIntel', 5, 'mobile'],
    ['', '', 0, 'unknown'],
  ]) assert.equal(detectOS(ua, platform, touch), expected);
});

test('download filenames match assets produced by the packaging workflow', async () => {
  const workflow = await readFile(new URL('../../.github/workflows/build.yml', import.meta.url), 'utf8');
  for (const build of downloads) {
    assert.equal(downloadUrl(build.asset), `https://github.com/ilyeshdz/yaplay/releases/latest/download/${build.asset}`);
    if (build.id === 'windows') assert.ok(workflow.includes(`output: ${build.asset}`));
    else {
      assert.ok(workflow.includes(`target: ${build.asset.slice(7, -4)}`));
      assert.ok(workflow.includes('Yaplay-${{ matrix.target }}.dmg'));
    }
  }
});

test('release notes include versioned entries only, preserving markdown', () => {
  const entries = parseReleases(`# Changelog\n\n## Unreleased\nNot published\n\n## [2.0.0](https://example.com/compare) (2026-11-21)\n\n### Features\n\n* A [linked change](https://example.com).\n\n## 1.0.0 (2026-10-01)\n\n* First release\n<!-- comment -->\n\n## Before the first release\nOld notes`);
  assert.equal(entries.length, 2);
  assert.equal(entries[0].version, '2.0.0');
  assert.equal(entries[0].date, '2026-11-21');
  assert.ok(entries[0].body.includes('[linked change](https://example.com)'));
  assert.equal(entries[1].body, '* First release');
});
