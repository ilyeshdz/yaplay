use freya::prelude::*;

use crate::data::configuration::Configuration;
use crate::state::app_state;
use crate::websocket;
use crate::windows::{
    CONTROLS_HEIGHT, CONTROLS_WIDTH, controls_window, overlay_window, target_monitor,
};

pub fn config_list() -> impl IntoElement {
    let mut config = use_state(|| Configuration::default());
    let port_text = use_state(|| "8080".to_string());
    let monitors = use_state(|| display_info::DisplayInfo::all().unwrap_or_default());
    let monitor_index = use_state(|| 0usize);
    let mut error = use_state(|| Option::<String>::None);

    let platform = Platform::get();

    let monitor_options: Vec<String> = monitors
        .read()
        .iter()
        .map(|m| m.friendly_name.clone())
        .collect();
    let selected_name = monitor_options
        .get(*monitor_index.read())
        .cloned()
        .unwrap_or_default();

    let error_text = error.read().clone().unwrap_or_default();

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .vertical()
        .main_align(Alignment::SpaceBetween)
        .padding(Gaps::new_all(20.0))
        .child(Card::new().width(Size::fill()).child(
            rect()
                .vertical()
                .spacing(16.0)
                .child(label().text("Configuration").subtitle().theme_color())
                .child(
                    rect()
                        .width(Size::fill())
                        .horizontal()
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(label().text("Display").body().theme_color())
                        .child(Select::new().selected_item(selected_name).children(
                            monitor_options.iter().enumerate().map(|(i, val)| {
                                let idx = i;
                                let mut mon_idx = monitor_index.clone();
                                let mut cfg = config.clone();
                                let mon = monitors.read()[i].id;
                                MenuItem::new()
                                    .selected(*mon_idx.read() == idx)
                                    .on_press(move |_| {
                                        *mon_idx.write() = idx;
                                        cfg.write().selected_monitor_id = mon;
                                    })
                                    .child(val.clone())
                                    .into()
                            }),
                        )),
                )
                .child(
                    rect()
                        .width(Size::fill())
                        .horizontal()
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(label().text("Enable Notifications").body().theme_color())
                        .child(Switch::new().toggled(config.read().notifications_enabled).on_toggle(
                            EventHandler::new(move |_| {
                                let current = config.read().notifications_enabled;
                                config.write().notifications_enabled = !current;
                            }),
                        )),
                )
                .child(
                    rect()
                        .width(Size::fill())
                        .horizontal()
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(label().text("WebSocket Port").body().theme_color())
                        .child(Input::new(port_text.clone()).placeholder("8080".to_string())),
                ),
        ))
        .child(
            rect()
                .width(Size::fill())
                .vertical()
                .spacing(12.0)
                .child(
                    label()
                        .text("Start opens the overlay and closes this window.")
                        .body()
                        .theme_color(),
                )
                .child(
                    label()
                        .text(error_text)
                        .color(Color::from_rgb(255, 95, 95)),
                )
                .child(
                    Button::new()
                        .width(Size::fill())
                        .height(Size::px(44.0))
                        .on_press(move |_| {
                            let port: u16 = match port_text.read().parse() {
                                Ok(port) if port != 0 => port,
                                _ => {
                                    *error.write() =
                                        Some("The WebSocket port must be a number between 1 and 65535.".to_string());
                                    return;
                                }
                            };

                            let monitor_id = monitors
                                .read()
                                .get(*monitor_index.read())
                                .map(|display| display.id)
                                .unwrap_or(0);

                            {
                                let shared = app_state();
                                let mut state = shared
                                    .write()
                                    .unwrap_or_else(|error| error.into_inner());
                                state.config = Configuration {
                                    selected_monitor_id: monitor_id,
                                    notifications_enabled: config.read().notifications_enabled,
                                    websocket_port: port,
                                };
                                state.seed_fake_voice_state();
                            }

                            if let Err(err) = websocket::start_server(port, app_state()) {
                                *error.write() = Some(err);
                                return;
                            }

                            let platform = platform.clone();
                            let monitor = target_monitor(monitor_id);
                            spawn(async move {
                                let overlay_id =
                                    platform.launch_window(overlay_window(monitor)).await;
                                let controls_id =
                                    platform.launch_window(controls_window(monitor)).await;
                                let _ = (overlay_id, controls_id);

                                let settings_id = platform
                                    .post_callback(|window_id, _| window_id)
                                    .await
                                    .ok();
                                if let Some(settings_id) = settings_id {
                                    platform.close_window(settings_id);
                                }
                            });
                        })
                        .child(label().text("Start")),
                )
                .child(
                    label()
                        .text(format!(
                            "Control panel: {}x{} px, toggle with the global shortcut.",
                            CONTROLS_WIDTH, CONTROLS_HEIGHT
                        ))
                        .color(Color::GRAY),
                ),
        )
}
