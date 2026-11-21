//! Signed GitHub releases, installed by gpui-updater's native backends.
use std::{fs::File, time::Duration};

use gpui::{prelude::*, *};
use gpui_component::{Disableable, button::Button};
use gpui_updater::{
    EngineConfig, GitHubSource, UpdateSource, UpdateStatus, Updater, Verification, Version,
    VersionBinding,
};

use crate::{data::configuration::Configuration, state::app_state, theme};

const RELEASES_URL: &str = "https://github.com/ilyeshdz/yaplay/releases";
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

pub struct Updates {
    pub updater: Option<Entity<Updater>>,
    unavailable: Option<String>,
    // Keep exclusive ownership until exit, including after installation. A second
    // process must not replace files while the first updater is still running.
    _lock: Option<File>,
}
impl Global for Updates {}

fn asset_name(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("macos", "aarch64") => Some("Yaplay-aarch64-apple-darwin.dmg"),
        ("windows", "x86_64") => Some("yaplay-windows-x86_64.exe"),
        _ => None,
    }
}

// GitHubSource matches substrings. Enforce the exact published filename before
// handing anything to the engine, including when release assets evolve later.
struct ReleaseSource {
    github: GitHubSource,
    asset: &'static str,
}
impl UpdateSource for ReleaseSource {
    fn fetch_latest(&self) -> gpui_updater::Result<gpui_updater::Release> {
        let release = self.github.fetch_latest()?;
        if release.asset.name != self.asset {
            return Err(gpui_updater::Error::Parse(
                "The release does not contain the expected update package.".into(),
            ));
        }
        Ok(release)
    }
}

fn engine_config(key: &str) -> EngineConfig {
    EngineConfig::new(Version::parse(env!("CARGO_PKG_VERSION")).expect("Cargo package version"))
        .verification(Verification::Strict)
        .version_binding(VersionBinding::Required)
        .minisign_public_key(key)
        .max_download_size(512 * 1024 * 1024)
}

fn setup(cx: &mut App) -> anyhow::Result<(Entity<Updater>, File)> {
    anyhow::ensure!(
        !cfg!(debug_assertions),
        "Updates are disabled in development builds."
    );
    let asset = asset_name(std::env::consts::OS, std::env::consts::ARCH)
        .ok_or_else(|| anyhow::anyhow!("Updates are not available for this platform."))?;
    let key = option_env!("YAPLAY_UPDATE_PUBLIC_KEY")
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .ok_or_else(|| anyhow::anyhow!("This build does not have an update verification key. Download the latest release to update."))?;
    let root = gpui_updater::install::current_install_root()?;
    if cfg!(target_os = "macos") {
        anyhow::ensure!(
            root.extension().is_some_and(|ext| ext == "app"),
            "Move Yaplay.app to a writable Applications folder to enable updates."
        );
    }
    let lock_path = Configuration::path()?.with_file_name("updater.lock");
    std::fs::create_dir_all(lock_path.parent().unwrap())?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    fs2::FileExt::try_lock_exclusive(&lock).map_err(|_| {
        anyhow::anyhow!(
            "Another Yaplay instance owns the updater. Close it and reopen Yaplay to update."
        )
    })?;
    gpui_updater::install::remove_leftovers(&root);
    let source = ReleaseSource {
        github: GitHubSource::new("ilyeshdz", "yaplay")
            .asset_patterns(vec![asset.into()])
            .with_checksums("SHA256SUMS")
            .with_minisig(),
        asset,
    };
    Ok((
        cx.new(|cx| Updater::new(source, engine_config(key), cx)),
        lock,
    ))
}

