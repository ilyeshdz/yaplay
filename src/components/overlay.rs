use super::voice_list::voice_list;
use crate::state::app_state;
use gpui::{prelude::*, *};

pub struct Overlay;
impl Render for Overlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = app_state().read().unwrap().snapshot();
        let style = &current.appearance;
        let bounds = window.viewport_size();
        let viewport = (f32::from(bounds.width), f32::from(bounds.height));
        let scale = style.factor();
        let margin = style.margin as f32;
        let corner = style.position.opposite_side();
        let mut notes = div().absolute().flex().flex_col().gap(px(8.0));
        notes = if corner.is_right() {
            notes.right(px(margin))
        } else {
            notes.left(px(margin))
        };
        notes = if corner.is_bottom() {
            notes.bottom(px(margin))
        } else {
            notes.top(px(margin))
        };
        let card_height = 98.0 * scale;
        let capacity = ((viewport.1 - margin * 2.0).max(0.0) / (card_height + 8.0)) as usize;
        for note in current.notifications.iter().rev().take(capacity) {
            notes = notes.child(
                div()
                    .w(px(
                        (320.0 * scale).min((viewport.0 / 2.0 - margin * 2.0).max(60.0))
                    ))
                    .h(px(card_height))
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .p(px(12.0))
                    .rounded(px(style.radius as f32))
                    .bg(style.surface())
                    .text_color(style.text())
                    .child(
                        div()
                            .text_size(px(14.0 * scale))
                            .text_ellipsis()
                            .child(note.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(13.0 * scale))
                            .line_clamp(2)
                            .child(note.body.clone()),
                    ),
            );
        }
        div()
            .relative()
            .size_full()
            .overflow_hidden()
            .child(voice_list(
                &current.voice_members,
                current.mic_muted,
                current.in_call,
                style,
                viewport,
                cx,
            ))
            .child(notes)
    }
}
