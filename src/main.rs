pub mod components;
pub mod data;
pub mod hotkey;
pub mod state;
pub mod websocket;
pub mod windows;

use freya::prelude::*;

use crate::components::config_list;

fn main() {
    if let Err(err) = hotkey::register() {
        eprintln!("Failed to register the global shortcut: {err}");
    }

    launch(
        LaunchConfig::new()
            .with_window(WindowConfig::new(app).with_min_size(400.0, 800.0)),
    );
}

fn app() -> impl IntoElement {
    use_init_theme(dark_theme);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .theme_background()
        .theme_color()
        .padding(Gaps::new_all(32.0))
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
