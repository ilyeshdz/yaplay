pub mod components;
pub mod data;
pub mod hotkey;
pub mod state;
pub mod theme;
pub mod websocket;
pub mod windows;

use freya::prelude::*;

#[cfg(target_os = "macos")]
use freya::winit::platform::macos::WindowAttributesExtMacOS;

use crate::components::config_list;

fn main() {
    if let Err(err) = hotkey::register() {
        eprintln!("Failed to register the global shortcut: {err}");
    }

    let mut launch_config = LaunchConfig::new();
    let saved = data::configuration::Configuration::load().ok();
    let started = saved.as_ref().is_some_and(|config| {
        state::app_state().write().unwrap().config = config.clone();
        match websocket::start_server(config.websocket_port, state::app_state()) {
            Ok(()) => true,
            Err(error) => {
                state::app_state().write().unwrap().startup_error = Some(error);
                false
            }
        }
    });
    if started {
        let monitor = windows::target_monitor(saved.unwrap().selected_monitor_id);
        launch_config = launch_config
            .with_window(windows::overlay_window(monitor))
            .with_window(windows::controls_window(monitor));
    } else {
        launch_config = launch_config.with_window(settings_window());
    }
    launch(launch_config);
}

pub fn settings_window() -> WindowConfig {
    state::app_state().write().unwrap().settings_open = true;
    let settings = WindowConfig::new(app)
        .with_title("Yaplay Settings")
        .with_icon(LaunchConfig::window_icon(include_bytes!(
            "../assets/branding/window-icon.png"
        )))
        .with_size(480.0, 720.0)
        .with_min_size(440.0, 680.0)
        .with_background(theme::BACKGROUND);

    // Draw the settings background underneath the native macOS titlebar,
    // retaining the traffic-light buttons and native titlebar dragging.
    #[cfg(target_os = "macos")]
    let settings = settings.with_window_attributes(|attributes, _| {
        attributes
            .with_titlebar_transparent(true)
            .with_title_hidden(true)
            .with_fullsize_content_view(true)
    });

    settings
        .with_window_handle(|window| {
            state::app_state().write().unwrap().settings_window = Some(window.id());
        })
        .with_on_close(|_, _| {
            let shared = state::app_state();
            let mut state = shared.write().unwrap();
            state.settings_open = false;
            state.settings_window = None;
            CloseDecision::Close
        })
}

fn app() -> impl IntoElement {
    use_init_theme(theme::yaplay_theme);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .theme_background()
        .theme_color()
        .padding(if cfg!(target_os = "macos") {
            // Leave the native titlebar controls clear of settings content.
            Gaps::new(60.0, 32.0, 32.0, 32.0)
        } else {
            Gaps::new_all(32.0)
        })
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .vertical()
                .spacing(24.0)
                .child(
                    rect()
                        .vertical()
                        .spacing(8.0)
                        .child(
                            SvgViewer::new(
                                include_bytes!("../assets/branding/wordmark.svg") as &[u8]
                            )
                            .width(Size::px(206.4))
                            .height(Size::px(48.0)),
                        )
                        .child(
                            label()
                                .text("An unofficial overlay for Yapin.")
                                .body()
                                .color(theme::MUTED),
                        ),
                )
                .child(config_list()),
        )
}
