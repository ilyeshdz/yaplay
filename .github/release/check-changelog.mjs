import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { hasNotes, section } from "./manual-changelog.mjs";

const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();
const base = process.env.CHANGELOG_BASE;
if (!base) throw new Error("CHANGELOG_BASE is required");
const pending = section(readFileSync("CHANGELOG.md", "utf8"), "Unreleased").body;
const files = git("diff", "--name-only", `${base}...HEAD`).split("\n");
if (files.some((file) => file.startsWith("src/") || file === "examples/yaplay-client.mjs")) {
  const previous = git("show", `${git("merge-base", base, "HEAD")}:CHANGELOG.md`);
  if (!hasNotes(pending) || pending === section(previous, "Unreleased").body) {
    throw new Error("Add hand-written user-facing notes under ## [Unreleased] in CHANGELOG.md.");
  }
}
