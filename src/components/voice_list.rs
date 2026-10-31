use freya::prelude::*;

use crate::state::VoiceMember;

use super::user_cell::UserCell;

pub fn voice_list(members: &[VoiceMember], mic_muted: bool, in_call: bool) -> impl IntoElement {
    let theme = use_theme();
    let colors = theme.read().colors.clone();

    let show = in_call || !members.is_empty();

    let mut panel = rect()
        .position(Position::new_global().top(48.0).left(24.0))
        .vertical()
        .spacing(8.0)
        .padding(if show {
            Gaps::new_all(12.0)
        } else {
            Gaps::new_all(0.0)
        })
        .background(if show {
            Color::from_argb(215, 18, 18, 24)
        } else {
            Color::TRANSPARENT
        })
        .rounded_lg()
        .border(if show {
            Border::new().fill(colors.border).width(1.0)
        } else {
            Border::new()
        });

    if show {
        panel = panel.child(
            label()
                .text("Voice")
                .body()
                .color(colors.text_secondary),
        );

        let mut list = rect().vertical().spacing(6.0);
        for member in members {
            let muted = member.muted || (member.is_self && mic_muted);
            list = list.child(UserCell {
                name: member.name.clone(),
                color: Color::from_rgb(member.color.0, member.color.1, member.color.2),
                speaking: member.speaking,
                muted,
                is_self: member.is_self,
            });
        }
        panel = panel.child(list);
    }

    panel
}
