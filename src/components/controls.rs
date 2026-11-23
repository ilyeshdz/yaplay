use crate::{
    state::{Capability, ControlAction, Integration, app_state},
    theme,
};
use gpui::{prelude::*, *};
use gpui_component::{
    Disableable, Icon, Sizable,
    button::{Button, ButtonCustomVariant, ButtonVariants},
};

pub fn width(integration: Option<&Integration>) -> f32 {
    let count = integration.map_or(3, |i| i.actions.len());
    354.0 - (3 - count) as f32 * 54.0
}
pub const HEIGHT: f32 = 80.0;

fn control_button(id: &'static str, cx: &App) -> Button {
    Button::new(id)
        // Keep 24px icons inside generous 46px hit targets.
        .with_size(px(32.0))
        .size(px(46.0))
        .border_0()
        .rounded(px(14.0))
        .custom(
            ButtonCustomVariant::new(cx)
                .color(rgb(theme::SURFACE).into())
                .foreground(rgb(theme::TEXT).into())
                .border(rgba(0x00000000).into())
                .hover(rgb(0x302b2c).into())
                .active(rgb(0x3d3432).into())
                .shadow(false),
        )
}

fn drag_grip() -> impl IntoElement {
    let mut grip = div()
        .id("controls-grip")
        .w(px(6.0))
        .h(px(46.0))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .justify_center()
        .items_center()
        .gap(px(4.0))
        .cursor_grab()
        .on_mouse_down(MouseButton::Left, |_, window, _| {
            crate::windows::begin_window_drag(window)
        });
    for _ in 0..3 {
        grip = grip.child(div().size(px(3.0)).rounded_full().bg(rgba(0xb4b1ba66)));
    }
    grip
}

pub struct Controls;
impl Render for Controls {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = app_state().read().unwrap().snapshot();
        let width = px(width(current.integration.as_ref()));
        if window.viewport_size().width != width {
            window.resize(size(width, px(HEIGHT)));
        }
        let enabled = current.in_call && current.integration.is_some();
        let visible = |action| {
            current
                .integration
                .as_ref()
                .is_none_or(|i| i.actions.contains(&action))
        };
        let mic = current.mic_muted;
        let deaf = current.deafened;
        let arranging = current.arranging;
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .p(px(12.0))
                    .rounded(px(14.0))
                    .bg(rgb(theme::SURFACE))
                    .child(drag_grip())
                    .when(visible(Capability::SetMute), |row| {
                        row.child(
                            control_button("mute", cx)
                                .icon(Icon::default().path(if mic {
                                    "icons/microphone-off.svg"
                                } else {
                                    "icons/microphone.svg"
                                }))
                                .tooltip(if mic {
                                    "Unmute microphone"
                                } else {
                                    "Mute microphone"
                                })
                                .when(mic, |b| b.primary())
                                .disabled(!enabled)
                                .on_click(move |_, _, _| {
                                    app_state()
                                        .read()
                                        .unwrap()
                                        .request_control(ControlAction::SetMute { muted: !mic })
                                }),
                        )
                    })
                    .when(visible(Capability::SetDeafen), |row| {
                        row.child(
                            control_button("deafen", cx)
                                .icon(Icon::default().path(if deaf {
                                    "icons/headphones-off.svg"
                                } else {
                                    "icons/headphones.svg"
                                }))
                                .tooltip(if deaf { "Undeafen" } else { "Deafen" })
                                .when(deaf, |b| b.primary())
                                .disabled(!enabled)
                                .on_click(move |_, _, _| {
                                    app_state().read().unwrap().request_control(
                                        ControlAction::SetDeafen { deafened: !deaf },
                                    )
                                }),
                        )
                    })
                    .when(visible(Capability::LeaveCall), |row| {
                        row.child(
                            control_button("leave", cx)
                                .icon(Icon::default().path("icons/phone-off.svg"))
                                .tooltip("Leave call")
                                .disabled(!enabled)
                                .on_click(|_, _, _| {
                                    app_state()
                                        .read()
                                        .unwrap()
                                        .request_control(ControlAction::LeaveCall {})
                                }),
                        )
                    })
                    .child(
                        control_button("arrange", cx)
                            .icon(Icon::default().path("icons/move.svg"))
                            .tooltip(if arranging {
                                "Done moving panels"
                            } else {
                                "Move panels"
                            })
                            .when(arranging, |b| b.primary())
                            .on_click(|_, _, cx| crate::windows::toggle_arranging(cx)),
                    )
                    .child(
                        control_button("settings", cx)
                            .icon(Icon::default().path("icons/settings.svg"))
                            .tooltip(if crate::updater::has_update(cx) {
                                "Update available · Settings"
                            } else {
                                "Settings"
                            })
                            .when(crate::updater::has_update(cx), |b| b.primary())
                            .on_click(|_, _, cx| cx.defer(crate::windows::open_settings)),
                    )
                    .child(
                        control_button("exit", cx)
                            .icon(Icon::default().path("icons/exit.svg"))
                            .tooltip("Exit Yaplay")
                            .on_click(|_, _, cx| crate::updater::quit(cx)),
                    ),
            )
    }
}
