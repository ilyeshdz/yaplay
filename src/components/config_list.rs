use freya::prelude::*;

use crate::data::configuration::Configuration;
use crate::state::app_state;
use crate::websocket;
use crate::windows::{controls_window, overlay_window, target_monitor};

pub fn config_list() -> impl IntoElement {
    let mut config = use_state(|| app_state().read().unwrap().config.clone());
    let port_text = use_state(|| config.read().websocket_port.to_string());
    let monitors = use_state(|| display_info::DisplayInfo::all().unwrap_or_default());
    let monitor_index = use_state(|| {
        let target = target_monitor(config.read().selected_monitor_id);
        monitors
            .read()
            .iter()
            .position(|monitor| monitor.id == target.id)
            .unwrap_or(0)
    });
    let mut error = use_state(|| app_state().write().unwrap().startup_error.take());
    let running = use_state(|| app_state().read().unwrap().overlay_window.is_some());

    let mut saving = use_state(|| false);
    let platform = Platform::get();

    let monitor_options: Vec<String> = monitors
        .read()
        .iter()
        .map(|m| m.friendly_name.clone())
        .collect();
    let selected_name = monitor_options
        .get(*monitor_index.read())
        .cloned()
        .unwrap_or_else(|| "Primary display".to_string());

    let error_text = error.read().clone().unwrap_or_default();

    rect()
        .width(Size::fill())
        .height(Size::flex(1.0))
        .vertical()
        .main_align(Alignment::SpaceBetween)
        .child(
            Card::new().width(Size::fill()).child(
                rect()
                    .width(Size::fill())
                    .vertical()
                    .spacing(20.0)
                    .child(
                        label()
                            .text("Overlay settings")
                            .subtitle()
                            .font_weight(FontWeight::SEMI_BOLD)
                            .theme_color(),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .vertical()
                            .spacing(8.0)
                            .child(label().text("Display").body().theme_color())
                            .child(
                                Select::new()
                                    .width(Size::fill())
                                    .selected_item(selected_name)
                                    .children(monitor_options.iter().enumerate().map(
                                        |(i, val)| {
                                            let idx = i;
                                            let mut mon_idx = monitor_index;
                                            let mut cfg = config;
                                            let mon = monitors.read()[i].id;
                                            MenuItem::new()
                                                .selected(*mon_idx.read() == idx)
                                                .on_press(move |_| {
                                                    *mon_idx.write() = idx;
                                                    cfg.write().selected_monitor_id = mon;
                                                })
                                                .child(val.clone())
                                                .into()
                                        },
                                    )),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .horizontal()
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .child(label().text("Notifications").body().theme_color())
                            .child(
                                Switch::new()
                                    .toggled(config.read().notifications_enabled)
                                    .on_toggle(EventHandler::new(move |_| {
                                        let current = config.read().notifications_enabled;
                                        config.write().notifications_enabled = !current;
                                    })),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .vertical()
                            .spacing(8.0)
                            .child(label().text("Connection port").body().theme_color())
                            .child(
                                Input::new(port_text)
                                    .width(Size::fill())
                                    .placeholder("8080".to_string()),
                            )
                            .child(
                                label()
                                    .text("Use the same port in your Yapin integration.")
                                    .caption()
                                    .color(crate::theme::MUTED),
                            ),
                    ),
            ),
        )
        .child(
            rect()
                .width(Size::fill())
                .vertical()
                .spacing(12.0)
                .child(
                    label()
                        .text(if *running.read() {
                            "Save applies your settings and closes this window."
                        } else {
                            "Start saves your settings and opens the overlay."
                        })
                        .caption()
                        .color(crate::theme::MUTED),
                )
                .child(label().text(error_text).color(DARK_COLORS.error))
                .child(
                    Button::new()
                        .filled()
                        .width(Size::fill())
                        .height(Size::px(46.0))
                        .enabled(!*saving.read())
                        .on_press(move |_| {
                            if *saving.read() {
                                return;
                            }
                            let port: u16 = match port_text.read().parse() {
                                Ok(port) if port != 0 => port,
                                _ => {
                                    *error.write() = Some(
                                        "The WebSocket port must be a number between 1 and 65535."
                                            .to_string(),
                                    );
                                    return;
                                }
                            };

                            let monitor_id = monitors
                                .read()
                                .get(*monitor_index.read())
                                .map(|display| display.id)
                                .unwrap_or(0);

                            let next_config = Configuration {
                                selected_monitor_id: monitor_id,
                                notifications_enabled: config.read().notifications_enabled,
                                websocket_port: port,
                            };
                            if let Err(err) = websocket::start_server(port, app_state()) {
                                *error.write() = Some(err);
                                return;
                            }
                            if let Err(err) = next_config.save() {
                                let old_port = app_state().read().unwrap().config.websocket_port;
                                let _ = websocket::start_server(old_port, app_state());
                                *error.write() = Some(format!("Could not save settings: {err}"));
                                return;
                            }
                            app_state().write().unwrap().config = next_config;

                            saving.set(true);
                            let platform = platform.clone();
                            let monitor = target_monitor(monitor_id);
                            spawn(async move {
                                let (overlay, controls) = {
                                    let shared = app_state();
                                    let state = shared.read().unwrap();
                                    (state.overlay_window, state.controls_window)
                                };
                                if let (Some(overlay), Some(controls)) = (overlay, controls) {
                                    platform.with_window(Some(overlay), move |window| {
                                        window.set_outer_position(
                                            freya::winit::dpi::LogicalPosition::new(
                                                monitor.x, monitor.y,
                                            ),
                                        );
                                        let _ = window.request_inner_size(
                                            freya::winit::dpi::LogicalSize::new(
                                                monitor.width,
                                                monitor.height,
                                            ),
                                        );
                                    });
                                    platform.with_window(Some(controls), move |window| {
                                        let (x, y) = monitor.bottom_center(
                                            crate::windows::CONTROLS_WIDTH,
                                            crate::windows::CONTROLS_HEIGHT,
                                            32.0,
                                        );
                                        window.set_outer_position(
                                            freya::winit::dpi::LogicalPosition::new(x, y),
                                        );
                                    });
                                } else {
                                    platform.launch_window(overlay_window(monitor)).await;
                                    platform.launch_window(controls_window(monitor)).await;
                                }
                                let settings_id = {
                                    let shared = app_state();
                                    let mut state = shared.write().unwrap();
                                    state.settings_open = false;
                                    state.settings_window.take()
                                };
                                if let Some(id) = settings_id {
                                    platform.close_window(id);
                                }
                            });
                        })
                        .child(label().text(if *running.read() { "Save" } else { "Start" })),
                )
                .child(shortcut_hint()),
        )
}

fn shortcut_hint() -> impl IntoElement {
    let keys = if cfg!(target_os = "macos") {
        ["⌘", "⇧", "Y"]
    } else {
        ["Ctrl", "⇧", "Y"]
    };
    let mut hint = rect()
        .horizontal()
        .cross_align(Alignment::Center)
        .spacing(6.0)
        .child(
            label()
                .text("Show controls")
                .caption()
                .color(crate::theme::MUTED),
        );
    for key in keys {
        hint = hint.child(
            rect()
                .center()
                .padding(Gaps::new_symmetric(4.0, 8.0))
                .background(crate::theme::SURFACE)
                .border(Border::new().fill(crate::theme::BORDER).width(1.0))
                .corner_radius(6.0)
                .child(label().text(key).font_size(14.0).color(crate::theme::TEXT)),
        );
    }
    hint
}
