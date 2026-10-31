use std::sync::OnceLock;

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

static HOTKEY_ID: OnceLock<u32> = OnceLock::new();
static MANAGER: OnceLock<GlobalHotKeyManager> = OnceLock::new();

pub fn register() -> Result<(), global_hotkey::Error> {
    let modifiers = if cfg!(target_os = "macos") {
        Modifiers::SUPER
    } else {
        Modifiers::CONTROL
    };
    let hotkey = HotKey::new(Some(modifiers | Modifiers::SHIFT), Code::KeyY);

    let manager = GlobalHotKeyManager::new()?;
    manager.register(hotkey)?;

    let _ = MANAGER.set(manager);
    let _ = HOTKEY_ID.set(hotkey.id());
    Ok(())
}

pub fn take_toggle_press() -> bool {
    let Some(hotkey_id) = HOTKEY_ID.get() else {
        return false;
    };

    let mut toggled = false;
    while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
        if event.id == *hotkey_id && event.state == HotKeyState::Pressed {
            toggled = !toggled;
        }
    }
    toggled
}
