#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

pub mod autostart;
pub mod components;
pub mod data;
pub mod hotkey;
pub mod state;
pub mod theme;
pub mod updater;
pub mod websocket;
pub mod windows;

use gpui::*;
use std::time::Duration;

actions!(yaplay, [OpenSettings, ToggleControls, Quit]);

fn main() {
    Application::new().with_assets(theme::Assets).run(|cx| {
        theme::init(cx);
        cx.on_action(|_: &OpenSettings, cx| windows::open_settings(cx));
        cx.on_action(|_: &ToggleControls, cx| windows::toggle_controls(cx));
        cx.on_action(|_: &Quit, cx| updater::quit(cx));
        cx.bind_keys([
            KeyBinding::new(
                if cfg!(target_os = "macos") {
                    "cmd-,"
                } else {
                    "ctrl-,"
                },
                OpenSettings,
                None,
            ),
            KeyBinding::new(
                if cfg!(target_os = "macos") {
                    "cmd-shift-y"
                } else {
                    "ctrl-shift-y"
                },
                ToggleControls,
                None,
            ),
            KeyBinding::new(
                if cfg!(target_os = "macos") {
                    "cmd-q"
                } else {
                    "ctrl-q"
                },
                Quit,
                None,
            ),
        ]);
        cx.set_menus(vec![Menu {
            name: "Yaplay".into(),
            items: vec![
                MenuItem::action("Settings…", OpenSettings),
                MenuItem::action("Show / hide controls", ToggleControls),
                MenuItem::separator(),
                MenuItem::action("Quit Yaplay", Quit),
            ],
        }]);
        cx.set_global(windows::Desktop::default());
        cx.set_global(components::user_cell::AvatarCache::default());
        if let Err(error) = hotkey::register() {
            eprintln!("Global shortcut: {error}");
        }
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                updater::quit(cx);
            }
        })
        .detach();
        let saved = data::configuration::Configuration::load().ok();
        let started = saved.is_some_and(|config| {
            state::app_state().write().unwrap().config = config.clone();
            match websocket::start_server(config.websocket_port, state::app_state()) {
                Ok(()) => windows::open_overlay(cx).is_ok(),
                Err(error) => {
                    state::app_state().write().unwrap().startup_error = Some(error);
                    false
                }
            }
        });
        updater::init(cx);
        if !started {
            windows::open_settings(cx);
        }
        cx.spawn(async move |cx| {
            let mut previous = None;
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                if cx
                    .update(|cx| {
                        if hotkey::take_toggle_press() {
                            windows::toggle_controls(cx);
                        }
                        let snapshot = {
                            let shared = state::app_state();
                            let mut state = shared.write().unwrap();
                            state.prune_notifications();
                            state.snapshot()
                        };
                        // Capture drag positions before panels close, then
                        // reconcile which panels should be open.
                        windows::persist_layout(cx);
                        if previous.as_ref() != Some(&snapshot) {
                            previous = Some(snapshot);
                            windows::sync(cx);
                            cx.refresh_windows();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    });
}
