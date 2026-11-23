use super::user_cell::user_cell;
use crate::{
    data::{
        appearance::{Appearance, Corner, parse_color},
        configuration::Configuration,
        layout::{Layout, Panel},
    },
    state::{VoiceMember, app_state},
    theme,
    windows::{self, Desktop},
};
use gpui::{prelude::*, *};
use gpui_component::{
    Disableable, IndexPath,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    select::{Select, SelectEvent, SelectItem, SelectState},
    slider::{Slider, SliderState},
    switch::Switch,
};

const RANGES: [(&str, f32, f32); 6] = [
    ("Scale (%)", 75.0, 150.0),
    ("Panel width", 180.0, 400.0),
    ("Background opacity (%)", 0.0, 100.0),
    ("Corner radius", 0.0, 24.0),
    ("Screen margin", 8.0, 80.0),
    ("Maximum visible people", 1.0, 20.0),
];

#[derive(Clone)]
struct DisplayChoice {
    id: u32,
    label: SharedString,
}
impl SelectItem for DisplayChoice {
    type Value = u32;
    fn title(&self) -> SharedString {
        self.label.clone()
    }
    fn value(&self) -> &u32 {
        &self.id
    }
}

pub struct Settings {
    display: Entity<SelectState<Vec<DisplayChoice>>>,
    config: Configuration,
    port: Entity<InputState>,
    colors: Vec<Entity<InputState>>,
    sliders: Vec<Entity<SliderState>>,
    error: String,
    start_at_login: Option<bool>,
    startup_error: String,
    /// Set when a corner or appearance reset should drop the saved panel positions.
    reset_panels: bool,
    _subscriptions: Vec<Subscription>,
}

impl Settings {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let shared = app_state();
        let mut state = shared.write().unwrap();
        let config = state.config.clone();
        let error = state.startup_error.take().unwrap_or_default();
        drop(state);
        let port = cx
            .new(|cx| InputState::new(window, cx).default_value(config.websocket_port.to_string()));
        let style = &config.appearance;
        let mut subscriptions = vec![];
        let primary = cx.primary_display().map(|d| u32::from(d.id()));
        let choices: Vec<_> = cx
            .displays()
            .iter()
            .enumerate()
            .map(|(index, display)| {
                let id = u32::from(display.id());
                DisplayChoice {
                    id,
                    label: format!(
                        "Display {} · {} × {}{}",
                        index + 1,
                        f32::from(display.bounds().size.width) as u32,
                        f32::from(display.bounds().size.height) as u32,
                        if Some(id) == primary {
                            " · Primary"
                        } else {
                            ""
                        }
                    )
                    .into(),
                }
            })
            .collect();
        let selected = choices
            .iter()
            .position(|d| d.id == config.selected_monitor_id)
            .or_else(|| choices.iter().position(|d| Some(d.id) == primary));
        let display = cx.new(|cx| {
            SelectState::new(
                choices,
                selected.map(|i| IndexPath::default().row(i)),
                window,
                cx,
            )
        });
        subscriptions.push(cx.subscribe(
            &display,
            |this, _, event: &SelectEvent<Vec<DisplayChoice>>, cx| {
                if let SelectEvent::Confirm(Some(id)) = event {
                    this.config.selected_monitor_id = *id;
                    cx.notify();
                }
            },
        ));
        let colors = [&style.background, &style.foreground, &style.accent]
            .into_iter()
            .map(|value| {
                let input = cx.new(|cx| InputState::new(window, cx).default_value(value.clone()));
                subscriptions.push(
                    cx.subscribe(&input, |_, _, _: &gpui_component::input::InputEvent, cx| {
                        cx.notify()
                    }),
                );
                input
            })
            .collect();
        let values = [
            style.scale as f32,
            style.width as f32,
            style.opacity as f32,
            style.radius as f32,
            style.margin as f32,
            style.max_members as f32,
        ];
        let sliders = RANGES
            .iter()
            .zip(values)
            .map(|((_, min, max), value)| {
                let slider = cx.new(|_| appearance_slider(*min, *max, value));
                subscriptions.push(cx.subscribe(
                    &slider,
                    |_, _, _: &gpui_component::slider::SliderEvent, cx| cx.notify(),
                ));
                slider
            })
            .collect();
        let (start_at_login, startup_error) = match crate::autostart::is_enabled() {
            Ok(enabled) => (Some(enabled), String::new()),
            Err(error) => (
                None,
                format!("Cannot read start-at-login setting: {error:#}"),
            ),
        };
        Self {
            start_at_login,
            startup_error,
            reset_panels: false,
            display,
            config,
            port,
            colors,
            sliders,
            error,
            _subscriptions: subscriptions,
        }
    }
    fn set_start_at_login(&mut self, enabled: bool, cx: &mut Context<Self>) {
        match crate::autostart::set_enabled(enabled) {
            Ok(actual) => {
                self.start_at_login = Some(actual);
                self.startup_error.clear();
                if actual != enabled {
                    self.startup_error = "The operating system did not apply the startup preference. Check your system's startup settings.".into();
                }
            }
            Err(error) => {
                // An OS operation can partially succeed; always re-read its state.
                self.start_at_login = crate::autostart::is_enabled().ok();
                self.startup_error = format!("Cannot change start-at-login setting: {error:#}");
            }
        }
        cx.notify();
    }
    fn appearance(&self, cx: &App) -> Appearance {
        let mut style = self.config.appearance.clone();
        let values: Vec<_> = self
            .sliders
            .iter()
            .map(|s| s.read(cx).value().start().round() as u16)
            .collect();
        style.scale = values[0];
        style.width = values[1];
        style.opacity = values[2] as u8;
        style.radius = values[3] as u8;
        style.margin = values[4] as u8;
        style.max_members = values[5] as u8;
        style.background = self.colors[0].read(cx).value().trim().to_string();
        style.foreground = self.colors[1].read(cx).value().trim().to_string();
        style.accent = self.colors[2].read(cx).value().trim().to_string();
        style
    }
    fn save(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(port) = self.port.read(cx).value().parse::<u16>() else {
            self.error = "Enter a port between 1 and 65535.".into();
            cx.notify();
            return;
        };
        if port == 0 {
            self.error = "Port must be greater than zero.".into();
            cx.notify();
            return;
        }
        let mut style = self.appearance(cx);
        if [&style.background, &style.foreground, &style.accent]
            .iter()
            .any(|c| parse_color(c).is_none())
        {
            self.error = "Use six-digit colors such as #ffb49b.".into();
            cx.notify();
            return;
        }
        style.normalize();
        let old = app_state().read().unwrap().config.clone();
        let mut layout = old.layout.clone();
        if old.selected_monitor_id != self.config.selected_monitor_id {
            layout = Layout::default();
        } else if self.reset_panels {
            layout.clear(&[Panel::Roster, Panel::Notifications]);
        }
        let next = Configuration {
            websocket_port: port,
            appearance: style,
            layout,
            ..self.config.clone()
        };
        let result = (|| -> anyhow::Result<()> {
            crate::websocket::start_server(port, app_state()).map_err(anyhow::Error::msg)?;
            if let Err(error) = next.save() {
                let _ = crate::websocket::start_server(old.websocket_port, app_state());
                return Err(error.into());
            }
            app_state().write().unwrap().config = next;
            windows::open_overlay(cx)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                cx.global_mut::<Desktop>().settings = None;
                window.remove_window();
            }
            Err(error) => {
                self.error = error.to_string();
                cx.notify();
            }
        }
    }
    fn reset(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let style = Appearance::default();
        let values = [
            style.scale as f32,
            style.width as f32,
            style.opacity as f32,
            style.radius as f32,
            style.margin as f32,
            style.max_members as f32,
        ];
        for (slider, value) in self.sliders.iter().zip(values) {
            slider.update(cx, |s, cx| s.set_value(value, window, cx));
        }
        for (input, value) in
            self.colors
                .iter()
                .zip([&style.background, &style.foreground, &style.accent])
        {
            input.update(cx, |s, cx| s.set_value(value.clone(), window, cx));
        }
        self.config.appearance = style;
        self.reset_panels = true;
        cx.notify();
    }
}

