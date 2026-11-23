use super::user_cell::user_cell;
use crate::{data::appearance::Appearance, state::VoiceMember};
use gpui::{App, Pixels, Size, div, prelude::*, px, size};

/// Stable within each priority group, so quiet participants do not shuffle every frame.
pub fn visible_members<'a>(
    members: &'a [VoiceMember],
    style: &Appearance,
    capacity: usize,
) -> (Vec<&'a VoiceMember>, usize) {
    let mut members: Vec<_> = members
        .iter()
        .filter(|m| style.show_self || !m.is_self)
        .collect();
    if style.speakers_first {
        members.sort_by_key(|m| (!m.speaking, !m.is_self));
    }
    let hidden = members.len().saturating_sub(capacity);
    members.truncate(capacity);
    (members, hidden)
}

pub struct RosterLayout<'a> {
    pub visible: Vec<&'a VoiceMember>,
    pub hidden: usize,
    pub total: usize,
    pub size: Size<Pixels>,
}

pub fn roster_layout<'a>(
    members: &'a [VoiceMember],
    style: &Appearance,
    viewport: (f32, f32),
    top: f32,
) -> RosterLayout<'a> {
    let scale = style.factor();
    let margin = style.margin as f32;
    let available = (viewport.1 - top.max(0.0) - 12.0).max(72.0);
    let capacity = style.capacity(available).max(1);
    let (visible, hidden) = visible_members(members, style, capacity);
    let rows = visible.len().max(1);
    let header = if style.show_header { 26.0 * scale } else { 0.0 };
    let overflow = if hidden > 0 { 18.0 * scale } else { 0.0 };
    let height = (24.0 + header + rows as f32 * style.row_height() + overflow)
        .min((viewport.1 - 16.0).max(24.0));
    let width = if style.show_names {
        style.width as f32 * scale
    } else {
        80.0 * scale
    };
    let width = width.min((viewport.0 / 2.0 - margin * 2.0).max(60.0));
    RosterLayout {
        visible,
        hidden,
        total: members.len(),
        size: size(px(width), px(height)),
    }
}

pub fn roster_size(
    members: &[VoiceMember],
    style: &Appearance,
    viewport: (f32, f32),
) -> Size<Pixels> {
    roster_layout(members, style, viewport, style.margin as f32).size
}

pub fn roster_panel<'a>(
    layout: &RosterLayout<'a>,
    mic_muted: bool,
    style: &Appearance,
    arranging: bool,
    cx: &mut App,
) -> impl IntoElement {
    let scale = style.factor();
    let mut panel = div()
        .flex()
        .flex_col()
        .size_full()
        .p(px(12.0))
        .bg(style.surface())
        .rounded(px(style.radius as f32))
        .text_color(style.text())
        .when(arranging, |panel| {
            panel
                .border_2()
                .border_dashed()
                .border_color(style.highlight())
                .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                    crate::windows::begin_window_drag(window)
                })
        });
    if style.show_header {
        panel = panel.child(
            div()
                .h(px(26.0 * scale))
                .flex_shrink_0()
                .text_ellipsis()
                .text_size(px(13.0 * scale))
                .child(format!("Voice · {}", layout.total)),
        );
    }
    if layout.visible.is_empty() {
        panel = panel.child(
            div()
                .h(px(style.row_height()))
                .flex()
                .items_center()
                .text_size(px(13.0 * scale))
                .text_color(style.text())
                .child(if style.speakers_only {
                    "Waiting for a speaker"
                } else {
                    "No one in the call yet"
                }),
        );
    } else {
        for member in &layout.visible {
            panel = panel.child(user_cell(member, mic_muted, style, cx));
        }
    }
    if layout.hidden > 0 {
        panel = panel.child(
            div()
                .text_size(px(12.0 * scale))
                .text_color(style.text())
                .child(format!("+{} more", layout.hidden)),
        );
    }
    panel
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn huge_roster_prioritizes_speakers_and_counts_overflow() {
        let members: Vec<_> = (0..500)
            .map(|i| VoiceMember {
                id: i.to_string(),
                name: format!("Person {i}"),
                avatar_url: None,
                color: (1, 2, 3),
                speaking: i == 499,
                muted: false,
                is_self: i == 0,
            })
            .collect();
        let style = Appearance::default();
        let (visible, hidden) = visible_members(&members, &style, 8);
        assert_eq!(visible.len(), 8);
        assert_eq!(hidden, 492);
        assert_eq!(visible[0].id, "499");
        assert_eq!(visible[1].id, "0");
        let (visible, hidden) = visible_members(
            &members,
            &Appearance {
                show_self: false,
                ..style.clone()
            },
            0,
        );
        assert!(visible.is_empty());
        assert_eq!(hidden, 499);
    }

    #[test]
    fn roster_window_fits_the_content_and_screen() {
        let style = Appearance::default();
        let empty = roster_size(&[], &style, (1920.0, 1080.0));
        // Header plus one placeholder row.
        assert_eq!(f32::from(empty.height), 24.0 + 26.0 + 38.0);
        let member = VoiceMember {
            id: "1".into(),
            name: "Ada".into(),
            avatar_url: None,
            color: (1, 2, 3),
            speaking: false,
            muted: false,
            is_self: false,
        };
        let one = roster_size(&[member], &style, (1920.0, 1080.0));
        assert_eq!(f32::from(one.height), 24.0 + 26.0 + 38.0);
        let tall = roster_size(&[], &style, (1920.0, 40.0));
        assert!(f32::from(tall.height) <= 24.0);
        assert!(f32::from(tall.width) > 0.0);
    }
}
