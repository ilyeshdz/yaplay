pub mod components;
pub mod data;
use freya::prelude::*;
use freya::winit::window::WindowLevel::AlwaysOnTop;
use crate::components::user_cell::UserCell;

fn main() {
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_decorations(false)
                .with_transparency(true)
                .with_background((0, 0, 0, 0))
                .with_window_attributes(|attrs, _| {
                    attrs
                        .with_window_level(AlwaysOnTop)
                        .with_maximized(true)
                })
                .with_window_handle(|window| {
                    window.set_cursor_hittest(false).unwrap();
                })
        ),
    )
}

fn app() -> impl IntoElement {
    rect()
        .width(Size::px(200.0))
        .height(Size::px(200.0))
        .child(UserCell {
            image_color: Color::BLUE,
            name: String::from("my name")
        })
        .child(UserCell {
            image_color: Color::RED,
            name: String::from("my second name")
        })
}
