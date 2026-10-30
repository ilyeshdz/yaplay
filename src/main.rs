use freya::prelude::{
    ChildrenExt, Color, ContainerExt, ContainerSizeExt, EventHandlersExt, Gaps, IntoElement,
    LaunchConfig, Size, StyleExt, WindowConfig, launch, rect, use_state,
};
use freya::prelude::OriginValue::Pixels;
use freya::winit::window::WindowLevel;

fn main() {
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_transparency(true)
                .with_decorations(false)
                .with_background((0, 0, 0, 0))
                .with_window_attributes(|attrs, _| {
                    attrs
                        .with_window_level(WindowLevel::AlwaysOnTop)
                        .with_maximized(true)
                }),
        ),
    )
}

fn app() -> impl IntoElement {
    let mut count = use_state(|| 0);

    rect()
        .width(Size::px(200.0))
        .height(Size::px(200.0))
        .background((35, 35, 35))
        .color(Color::WHITE)
        .padding(Gaps::new_all(12.))
        .on_mouse_up(move |_| *count.write() += 1)
        .child(format!("Click to increase = {}", count.read()))
}
