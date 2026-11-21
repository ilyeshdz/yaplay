//! The OS registration is the source of truth; launching Yaplay never enables it.
use anyhow::{Context, Result};
use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};
use std::path::Path;

fn registration(executable: &Path) -> Result<AutoLaunch> {
    let executable = executable
        .to_str()
        .context("The executable path is not valid Unicode")?;
    let mut builder = AutoLaunchBuilder::new();
    builder
        .set_app_name("Yaplay")
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser);

    #[cfg(target_os = "macos")]
    {
        // auto-launch interpolates LaunchAgent arguments into XML without escaping.
        // Use env as the fixed executable so the actual path can be an escaped
        // argument (the crate checks app_path on disk before writing the plist).
        // env execs this absolute path, preserving bundle and standalone launches.
        builder
            .set_app_path("/usr/bin/env")
            .set_args(&[xml_text(executable)])
            .set_macos_launch_mode(auto_launch::MacOSLaunchMode::LaunchAgent)
            .set_bundle_identifiers(&["io.github.ilyeshdz.yaplay"]);
    }
    #[cfg(not(target_os = "macos"))]
    builder.set_app_path(executable);

    builder
        .build()
        .context("Cannot configure startup registration")
}

fn current_registration() -> Result<AutoLaunch> {
    registration(&std::env::current_exe().context("Cannot locate Yaplay")?)
}

pub fn is_enabled() -> Result<bool> {
    current_registration()?
        .is_enabled()
        .context("Cannot read startup registration")
}

pub fn set_enabled(enabled: bool) -> Result<bool> {
    let registration = current_registration()?;
    if enabled {
        registration.enable().context("Cannot enable startup")?;
    } else {
        registration.disable().context("Cannot disable startup")?;
    }
    registration
        .is_enabled()
        .context("Cannot verify startup registration")
}

#[cfg(target_os = "macos")]
fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn launch_agent_can_be_enabled_and_disabled() {
        const CHILD_HOME: &str = "YAPLAY_TEST_AUTOSTART_HOME";
        if let Some(home) = std::env::var_os(CHILD_HOME) {
            assert_eq!(std::env::var_os("HOME"), Some(home.clone()));
            assert!(!is_enabled().unwrap());
            assert!(set_enabled(true).unwrap());
            assert!(set_enabled(true).unwrap());
            let plist = std::path::PathBuf::from(home).join("Library/LaunchAgents/Yaplay.plist");
            let contents = std::fs::read_to_string(&plist).unwrap();
            assert!(contents.contains("<key>RunAtLoad</key>"));
            assert!(contents.contains("io.github.ilyeshdz.yaplay"));
            assert!(!set_enabled(false).unwrap());
            assert!(!set_enabled(false).unwrap());
            assert!(!plist.exists());
            return;
        }

        // Isolate HOME in a child process, never touching real login settings or
        // changing the environment of concurrently running tests.
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir(home.path().join("Library")).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "autostart::tests::launch_agent_can_be_enabled_and_disabled",
            ])
            .env("HOME", home.path())
            .env(CHILD_HOME, home.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn registration_targets_the_running_executable_without_shell_parsing() {
        let path = std::env::current_exe().unwrap();
        let registration = registration(&path).unwrap();
        assert_eq!(registration.get_app_name(), "Yaplay");
        #[cfg(target_os = "macos")]
        {
            assert_eq!(registration.get_app_path(), "/usr/bin/env");
            assert_eq!(registration.get_args(), &[xml_text(path.to_str().unwrap())]);
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(registration.get_app_path(), path.to_str().unwrap());
            assert!(registration.get_args().is_empty());
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn launch_agent_path_escapes_xml_but_preserves_spaces_and_quotes() {
        let registration =
            registration(Path::new("/Users/A & B/<apps>/Yaplay's app/yaplay")).unwrap();
        assert_eq!(
            registration.get_args(),
            &["/Users/A &amp; B/&lt;apps&gt;/Yaplay's app/yaplay"]
        );
    }
}
