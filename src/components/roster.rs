use super::voice_list;
use crate::state::app_state;
use gpui::{prelude::*, *};

pub struct Roster;

impl Render for Roster {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = app_state().read().unwrap().snapshot();
        let display = window
            .display(cx)
            .map(|display| display.bounds())
            .unwrap_or(Bounds::new(
                point(px(0.0), px(0.0)),
                size(px(1280.0), px(720.0)),
            ));
        let viewport = (
            f32::from(display.size.width),
            f32::from(display.size.height),
        );
        let top = f32::from(window.bounds().origin.y) - f32::from(display.origin.y);
        let layout =
            voice_list::roster_layout(&current.voice_members, &current.appearance, viewport, top);
        let current_size = window.viewport_size();
        if (f32::from(current_size.width) - f32::from(layout.size.width)).abs() > 0.5
            || (f32::from(current_size.height) - f32::from(layout.size.height)).abs() > 0.5
        {
            window.resize(layout.size);
        }
        div().size_full().child(voice_list::roster_panel(
            &layout,
            current.mic_muted,
            &current.appearance,
            current.arranging,
            cx,
        ))
    }
}
