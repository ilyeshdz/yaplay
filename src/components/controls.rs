use crate::{
    state::{ControlAction, app_state},
    theme,
};
use gpui::{prelude::*, *};
use gpui_component::{
    Disableable, Icon, Sizable,
    button::{Button, ButtonCustomVariant, ButtonVariants},
};

pub const WIDTH: f32 = 296.0;
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

pub struct Controls;
impl Render for Controls {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = app_state().read().unwrap().snapshot();
        let enabled = current.in_call && current.controls_available;
        let mic = current.mic_muted;
        let deaf = current.deafened;
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
                    .child(
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
                    .child(
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
                                app_state()
                                    .read()
                                    .unwrap()
                                    .request_control(ControlAction::SetDeafen { deafened: !deaf })
                            }),
                    )
                    .child(
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
