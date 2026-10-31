use std::time::Duration;

use async_io::Timer;
use freya::prelude::*;

use crate::state::{app_state, OverlaySnapshot};

use super::{notification_list::notification_list, voice_list::voice_list};

pub fn overlay() -> impl IntoElement {
    use_init_theme(dark_theme);

    let mut snapshot = use_state(OverlaySnapshot::default);

    use_hook(move || {
        spawn(async move {
            loop {
                let next = {
                    let shared = app_state();
                    let mut state = shared
                        .write()
                        .unwrap_or_else(|error| error.into_inner());
                    state.prune_notifications();
                    state.snapshot()
                };
                snapshot.set_if_modified(next);
                Timer::after(Duration::from_millis(100)).await;
            }
        });
    });

    let current = snapshot.read();

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .child(voice_list(
            &current.voice_members,
            current.mic_muted,
            current.in_call,
        ))
        .child(notification_list(&current.notifications))
}
