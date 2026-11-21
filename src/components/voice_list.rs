use super::user_cell::user_cell;
use crate::{data::appearance::Appearance, state::VoiceMember};
use gpui::{App, div, prelude::*, px};

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

pub fn voice_list(
    members: &[VoiceMember],
    mic_muted: bool,
    in_call: bool,
    style: &Appearance,
    viewport: (f32, f32),
    cx: &mut App,
) -> impl IntoElement {
    let margin = style.margin as f32;
    let mut panel = div().absolute().flex().flex_col();
    panel = if style.position.is_right() {
        panel.right(px(margin))
    } else {
        panel.left(px(margin))
    };
    panel = if style.position.is_bottom() {
        panel.bottom(px(margin))
    } else {
        panel.top(px(margin))
    };
    if !in_call && members.is_empty() {
        return panel;
    }
    let width =
        (style.width as f32 * style.factor()).min((viewport.0 / 2.0 - margin * 2.0).max(60.0));
    panel = panel
        .w(px(if style.show_names {
            width
        } else {
            80.0 * style.factor()
        }))
        .max_h(px((viewport.1 - margin * 2.0).max(0.0)))
        .overflow_hidden()
        .p(px(12.0))
        .bg(style.surface())
        .rounded(px(style.radius as f32));
    if style.show_header {
        panel = panel.child(
            div()
                .h(px(26.0 * style.factor()))
                .flex_shrink_0()
                .text_ellipsis()
                .text_size(px(13.0 * style.factor()))
                .text_color(style.text())
                .child(format!("Voice · {}", members.len())),
        );
    }
    let (visible, hidden) = visible_members(members, style, style.capacity(viewport.1));
    for member in visible {
        panel = panel.child(user_cell(member, mic_muted, style, cx));
    }
    if hidden > 0 {
        panel = panel.child(
            div()
                .text_size(px(12.0 * style.factor()))
                .text_color(style.text())
                .child(format!("+{hidden} more")),
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
                ..style
            },
            0,
        );
        assert!(visible.is_empty());
        assert_eq!(hidden, 499);
    }
}
