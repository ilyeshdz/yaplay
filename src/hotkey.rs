use crate::{
    components::settings::Settings,
    state::{ControlAction, app_state},
    windows,
};
use anyhow::{Result, anyhow, ensure};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};
use gpui::{App, Global, Keystroke, WeakEntity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Controls,
    Mute,
    Deafen,
}

impl Action {
    pub const ALL: [Self; 3] = [Self::Controls, Self::Mute, Self::Deafen];
    pub fn label(self) -> &'static str {
        match self {
            Self::Controls => "Show or hide controls",
            Self::Mute => "Mute or unmute",
            Self::Deafen => "Deafen or undeafen",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shortcuts {
    pub controls: String,
    pub mute: Option<String>,
    pub deafen: Option<String>,
}
impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            controls: if cfg!(target_os = "macos") {
                "Cmd+Shift+Y"
            } else {
                "Ctrl+Shift+Y"
            }
            .into(),
            mute: None,
            deafen: None,
        }
    }
}
impl Shortcuts {
    pub fn get(&self, action: Action) -> Option<&str> {
        match action {
            Action::Controls => Some(&self.controls),
            Action::Mute => self.mute.as_deref(),
            Action::Deafen => self.deafen.as_deref(),
        }
    }
    pub fn set(&mut self, action: Action, value: String) {
        match action {
            Action::Controls => self.controls = value,
            Action::Mute => self.mute = Some(value),
            Action::Deafen => self.deafen = Some(value),
        }
    }
    fn bindings(&self) -> Result<Vec<(Action, HotKey)>> {
        let mut bindings = Vec::new();
        for action in Action::ALL {
            if let Some(value) = self.get(action) {
                let key = parse(value)?;
                ensure!(
                    !bindings.iter().any(|(_, k)| *k == key),
                    "{value} is already assigned to another Yaplay action."
                );
                bindings.push((action, key));
            }
        }
        Ok(bindings)
    }
}

fn parse(value: &str) -> Result<HotKey> {
    let key: HotKey = value.parse()?;
    ensure!(
        !key.mods.is_empty(),
        "Include a modifier such as Ctrl or Shift."
    );
    for reserved in ["CmdOrCtrl+Q", "CmdOrCtrl+Comma"] {
        ensure!(
            key != reserved.parse()?,
            "{value} is already used by Yaplay."
        );
    }
    Ok(key)
}

pub fn recorded(stroke: &Keystroke) -> String {
    let m = stroke.modifiers;
    let mut parts = Vec::new();
    if m.platform {
        parts.push(if cfg!(target_os = "macos") {
            "Cmd"
        } else {
            "Super"
        });
    }
    if m.control {
        parts.push("Ctrl");
    }
    if m.alt {
        parts.push("Alt");
    }
    if m.shift {
        parts.push("Shift");
    }
    let key = stroke.key.to_ascii_uppercase();
    parts.push(&key);
    parts.join("+")
}

pub struct Registry {
    manager: GlobalHotKeyManager,
    active: Vec<(Action, HotKey)>,
    pub recorder: Option<WeakEntity<Settings>>,
}
impl Global for Registry {}

impl Registry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            manager: GlobalHotKeyManager::new()?,
            active: vec![],
            recorder: None,
        })
    }
    fn register(&self, key: HotKey) -> Result<()> {
        self.manager.register(key).map_err(|err| match err {
            global_hotkey::Error::AlreadyRegistered(_) => {
                anyhow!("{key} is already in use. Choose another shortcut.")
            }
            other => anyhow!("Cannot register {key}: {other}"),
        })
    }
    pub fn check(&self, draft: &Shortcuts, action: Action) -> Result<()> {
        let bindings = draft.bindings()?;
        if let Some((_, key)) = bindings.iter().find(|(which, _)| *which == action)
            && !self.active.iter().any(|(_, k)| k == key)
        {
            self.register(*key)?;
            self.manager.unregister(*key)?;
        }
        Ok(())
    }

    /// Keep existing shortcuts registered until all replacements and the settings save succeed.
    pub fn apply(&mut self, draft: &Shortcuts, save: impl FnOnce() -> Result<()>) -> Result<()> {
        let bindings = draft.bindings()?;
        let mut acquired = Vec::new();
        let result = (|| {
            for (_, key) in &bindings {
                if !self.active.iter().any(|(_, k)| k == key) {
                    self.register(*key)?;
                    acquired.push(*key);
                }
            }
            save()
        })();
        // Once saved, dispatch must follow the new bindings even if the OS fails
        // to release an obsolete key. Never dispatch abandoned registrations.
        let obsolete = if result.is_ok() {
            std::mem::replace(&mut self.active, bindings)
                .into_iter()
                .map(|(_, key)| key)
                .filter(|key| !self.active.iter().any(|(_, k)| k == key))
                .collect()
        } else {
            acquired
        };
        for key in obsolete {
            if let Err(error) = self.manager.unregister(key) {
                eprintln!("Cannot release {key}: {error}");
            }
        }
        result
    }
}

pub fn init(cx: &mut App, shortcuts: &Shortcuts) -> Result<()> {
    let mut registry = Registry::new()?;
    let mut errors = Vec::new();
    for action in Action::ALL {
        if let Some(value) = shortcuts.get(action) {
            match parse(value).and_then(|key| registry.register(key).map(|()| key)) {
                Ok(key) => registry.active.push((action, key)),
                Err(error) => errors.push(format!("{}: {error}", action.label())),
            }
        }
    }
    cx.set_global(registry);
    ensure!(errors.is_empty(), "{}", errors.join(". "));
    Ok(())
}

pub fn ensure_registry(cx: &mut App) -> Result<()> {
    if !cx.has_global::<Registry>() {
        cx.set_global(Registry::new()?);
    }
    Ok(())
}

pub fn dispatch(cx: &mut App) {
    while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
        if event.state != HotKeyState::Pressed {
            continue;
        }
        let Some(registry) = cx.try_global::<Registry>() else {
            continue;
        };
        let Some(&(action, key)) = registry.active.iter().find(|(_, key)| key.id() == event.id)
        else {
            continue;
        };
        if let Some(editor) = registry.recorder.clone()
            && editor
                .update(cx, |settings, cx| {
                    settings.accept_shortcut(key.to_string(), cx)
                })
                .is_ok()
        {
            continue;
        }
        match action {
            Action::Controls => windows::toggle_controls(cx),
            Action::Mute | Action::Deafen => {
                let shared = app_state();
                let state = shared.read().unwrap();
                state.request_control(match action {
                    Action::Mute => ControlAction::SetMute {
                        muted: !state.mic_muted,
                    },
                    _ => ControlAction::SetDeafen {
                        deafened: !state.deafened,
                    },
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_and_reserved_chords_are_rejected_before_registration() {
        let mut draft = Shortcuts {
            controls: "Ctrl+Shift+M".into(),
            mute: Some("shift+control+KeyM".into()),
            deafen: None,
        };
        assert!(draft.bindings().is_err());
        draft.mute = Some("CmdOrCtrl+Q".into());
        assert!(draft.bindings().is_err());
        draft.mute = Some("M".into());
        assert!(draft.bindings().is_err());
        draft.mute = None;
        assert_eq!(draft.bindings().unwrap().len(), 1);
    }
}
