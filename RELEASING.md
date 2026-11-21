# Releasing Yaplay

Release Please manages one release for the Rust application. The website and
example wrapper ship from this repository but do not have independent versions.

## Repository setup

- Under **Settings → Actions → General → Workflow permissions**, enable
  **Allow GitHub Actions to create and approve pull requests**. The workflows
  grant their own scoped permissions; the repository's default token permission
  can stay read-only. No personal access token is needed.
- Enable squash merging and use the PR title as the squash commit title. Feature
  PR titles must follow Conventional Commits. Preserve any `BREAKING CHANGE:`
  footer in the squash commit body.
- If branch protection is used, require **Required checks** from the Checks
  workflow. Human-authored PRs also get a **Conventional PR title** check; title
  edits run only that workflow, not code checks. Website validation also runs when its files change; do not require
  a path-filtered workflow on every PR. Remove any old required release-build
  checks, since those no longer run on PRs.
- GitHub Pages still needs **Settings → Pages → GitHub Actions**. Website
  deployment is independent of desktop releases.

## Normal release cycle

1. Open a feature/fix PR with a conventional title, such as
   `feat: add voice activity indicators` or `fix: clear the roster after leaving`.
2. Pass the PR checks and squash-merge into `master`.
3. The **Release Please** workflow creates or updates a release PR. It includes
   `Cargo.toml`, the root package entry in `Cargo.lock`, `CHANGELOG.md`, and
   `.release-please-manifest.json`. Its changelog covers commits since the last
   release, including documentation, tests, and CI changes.
4. Review the accumulated release notes and version, and wait for **Required
   checks**. Keep the Release Please title and `autorelease:*` labels intact.
5. Merge the release PR. Release Please creates a `vX.Y.Z` tag and a GitHub
   Release using the changelog as release notes.
6. The same workflow calls **Release assets** to build Windows x86_64, macOS
   Apple Silicon, and macOS Intel packages from the resolved tag commit. It
   signs the DMGs and Windows executable, then uploads packages, signatures,
   and finally `SHA256SUMS` after all three platform jobs succeed.

The GitHub Release is visible before packaging finishes. Allow the release run
to finish before announcing downloads. No desktop release package is built on an
ordinary PR or push; native macOS and Windows runners perform debug lint/tests for
Rust changes because the GUI and installer depend on native libraries. These checks still
compile dependencies and exercise temporary installer fixtures, but do not produce
optimized release binaries or distributable installers.
Both updater backends compile in pull-request checks.

## Versioning and changelog

The first stable release is explicitly selected with a one-time
`Release-As: 1.0.0` commit footer. The Rust strategy defaults to `0.1.0` and does
not honor `initial-version`; use `Release-As` for an intentional version override.
The initially empty manifest lets Release Please collect the repository's full
history. The release PR bumps `Cargo.toml`, `Cargo.lock`, and the manifest together.
Do not prefill the manifest with a version that has not actually been released.

After bootstrap:

- `fix:` and `perf:` produce a patch release.
- `feat:` produces a minor release.
- Breaking changes (`feat!:` or a `BREAKING CHANGE:` footer) produce a minor bump
  before `1.0.0`, and a major bump after `1.0.0`.
- Documentation, CI, test, refactoring, build, and maintenance commits appear in
  the next release's notes, but normally do not trigger a release by themselves.

Do not manually maintain an `Unreleased` section: Release Please does not move
that section into versioned entries. The open release PR is the rolling view of
changes since the latest release. `CHANGELOG.md` on `master` contains released
entries plus the clearly labeled initial development record. Review generated
notes in the release PR before merging. Correct missing or misleading entries
through conventional source commits or Release Please's supported commit-message
overrides, rather than relying on edits the bot may regenerate.

## Why there is no tag-push trigger

GitHub suppresses new workflow runs for most events created by `GITHUB_TOKEN`.
A bot-created tag or release will therefore not start a separate `push: tags`
or `release: published` workflow.