pub fn init(cx: &mut App) {
    let updates = match setup(cx) {
        Ok((updater, lock)) => Updates {
            updater: Some(updater),
            unavailable: None,
            _lock: Some(lock),
        },
        Err(error) => Updates {
            updater: None,
            unavailable: Some(error.to_string()),
            _lock: None,
        },
    };
    if let Some(updater) = &updates.updater {
        cx.observe(updater, |_, cx| cx.refresh_windows()).detach();
    }
    cx.set_global(updates);
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(Duration::from_secs(15))
            .await;
        loop {
            if cx
                .update(|cx| {
                    if app_state().read().unwrap().config.automatic_update_checks {
                        check(cx);
                    }
                })
                .is_err()
            {
                break;
            }
            cx.background_executor().timer(CHECK_INTERVAL).await;
        }
    })
    .detach();
}

// Never erase a pending restart or an available release with a scheduled check.
fn can_check(status: &UpdateStatus) -> bool {
    matches!(
        status,
        UpdateStatus::Idle | UpdateStatus::UpToDate | UpdateStatus::Errored(_)
    )
}

pub fn check(cx: &mut App) {
    if let Some(updater) = cx.global::<Updates>().updater.clone() {
        updater.update(cx, |updater, cx| {
            if can_check(updater.status()) {
                updater.check(cx);
            }
        });
    }
}

pub fn quit(cx: &mut App) {
    let installing = cx
        .try_global::<Updates>()
        .and_then(|updates| updates.updater.as_ref())
        .is_some_and(|updater| matches!(updater.read(cx).status(), UpdateStatus::Installing));
    if installing {
        // The native swap cannot be cancelled safely. Keep a window alive until
        // it finishes instead of terminating the process midway through it.
        crate::windows::open_settings(cx);
    } else {
        cx.quit();
    }
}

pub fn has_update(cx: &App) -> bool {
    cx.global::<Updates>()
        .updater
        .as_ref()
        .is_some_and(|updater| {
            matches!(
                updater.read(cx).status(),
                UpdateStatus::Available(_) | UpdateStatus::Staged(_)
            )
        })
}

fn status_text(status: &UpdateStatus) -> String {
    match status {
        UpdateStatus::Idle => "Updates have not been checked yet.".into(),
        UpdateStatus::Checking => "Checking for updates…".into(),
        UpdateStatus::UpToDate => "Yaplay is up to date.".into(),
        UpdateStatus::Available(version) => format!("Yaplay {version} is available."),
        UpdateStatus::Downloading { downloaded, total } => match total.filter(|total| *total > 0) {
            Some(total) => format!(
                "Downloading update… {}%",
                (downloaded.saturating_mul(100) / total).min(100)
            ),
            None => format!("Downloading update… {} MiB", downloaded / (1024 * 1024)),
        },
        UpdateStatus::Installing => "Verifying and installing update… Keep Yaplay open.".into(),
        UpdateStatus::Staged(version) => {
            format!("Yaplay {version} is installed. Restart when you are ready.")
        }
        UpdateStatus::Errored(error) => {
            format!("Update failed: {error} You can retry or download manually.")
        }
        _ => "Update in progress…".into(),
    }
}

pub fn panel(cx: &App) -> impl IntoElement {
    let updates = cx.global::<Updates>();
    let mut panel = div().flex().flex_col().gap_2().child(
        div()
            .text_lg()
            .child(format!("Updates · {}", env!("CARGO_PKG_VERSION"))),
    );
    if let Some(updater) = &updates.updater {
        let status = updater.read(cx).status().clone();
        panel = panel.child(div().text_sm().child(status_text(&status)));
        let in_call = app_state().read().unwrap().in_call;
        let (label, enabled) = match &status {
            UpdateStatus::Available(_) => ("Download and install", !in_call),
            UpdateStatus::Downloading { .. } => ("Cancel download", true),
            UpdateStatus::Staged(_) => ("Restart Yaplay", !in_call),
            _ => ("Check for updates", can_check(&status)),
        };
        let updater = updater.clone();
        panel = panel.child(
            Button::new("update-action")
                .label(label)
                .disabled(!enabled)
                .on_click(move |_, _, cx| {
                    updater.update(cx, |updater, cx| match updater.status() {
                        UpdateStatus::Available(_) if !app_state().read().unwrap().in_call => {
                            updater.download_and_install(cx)
                        }
                        UpdateStatus::Downloading { .. } => updater.cancel(cx),
                        UpdateStatus::Staged(_) if !app_state().read().unwrap().in_call => {
                            updater.restart(cx)
                        }
                        status if can_check(status) => updater.check(cx),
                        _ => {}
                    });
                }),
        );
        if in_call && matches!(status, UpdateStatus::Available(_) | UpdateStatus::Staged(_)) {
            panel = panel.child(
                div()
                    .text_sm()
                    .child("Finish your call before installing or restarting."),
            );
        }
    } else if let Some(reason) = &updates.unavailable {
        panel = panel.child(div().text_sm().child(reason.clone()));
    }
    panel.child(div().text_sm().text_color(rgb(theme::MUTED))
        .child("Automatic checks contact GitHub every six hours. Downloads and restarts require your action."))
        .child(Button::new("release-notes").label("Release notes and manual download")
            .on_click(|_, _, cx| cx.open_url(RELEASES_URL)))
}

