use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuration {
    pub appearance: super::appearance::Appearance,
    pub selected_monitor_id: u32,
    pub notifications_enabled: bool,
    pub automatic_update_checks: bool,
    pub websocket_port: u16,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            appearance: Default::default(),
            selected_monitor_id: 0,
            notifications_enabled: true,
            automatic_update_checks: true,
            websocket_port: 8080,
        }
    }
}

impl Configuration {
    pub(crate) fn path() -> io::Result<PathBuf> {
        let base = if cfg!(target_os = "windows") {
            std::env::var_os("APPDATA").map(PathBuf::from)
        } else if cfg!(target_os = "macos") {
            std::env::var_os("HOME")
                .map(|home| PathBuf::from(home).join("Library/Application Support"))
        } else {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
                })
        };
        base.map(|base| base.join("yaplay/settings.json"))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Cannot locate the settings directory",
                )
            })
    }

    pub fn load() -> io::Result<Self> {
        let mut config: Self = serde_json::from_slice(&std::fs::read(Self::path()?)?)?;
        if config.websocket_port == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid connection port",
            ));
        }
        config.appearance.normalize();
        Ok(config)
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path()?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, serde_json::to_vec_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_round_trip_and_missing_fields_keep_defaults() {
        let config = Configuration {
            selected_monitor_id: 42,
            notifications_enabled: false,
            websocket_port: 9090,
            ..Default::default()
        };
        let restored: Configuration =
            serde_json::from_slice(&serde_json::to_vec(&config).unwrap()).unwrap();
        assert_eq!(restored, config);
        assert_eq!(
            serde_json::from_str::<Configuration>("{}").unwrap(),
            Configuration::default()
        );
    }
}
