import { execFileSync } from "node:child_process";
import { GitHub, Manifest, registerReleaseType } from "release-please";
import { ManualChangelog } from "./manual-changelog.mjs";

registerReleaseType("manual-rust", (options) => new ManualChangelog(options));
const [owner, repo] = process.env.GITHUB_REPOSITORY.split("/");
const github = await GitHub.create({ owner, repo, token: process.env.GH_TOKEN });
const manifest = await Manifest.fromManifest(github, "master");
const dispatch = (workflow, ref, inputs = []) =>
  execFileSync("gh", ["workflow", "run", workflow, "--ref", ref, ...inputs], { stdio: "inherit" });
for (const release of await manifest.createReleases()) {
  if (release) dispatch("build.yml", "master", ["-f", `tag=${release.tagName}`]);
}
for (const pr of await manifest.createPullRequests()) {
  if (pr) dispatch("checks.yml", pr.headBranchName);
}