#[cfg(test)]
mod tests {
    use super::{asset_name, can_check, engine_config, status_text};
    use gpui_updater::{UpdateStatus, Verification, Version, VersionBinding};

    #[test]
    fn selects_only_supported_release_targets() {
        assert_eq!(
            asset_name("macos", "aarch64"),
            Some("Yaplay-aarch64-apple-darwin.dmg")
        );
        assert_eq!(asset_name("macos", "x86_64"), None);
        assert_eq!(
            asset_name("windows", "x86_64"),
            Some("yaplay-windows-x86_64.exe")
        );
        assert_eq!(asset_name("windows", "aarch64"), None);
        assert_eq!(asset_name("linux", "x86_64"), None);
    }

    #[test]
    fn checks_cannot_discard_an_update_or_restart() {
        let version = Version::new(2, 0, 0);
        for status in [
            UpdateStatus::Checking,
            UpdateStatus::Available(version.clone()),
            UpdateStatus::Downloading {
                downloaded: 0,
                total: None,
            },
            UpdateStatus::Installing,
            UpdateStatus::Staged(version),
        ] {
            assert!(!can_check(&status));
        }
        assert!(can_check(&UpdateStatus::Idle));
        assert!(can_check(&UpdateStatus::UpToDate));
    }

    #[test]
    fn verification_requires_signature_checksum_and_version_binding() {
        let config = engine_config("test public key");
        assert_eq!(config.verification, Verification::Strict);
        assert_eq!(config.version_binding, VersionBinding::Required);
        assert_eq!(config.minisign_public_keys, ["test public key"]);
    }

    #[test]
    fn progress_handles_unknown_empty_and_overreported_lengths() {
        for total in [None, Some(0), Some(1)] {
            let text = status_text(&UpdateStatus::Downloading {
                downloaded: u64::MAX,
                total,
            });
            assert!(text.starts_with("Downloading update…"));
        }
    }

    #[derive(Clone)]
    struct FixtureSource(gpui_updater::Release);
    impl gpui_updater::UpdateSource for FixtureSource {
        fn fetch_latest(&self) -> gpui_updater::Result<gpui_updater::Release> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn incomplete_releases_are_not_offered_and_older_versions_are_ignored() {
        use gpui_updater::{Asset, Release, UpdateEngine};
        let mut config = engine_config("test key");
        config.current_version = Version::new(1, 9, 0);
        let release = Release::new(
            Version::new(1, 10, 0),
            Asset::new("yaplay.exe", "https://example.invalid/yaplay.exe", 10),
        );
        let check = |release| UpdateEngine::new(FixtureSource(release), config.clone()).check();
        assert!(check(release.clone()).is_err());
        assert!(check(release.clone().with_sha256("00".repeat(32))).is_err());
        assert!(check(release.clone().with_minisign_signature("signature")).is_err());
        let mut older = release.clone();
        older.version = Version::new(1, 8, 0);
        assert!(check(older).unwrap().is_none());
        assert!(
            check(
                release
                    .with_sha256("00".repeat(32))
                    .with_minisign_signature("signature")
            )
            .unwrap()
            .is_some()
        );
    }

