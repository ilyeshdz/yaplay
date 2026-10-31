use std::time::Duration;

use async_io::Timer;
use freya::prelude::*;

use crate::state::{app_state, OverlaySnapshot};

pub fn controls() -> impl IntoElement {
    use_init_theme(dark_theme);

    let theme = use_theme();
    let colors = theme.read().colors.clone();

    let mut snapshot = use_state(OverlaySnapshot::default);

    let platform = Platform::get();
    let quit_platform = platform.clone();

    use_hook(move || {
        spawn(async move {
            let mut last_open: Option<bool> = None;
            loop {
                if crate::hotkey::take_toggle_press() {
                    if let Ok(mut state) = app_state().write() {
                        state.controls_open = !state.controls_open;
                    }
                }

                let next = {
                    let shared = app_state();
                    let mut state = shared
                        .write()
                        .unwrap_or_else(|error| error.into_inner());
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
                .background(Color::from_argb(225, 24, 24, 30))
                .border(Border::new().fill(colors.border).width(1.0))
                .rounded_full()
                .child(
                    Button::new()
                        .on_press(move |_| {
                            if let Ok(mut state) = app_state().write() {
                                state.mic_muted = !state.mic_muted;
                            }
                        })
                        .child(label().text(mic_label)),
                )
                .child(
                    Button::new()
                        .on_press(move |_| {
                            if let Ok(mut state) = app_state().write() {
                                state.deafened = !state.deafened;
                            }
                        })
                        .child(label().text(deafen_label)),
                )
                .child(
                    Button::new()
                        .on_press(move |_| {
                            let _ =
                                quit_platform.post_callback(|_, context| context.exit());
                        })
                        .child(label().text("Exit")),
                ),
        )
}
