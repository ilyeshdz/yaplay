use gpui::{prelude::*, *};

/// Fullscreen, click-through window that keeps the app alive and anchors the
/// overlay layer. Floating panels are their own windows so they can be dragged
/// and placed independently.
pub struct Overlay;

impl Render for Overlay {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}