impl Render for Settings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut preview = self.appearance(cx);
        preview.normalize();
        let style = &self.config.appearance;
        let mut form = div()
            .flex()
            .flex_col()
            .gap(px(18.0))
            .p(px(20.0))
            .bg(rgb(theme::SURFACE))
            .rounded(px(12.0))
            .child(crate::updater::panel(cx))
            .child(
                Switch::new("automatic-update-checks")
                    .label("Automatically check for updates")
                    .checked(self.config.automatic_update_checks)
                    .on_click(cx.listener(|this, value, _, cx| {
                        this.config.automatic_update_checks = *value;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(theme::MUTED))
                    .child("Press Save or Start to apply this preference."),
            )
            .child(
                Switch::new("start-at-login")
                    .label("Start Yaplay when I sign in")
                    .checked(self.start_at_login.unwrap_or(false))
                    .disabled(self.start_at_login.is_none())
                    .on_click(cx.listener(|this, value, _, cx| {
                        this.set_start_at_login(*value, cx);
                    })),
            )
            .child(
                div().text_sm().text_color(rgb(theme::MUTED))
                    .child("Applies immediately for your user account. Install Yaplay in a permanent location first."),
            )
            .when(!self.startup_error.is_empty(), |form| {
                form.child(div().text_sm().text_color(rgb(0xff8c8c)).child(self.startup_error.clone()))
            })
            .child(div().text_lg().child("Overlay settings"));
        form = form.child(
            div().flex().flex_col().gap_2().child("Display").child(
                Select::new(&self.display)
                    .icon(gpui_component::Icon::default().path("icons/chevrons-up-down.svg"))
                    .w_full(),
            ),
        );
        let arranging = app_state().read().unwrap().arranging;
        form = form.child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    Switch::new("arrange-panels")
                        .label("Move panels freely")
                        .checked(arranging)
                        .on_click(cx.listener(|_, value, _, cx| {
                            crate::windows::set_arranging(cx, *value)
                        })),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme::MUTED))
                        .child("Drag the controls, voice panel, and notifications anywhere. Positions save automatically."),
                ),
        );
        form = form
            .child(
                Switch::new("notifications")
                    .label("Notifications")
                    .checked(self.config.notifications_enabled)
                    .on_click(cx.listener(|this, value, _, cx| {
                        this.config.notifications_enabled = *value;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child("Connection port")
                    .child(Input::new(&self.port)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(theme::MUTED))
                    .child("Use the same port in your Yapin integration."),
            )
            .child(div().text_lg().child("Appearance"))
            .child(
                div()
                    .p_3()
                    .rounded(px(preview.radius as f32))
                    .bg(preview.surface())
                    .child(user_cell(
                        &VoiceMember {
                            id: "preview".into(),
                            name: "Yapin friend".into(),
                            avatar_url: None,
                            color: (180, 120, 100),
                            speaking: true,
                            muted: false,
                            is_self: false,
                        },
                        false,
                        &preview,
                        cx,
                    )),
            );
        let mut corners = div().flex().flex_wrap().gap_2();
        for corner in Corner::ALL {
            corners = corners.child(
                Button::new(SharedString::from(corner.name()))
                    .label(corner.name())
                    .when(style.position == corner, |b| b.primary())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.config.appearance.position = corner;
                        this.reset_panels = true;
                        cx.notify();
                    })),
            );
        }
        form = form.child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child("Roster corner")
                .child(corners),
        );
        for (name, value, setter) in [
            (
                "Compact rows",
                style.compact,
                (|s: &mut Appearance, v| s.compact = v) as fn(&mut Appearance, bool),
            ),
            ("Profile pictures", style.show_avatars, |s, v| {
                s.show_avatars = v;
                if !v {
                    s.show_names = true;
                }
            }),
            ("Names", style.show_names, |s, v| {
                s.show_names = v;
                if !v {
                    s.show_avatars = true;
                }
            }),
            ("Roster heading", style.show_header, |s, v| {
                s.show_header = v
            }),
            ("Show yourself", style.show_self, |s, v| s.show_self = v),
            ("Active speakers first", style.speakers_first, |s, v| {
                s.speakers_first = v
            }),
        ] {
            form = form.child(
                Switch::new(name)
                    .label(name)
                    .checked(value)
                    .on_click(cx.listener(move |this, value, _, cx| {
                        setter(&mut this.config.appearance, *value);
                        cx.notify();
                    })),
            );
        }
        for (index, (name, _, _)) in RANGES.iter().enumerate() {
            form = form.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(format!(
                        "{name}: {}",
                        self.sliders[index].read(cx).value().start() as u16
                    ))
                    .child(Slider::new(&self.sliders[index])),
            );
        }
        for (index, name) in ["Background color", "Text color", "Speaking highlight"]
            .iter()
            .enumerate()
        {
            form = form.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(*name)
                    .child(Input::new(&self.colors[index])),
            );
        }
        form=form.child(div().text_sm().text_color(rgb(theme::MUTED)).child("Colors use #RRGGBB. Large calls show +N more. Choosing a corner resets the voice and notification positions."))
            .child(Button::new("reset").label("Reset appearance").on_click(cx.listener(Self::reset)));
        let running = cx.global::<Desktop>().overlay.is_some();
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::TEXT))
            .p(px(24.0))
            .pt(px(48.0))
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        svg()
                            .path("wordmark.svg")
                            .w(px(155.0))
                            .h(px(36.0))
                            .text_color(rgb(theme::ACCENT)),
                    )
                    .child(
                        div()
                            .text_color(rgb(theme::MUTED))
                            .child("An unofficial overlay for Yapin."),
                    ),
            )
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(form),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .when(!self.error.is_empty(), |d| {
                        d.child(
                            div()
                                .text_color(rgb(0xff8c8c))
                                .text_sm()
                                .child(self.error.clone()),
                        )
                    })
                    .child(
                        Button::new("save")
                            .primary()
                            .w_full()
                            .label(if running { "Save" } else { "Start" })
                            .on_click(cx.listener(Self::save)),
                    )
                    .child(div().text_sm().text_color(rgb(theme::MUTED)).child(
                        if cfg!(target_os = "macos") {
                            "⌘ ⇧ Y  ·  Show controls"
                        } else {
                            "Ctrl Shift Y  ·  Show controls"
                        },
                    )),
            )
    }
}

// Set the upper bound first: GPUI clamps on each builder call, and some
// appearance ranges start above the slider's default maximum of 100.
fn appearance_slider(min: f32, max: f32, value: f32) -> SliderState {
    SliderState::new()
        .max(max)
        .min(min)
        .step(1.0)
        .default_value(value)
}

#[cfg(test)]
mod tests {
    use super::{RANGES, appearance_slider};

    #[test]
    fn appearance_ranges_accept_both_endpoints() {
        for (_, min, max) in RANGES {
            assert_eq!(appearance_slider(min, max, min).value().start(), min);
            assert_eq!(appearance_slider(min, max, max).value().start(), max);
        }
    }
}