    // Uses only temporary installations and ephemeral keys. The cached download
    // still passes the engine's strict signature, checksum, and payload checks.
    #[test]
    #[cfg(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    ))]
    fn signed_native_package_installs_to_an_isolated_location() {
        use gpui_updater::{Asset, Release, UpdateEngine, verify};
        use gpui_updater_release::{MinisignKey, MinisignOptions};
        use std::fs;
        #[cfg(target_os = "macos")]
        use std::path::Path;

        let temporary = tempfile::tempdir().unwrap();
        let directory = temporary.path();
        let cache = directory.join("cache");
        fs::create_dir(&cache).unwrap();
        let name = asset_name(std::env::consts::OS, std::env::consts::ARCH).unwrap();
        let package = cache.join(name);

        #[cfg(target_os = "macos")]
        fn bundle(path: &Path, version: &str) {
            fs::create_dir_all(path.join("Contents/MacOS")).unwrap();
            fs::copy("/usr/bin/true", path.join("Contents/MacOS/yaplay")).unwrap();
            fs::write(path.join("Contents/Info.plist"), format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>io.github.ilyeshdz.yaplay.test</string>
<key>CFBundleExecutable</key><string>yaplay</string>
<key>CFBundleShortVersionString</key><string>{version}</string>
<key>CFBundleVersion</key><string>{version}</string>
</dict></plist>"#)).unwrap();
        }

        #[cfg(target_os = "macos")]
        let root = {
            let root = directory.join("Yaplay test é.app");
            bundle(&root, "1.0.0");
            let stage = directory.join("dmg-stage");
            bundle(&stage.join("Yaplay.app"), "2.0.0");
            let result = std::process::Command::new("hdiutil")
                .args(["create", "-ov", "-format", "UDZO", "-srcfolder"])
                .arg(&stage)
                .arg(&package)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            root
        };
        #[cfg(target_os = "windows")]
        let root = {
            let root = directory.join("Renamed Yaplay é.exe");
            let exe = std::env::current_exe().unwrap();
            fs::copy(&exe, &root).unwrap();
            fs::copy(&exe, &package).unwrap();
            root
        };
        let key = MinisignKey::generate().unwrap();
        let public = key.public_key_base64();
        let signature = key
            .sign_file(&package, &MinisignOptions::version("2.0.0"))
            .unwrap();
        let comment = verify::verify_minisign_file(&package, &[&public], &signature).unwrap();
        assert_eq!(verify::signed_version(&comment), Some("2.0.0"));
        let hash = verify::sha256_hex(&package).unwrap();
        let release = Release::new(
            Version::new(2, 0, 0),
            Asset::new(
                name,
                format!("https://example.invalid/{name}"),
                fs::metadata(&package).unwrap().len(),
            ),
        )
        .with_sha256(hash)
        .with_minisign_signature(signature.clone());
        let mut config = engine_config(&public)
            .install_root(&root)
            .download_dir(&cache);
        config.current_version = Version::new(1, 0, 0);
        let engine = UpdateEngine::new(FixtureSource(release), config);
        let release = engine.check().unwrap().unwrap();
        let artifact = engine.download(&release, |_, _| {}).unwrap();
        let installed = engine.install(&artifact).unwrap();
        assert_eq!(installed.restart_path.as_deref(), Some(root.as_path()));
        assert!(root.exists());
        #[cfg(target_os = "macos")]
        assert!(
            fs::read_to_string(root.join("Contents/Info.plist"))
                .unwrap()
                .contains("2.0.0")
        );
        #[cfg(target_os = "windows")]
        assert_eq!(
            verify::sha256_hex(&root).unwrap(),
            verify::sha256_hex(&package).unwrap()
        );
        fs::write(&package, b"tampered package").unwrap();
        assert!(verify::verify_minisign_file(&package, &[&public], &signature).is_err());
    }
}
