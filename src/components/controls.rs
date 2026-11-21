use std::time::Duration;

use async_io::Timer;
use freya::prelude::*;

use crate::state::{OverlaySnapshot, app_state};

pub fn controls() -> impl IntoElement {
    use_init_theme(crate::theme::yaplay_theme);

    let theme = use_theme();
    let colors = theme.read().colors.clone();

    let mut snapshot = use_state(OverlaySnapshot::default);

    let platform = Platform::get();
    let quit_platform = platform.clone();
    let settings_platform = platform.clone();

    use_hook(move || {
        spawn(async move {
            let mut last_open: Option<bool> = None;
            loop {
                if crate::hotkey::take_toggle_press() {
                    let shared = app_state();
                    if let Ok(mut state) = shared.write() {
                        state.controls_open = !state.controls_open;
                    }
                }

                let next = {
                    let shared = app_state();
                    let mut state = shared.write().unwrap_or_else(|error| error.into_inner());
                    state.prune_notifications();
                    state.snapshot()
                };
                let open = next.controls_open;

                if last_open != Some(open) {
                    last_open = Some(open);
                    platform.with_window(None, move |window| {
                        let _ = window.set_cursor_hittest(open);
                    });
                }

                snapshot.set_if_modified(next);
                Timer::after(Duration::from_millis(50)).await;
            }
        });
    });

    let current = snapshot.read();

    if !current.controls_open {
        return rect().width(Size::fill()).height(Size::fill());
    }

    let mic_label = if current.mic_muted {
        "Unmute mic"
    } else {
        "Mute mic"
    };
    let deafen_label = if current.deafened {
        "Undeafen"
    } else {
        "Deafen"
    };

    let mic_icon: &[u8] = if current.mic_muted {
        include_bytes!("../assets/icons/microphone-off.svg")
    } else {
        include_bytes!("../assets/icons/microphone.svg")
    };
    let deafen_icon: &[u8] = if current.deafened {
        include_bytes!("../assets/icons/headphones-off.svg")
    } else {
        include_bytes!("../assets/icons/headphones.svg")
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .center()
        .child(
            rect()
                .horizontal()
                .spacing(10.0)
                .cross_align(Alignment::Center)
                .padding(Gaps::new_symmetric(12.0, 14.0))
                .background(crate::theme::OVERLAY_SURFACE)
                .border(Border::new().fill(colors.border).width(1.0))
                .corner_radius(12.0)
                .child(
                    Button::new()
                        .width(Size::px(42.0))
                        .height(Size::px(42.0))
                        .style_variant(if current.mic_muted {
                            ButtonStyleVariant::Filled
                        } else {
                            ButtonStyleVariant::Normal
                        })
                        .on_press(move |_| {
                            if let Ok(mut state) = app_state().write() {
                                state.mic_muted = !state.mic_muted;
                            }
                        })
                        .child(control_content(mic_icon, mic_label, current.mic_muted)),
                )
                .child(
                    Button::new()
                        .width(Size::px(42.0))
                        .height(Size::px(42.0))
                        .style_variant(if current.deafened {
                            ButtonStyleVariant::Filled
                        } else {
                            ButtonStyleVariant::Normal
                        })
                        .on_press(move |_| {
                            if let Ok(mut state) = app_state().write() {
                                state.deafened = !state.deafened;
                            }
                        })
                        .child(control_content(deafen_icon, deafen_label, current.deafened)),
                )
                .child(
                    Button::new()
                        .width(Size::px(42.0))
                        .height(Size::px(42.0))
                        .on_press(move |_| {
                            let shared = app_state();
                            let mut state = shared.write().unwrap();
                            if let Some(id) = state.settings_window {
                                settings_platform.with_window(Some(id), |window| {
                                    window.set_minimized(false);
                                    window.focus_window();
                                });
                            } else if !state.settings_open {
                                state.settings_open = true;
                                let platform = settings_platform.clone();
                                spawn(async move {
                                    platform.launch_window(crate::settings_window()).await;
                                });
                            }
                        })
                        .child(control_content(
                            include_bytes!("../assets/icons/settings.svg"),
                            "Settings",
                            false,
                        )),
                )
                .child(
                    Button::new()
                        .width(Size::px(42.0))
                        .height(Size::px(42.0))
                        .on_press(move |_| {
                            drop(quit_platform.post_callback(|_, context| context.exit()));
                        })
                        .child(control_content(
                            include_bytes!("../assets/icons/exit.svg"),
                            "Exit",
                            false,
                        )),
                ),
        )
}

/// Keep action names available to assistive technology without visible labels.
fn control_content(icon: &'static [u8], text: &'static str, active: bool) -> impl IntoElement {
    let color = if active {
        crate::theme::BACKGROUND
    } else {
        crate::theme::TEXT
    };
    rect().center().a11y_alt(text).child(
        SvgViewer::new(icon)
            .width(Size::px(18.0))
            .height(Size::px(18.0))
            .color(color),
    )
}
