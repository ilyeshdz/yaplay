pub mod components;
pub mod data;
pub mod hotkey;
pub mod state;
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

    let settings = WindowConfig::new(app)
        .with_title("Yaplay Settings")
        .with_min_size(400.0, 800.0);

    // Draw the settings background underneath the native macOS titlebar,
    // retaining the traffic-light buttons and native titlebar dragging.
    #[cfg(target_os = "macos")]
    let settings = settings.with_window_attributes(|attributes, _| {
        attributes
            .with_titlebar_transparent(true)
            .with_title_hidden(true)
            .with_fullsize_content_view(true)
    });

    launch(LaunchConfig::new().with_window(settings));
}

fn app() -> impl IntoElement {
    use_init_theme(dark_theme);

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
                        .child(label().text("Yaplay").title().theme_color())
                        .child(label().text("Configure your experience").subtitle().theme_color()),
                )
                .child(config_list()),
        )
}
