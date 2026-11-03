use std::sync::OnceLock;

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

static HOTKEY_ID: OnceLock<u32> = OnceLock::new();

pub fn register() -> Result<(), global_hotkey::Error> {
    let modifiers = if cfg!(target_os = "macos") {
        Modifiers::SUPER
    } else {
        Modifiers::CONTROL
    };
    let hotkey = HotKey::new(Some(modifiers | Modifiers::SHIFT), Code::KeyY);

    let manager = GlobalHotKeyManager::new()?;
    manager.register(hotkey)?;

    // The manager must stay alive for the lifetime of the process. On Windows it
    // contains a raw pointer, so it cannot be stored in a global static.
    std::mem::forget(manager);

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