`release-please.yml` calls the reusable packaging workflow directly when
`release_created` is true. It also explicitly dispatches `checks.yml` on the
release PR's branch whenever the bot creates or updates it; workflow dispatches
are allowed with `GITHUB_TOKEN`, and checks attach to that branch's commit.
This avoids an extra PAT and avoids missing CI on bot PRs.

## Recovery

- **Release PR checks failed:** fix the underlying issue, merge the fix into
  `master`, and let Release Please refresh the release PR. If only CI was
  interrupted, rerun the failed checks or dispatch **Checks** on the release
  branch.
- **Release Please failed before publishing:** rerun its failed job, or dispatch
  **Release Please** on `master`. It reconciles existing release PRs and tags.
- **Packaging failed before uploading:** dispatch **Release assets** with the
  existing tag. Do not rerun only Release Please and expect `release_created`
  to become true again.
- **Upload failed:** rerun the failed upload job using the original build
  artifacts. Identical uploaded files are skipped; missing files are uploaded.
  A full rebuild may produce different bytes and will be rejected. Publish a new
  version if the original artifacts are no longer available. Never delete or
  replace files referenced by a published updater release.
- The packaging workflow rejects missing releases and tags whose version differs
  from `Cargo.toml`. It checks out the tag's immutable commit rather than the
  branch selected for manual dispatch. Published assets are immutable: retries
  compare existing files before uploading anything and reject changed bytes.
- An arbitrary pushed tag does not publish or package anything. Use the release
  PR flow. The manual workflow is for existing releases, not a shortcut for
  creating releases.

## Update signing

The application uses pinned `gpui-updater` 0.4.4 with strict SHA-256 and minisign
verification and required signed version binding. Its native backends install
our existing DMG and portable EXE formats. No custom updater executable, feed
server, or Sparkle framework is needed. Rust 1.98 or newer is required by the GPUI
adapter.

Before the first release with updates, create a dedicated release key on a trusted
machine (keep a secure backup):

```sh
cargo install gpui-updater-release --version 0.4.4 --features cli --locked
umask 077
gpui-updater-release keys generate-minisign --out /secure/location/yaplay-updater
```

Set repository variable **YAPLAY_UPDATE_PUBLIC_KEY** to the public key base64 line printed
by the command and repository Actions secret **UPDATER_SIGNING_KEY** to the full
secret-key file contents. The tool generates an unencrypted signing key for CI;
protect it like a release credential and never commit it. The reusable release
workflow explicitly receives the secret. Signing verifies that the secret matches
the public key and fails if they differ.

The public key is embedded at build time using `YAPLAY_UPDATE_PUBLIC_KEY`; it
cannot be changed by a downloaded release. Local builds without it remain usable
and provide manual downloads. Debug builds disable the updater even if a key is
set. A release workflow refuses to package without signing configuration.

Release assets retain their current names. Each DMG and EXE gets a `.minisig`
sidecar whose trusted comment binds its release version. SHA256SUMS is published
last so a partially uploaded release cannot pass strict verification. The GitHub
release can appear earlier; checks may report an update error until packaging
finishes and will retry on the next scheduled or manual check.

Key rotation requires first shipping a client that trusts both the old and new
public keys, signed with the old key, before switching release signing to the
new key. Do not simply replace the repository variable: existing clients still
trust the old key. Apple notarization and Windows Authenticode are separate from
update signatures and are not enabled by this change.

The library validates downloads and handles normal replacement failures. It does
not retain a health-checked rollback version after successful installation or
provide power-loss recovery. Before rollout, exercise a signed upgrade between
two test versions on macOS Intel/Apple Silicon and Windows, including a renamed
EXE, spaces in paths, an unwritable location, cancellation, and a failed download.
Use isolated test keys and installations; never replace production assets for
this test. The first updater-enabled build still requires a manual installation.

## Local validation

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
node --test examples/yaplay-client.test.mjs
python3 -m unittest discover -s scripts -p 'test_*.py'
```

For website changes, run the commands documented in `README.md`. Workflow syntax
is checked in CI with actionlint. Native packaging itself is validated during an
actual release; do not create a dummy release just to test this configuration.
