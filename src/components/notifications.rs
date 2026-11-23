use crate::data::configuration::NotificationSettings;
use crate::{state::app_state, windows::begin_window_drag};
use gpui::{prelude::*, *};

pub struct Notifications;

impl Render for Notifications {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = app_state().read().unwrap().snapshot();
        let style = &current.appearance;
        let arranging = current.arranging;
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
        let size = window_size(
            current.notifications.len(),
            style,
            &current.notification_style,
            viewport,
            arranging,
        );
        resize(window, size);

        let scale = style.factor();
        let card_height = card_height(style, &current.notification_style);
        let capacity = visible_capacity(style, &current.notification_style, viewport, arranging);
        let mut notes = div().flex().flex_col().gap(px(8.0));
        let mut shown = 0;
        for note in current.notifications.iter().rev().take(capacity) {
            shown += 1;
            notes = notes.child(
                div()
                    .w_full()
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
                    .when(!current.notification_style.title_only, |card| {
                        card.child(
                            div()
                                .text_size(px(13.0 * scale))
                                .line_clamp(2)
                                .child(note.body.clone()),
                        )
                    }),
            );
        }
        if shown == 0 && arranging {
            notes = notes.child(
                div()
                    .w_full()
                    .h(px(card_height))
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(px(12.0))
                    .rounded(px(style.radius as f32))
                    .bg(style.surface())
                    .text_color(style.text())
                    .text_size(px(13.0))
                    .child("Notifications appear here"),
            );
        }
        div()
            .size_full()
            .when(arranging, |panel| {
                panel
                    .border_2()
                    .border_dashed()
                    .border_color(style.highlight())
                    .rounded(px(style.radius as f32))
                    .on_mouse_down(MouseButton::Left, |_, window, _| begin_window_drag(window))
            })
            .child(notes)
    }
}

fn resize(window: &mut Window, target: Size<Pixels>) {
    let current = window.viewport_size();
    if (f32::from(current.width) - f32::from(target.width)).abs() > 0.5
        || (f32::from(current.height) - f32::from(target.height)).abs() > 0.5
    {
        window.resize(target);
    }
}

fn card_height(
    style: &crate::data::appearance::Appearance,
    settings: &NotificationSettings,
) -> f32 {
    if settings.title_only {
        24.0 + 22.0 * style.factor()
    } else {
        98.0 * style.factor()
    }
}

fn visible_capacity(
    style: &crate::data::appearance::Appearance,
    settings: &NotificationSettings,
    viewport: (f32, f32),
    arranging: bool,
) -> usize {
    let card = card_height(style, settings) + 8.0;
    let available = (viewport.1 - style.margin as f32 * 2.0).max(card);
    let capacity = (available / card) as usize;
    capacity
        .max(if arranging { 1 } else { 0 })
        .min(settings.limit.into())
}

pub fn window_size(
    count: usize,
    style: &crate::data::appearance::Appearance,
    settings: &NotificationSettings,
    viewport: (f32, f32),
    arranging: bool,
) -> Size<Pixels> {
    let scale = style.factor();
    let capacity = visible_capacity(style, settings, viewport, arranging);
    let visible = count.min(capacity).max(if arranging { 1 } else { 0 });
    let width = (320.0 * scale).min((viewport.0 / 2.0 - style.margin as f32 * 2.0).max(60.0));
    let height = if visible == 0 {
        1.0
    } else {
        visible as f32 * card_height(style, settings) + (visible - 1) as f32 * 8.0
    };
    size(px(width), px(height))
}

#[cfg(test)]
mod tests {
    use super::window_size;
    use crate::data::appearance::Appearance;

    #[test]
    fn notification_window_grows_with_visible_cards() {
        let style = Appearance::default();
        let empty = window_size(0, &style, &Default::default(), (1920.0, 1080.0), false);
        assert_eq!(f32::from(empty.height), 1.0);
        let one = window_size(1, &style, &Default::default(), (1920.0, 1080.0), false);
        assert_eq!(f32::from(one.height), 98.0);
        let two = window_size(2, &style, &Default::default(), (1920.0, 1080.0), false);
        assert_eq!(f32::from(two.height), 204.0);
        let capped = window_size(20, &style, &Default::default(), (1920.0, 1080.0), false);
        assert!(f32::from(capped.height) < 600.0);
        let placeholder = window_size(0, &style, &Default::default(), (1920.0, 1080.0), true);
        assert_eq!(f32::from(placeholder.height), 98.0);
    }
}
