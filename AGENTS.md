# Yaplay release notes

Write user-facing notes under `## [Unreleased]` in `CHANGELOG.md` in the same PR as product changes. Use Added, Changed, Fixed, or Removed headings, concise bullets, and inline links. Docs, tests, and tooling-only changes do not require notes. Keep Conventional Commit PR titles for version selection.

The default branch is `master`. Release Please maintains a rolling PR using the custom Rust strategy in `.github/release`. Do not manually bump versions, create release tags, or edit the rolling release branch: correct notes through ordinary PRs against master. Merging the release PR builds and signs the installers, publishes after assets are complete, then announces on Yapin.

For release tooling changes, run `npm ci --prefix .github/release`, `npm test --prefix .github/release`, and `python3 -m unittest discover -s scripts -p 'test_*.py'` with Python 3.11+. Run `cargo fmt --all --check` and validate workflows with actionlint. Product Rust changes also require Clippy and Cargo tests. See README.md for publishing setup and recovery.
