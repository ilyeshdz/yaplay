import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { ManualChangelog, section, promote } from "./manual-changelog.mjs";
import { parseConventionalCommits } from "release-please/build/src/commit.js";
import { Version } from "release-please/build/src/version.js";
import { TagName } from "release-please/build/src/util/tag-name.js";
import { releaseMessage } from "./announce-release.mjs";

const notes = "### Changed\n\n- Hand-written release notes.";
const log = `# Changelog\n\n## [Unreleased]\n\n${notes}\n\n## [1.4.0] (2026-11-21)\n\n- Previous release.\n`;
const root = new URL("../../", import.meta.url);

test("Rust release proposals preserve notes and update both Cargo versions", async () => {
  const github = {
    repository: { owner: "ilyeshdz", repo: "yaplay" },
    getFileContentsOnBranch: async (path) => ({
      parsedContent: path === "CHANGELOG.md" ? log : readFileSync(new URL(path, root), "utf8"),
    }),
  };
  const strategy = new ManualChangelog({
    github,
    targetBranch: "master",
    includeComponentInTag: false,
  });
  const pr = await strategy.buildReleasePullRequest(
    parseConventionalCommits([
      { sha: "abc", message: "feat: generated wording", files: ["src/main.rs"] },
    ]),
    { tag: new TagName(Version.parse("1.4.0")), sha: "previous", notes: "" },
  );
  assert.equal(pr.version.toString(), "1.5.0");
  assert.ok(pr.body.toString().includes(notes));
  assert.ok(!pr.body.toString().includes("generated wording"));
  for (const path of ["Cargo.toml", "Cargo.lock"]) {
    const original = readFileSync(new URL(path, root), "utf8");
    const updated = pr.updates
      .find((update) => update.path === path)
      .updater.updateContent(original);
    assert.ok(updated.includes("1.5.0"));
  }
  const updated = pr.updates
    .find((update) => update.path === "CHANGELOG.md")
    .updater.updateContent(log);
  assert.equal(section(updated, "Unreleased").body, "");
  assert.equal(section(updated, "1.5.0").body, notes);
  // Keep the website's existing dated-heading format.
  assert.match(section(updated, "1.5.0").heading, /^## \[1\.5\.0\] \(\d{4}-\d{2}-\d{2}\)$/);
  github.getFileContentsOnBranch = async (_path, ref) => {
    assert.equal(ref, "approved");
    return { parsedContent: updated };
  };
  const release = await strategy.buildRelease({
    title: pr.title.toString(),
    body: pr.body.toString().replace(notes, "Edited PR prose"),
    headBranchName: pr.headRefName,
    sha: "approved",
  });
  assert.ok(release.notes.endsWith(notes));
});

test("empty or ambiguous changelogs cannot be promoted", () => {
  assert.throws(() => promote("# Changelog\n\n## [Unreleased]\n", "## [1.5.0]"));
  assert.throws(() => section(log + "\n## [Unreleased]\n", "Unreleased"));
});

test("Yaplay announcements retain notes and ping only its configured role", () => {
  const release = {
    tag_name: "v1.5.0",
    html_url: "https://github.com/ilyeshdz/yaplay/releases/tag/v1.5.0",
    body: notes,
    draft: false,
    prerelease: false,
  };
  const role = "c8be2daa-4ac7-402a-b119-2336394a0b24";
  const message = releaseMessage(release, role);
  assert.ok(message.content.includes("Yaplay v1.5.0"));
  assert.ok(message.content.includes(notes));
  assert.deepEqual(message.allowed_mentions, { parse: [], roles: [role] });
  assert.throws(() => releaseMessage({ ...release, draft: true }, role));
});
