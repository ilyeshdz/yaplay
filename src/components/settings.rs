use super::voice_list::{roster_layout, roster_panel};
use crate::{
    data::{
        appearance::{Appearance, Corner, parse_color},
        configuration::Configuration,
        layout::{Layout, Panel},
    },
    hotkey::{self, Action as ShortcutAction, Registry},
    state::{VoiceMember, app_state},
    theme,
    windows::{self, Desktop},
};
use gpui::{prelude::*, *};
use gpui_component::{
    Disableable, IndexPath,
    button::{Button, ButtonCustomVariant, ButtonVariants},
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    General,
    Overlay,
    Appearance,
    Updates,
}

impl Section {
    fn icon(self) -> &'static str {
        match self {
            Self::General => "icons/settings.svg",
            Self::Overlay => "icons/panels.svg",
            Self::Appearance => "icons/palette.svg",
            Self::Updates => "icons/update.svg",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Overlay => "Overlay",
            Self::Appearance => "Appearance",
            Self::Updates => "Updates",
        }
    }
}

pub struct Settings {
    section: Section,
    recorder_focus: FocusHandle,
    recording: Option<ShortcutAction>,
    typing: bool,
    shortcut_text: Entity<InputState>,
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
        let shortcut_text = cx.new(|cx| InputState::new(window, cx).placeholder("Ctrl+Shift+M"));
        let mut subscriptions = vec![cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() && !this.typing {
                this.end_recording(cx);
            }
        })];
        // Intercept before application bindings such as Quit can consume the chord.
        let settings = cx.weak_entity();
        let settings_window = window.window_handle();
        subscriptions.push(cx.intercept_keystrokes(move |event, window, cx| {
            if window.window_handle() == settings_window {
                let _ = settings.update(cx, |this, cx| {
                    if this.recording.is_some() && !this.typing {
                        cx.stop_propagation();
                        window.prevent_default();
                        if event.keystroke.key == "escape" {
                            this.end_recording(cx);
                        } else {
                            this.accept_shortcut(hotkey::recorded(&event.keystroke), cx);
                        }
                    }
                });
            }
        }));
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
            section: Section::General,
            recorder_focus: cx.focus_handle(),
            recording: None,
            typing: false,
            shortcut_text,
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
        let Some(port) = self
            .port
            .read(cx)
            .value()
            .parse::<u16>()
            .ok()
            .filter(|port| *port > 0)
        else {
            self.section = Section::General;
            self.port.update(cx, |input, cx| input.focus(window, cx));
            self.error = "Enter a port between 1 and 65535.".into();
            cx.notify();
            return;
        };
        let mut style = self.appearance(cx);
        if let Some(index) = [&style.background, &style.foreground, &style.accent]
            .iter()
            .position(|c| parse_color(c).is_none())
        {
            self.section = Section::Appearance;
            self.colors[index].update(cx, |input, cx| input.focus(window, cx));
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
            hotkey::ensure_registry(cx)?;
            cx.global_mut::<Registry>().apply(&next.shortcuts, || {
                crate::websocket::start_server(port, app_state()).map_err(anyhow::Error::msg)?;
                if let Err(error) = next.save() {
                    let _ = crate::websocket::start_server(old.websocket_port, app_state());
                    return Err(error.into());
                }
                Ok(())
            })?;
            {
                let shared = app_state();
                let mut state = shared.write().unwrap();
                state.config = next;
                state.prune_notifications();
            }
            windows::open_overlay(cx)?;
            Ok(())
        })();
        match result {
            Ok(()) => window.remove_window(),
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

fn note(text: impl Into<SharedString>) -> Div {
    div()
        .text_sm()
        .text_color(rgb(theme::MUTED))
        .child(text.into())
}

fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(rgb(theme::BORDER))
        .bg(rgb(theme::SURFACE))
}

fn preference_row(title: &'static str, detail: &'static str, control: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .gap_4()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(title)
                .child(note(detail)),
        )
        .child(div().flex_shrink_0().child(control))
}

fn number_preference(
    title: &'static str,
    value: u8,
    range: std::ops::RangeInclusive<u8>,
    cx: &Context<Settings>,
    set: fn(&mut Settings, u8),
) -> Div {
    let mut controls = div().flex().items_center().gap_2();
    for (index, label, next) in [
        (0usize, "−", value.checked_sub(1)),
        (1, "+", value.checked_add(1)),
    ] {
        if index == 1 {
            controls = controls.child(value.to_string());
        }
        let next = next.filter(|n| range.contains(n));
        controls = controls.child(
            Button::new((title, index))
                .label(label)
                .disabled(next.is_none())
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(next) = next {
                        set(this, next);
                        cx.notify();
                    }
                })),
        );
    }
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(title)
        .child(controls)
}

fn group(title: &'static str, contents: Div) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
        .child(contents)
}

impl Settings {
    fn end_recording(&mut self, cx: &mut Context<Self>) {
        if self.recording.take().is_some() {
            self.typing = false;
            if cx.has_global::<Registry>() {
                cx.global_mut::<Registry>().recorder = None;
            }
            cx.notify();
        }
    }

    pub fn accept_shortcut(&mut self, value: String, cx: &mut Context<Self>) {
        let Some(action) = self.recording else { return };
        let mut draft = self.config.shortcuts.clone();
        draft.set(action, value);
        match hotkey::ensure_registry(cx)
            .and_then(|()| cx.global::<Registry>().check(&draft, action))
        {
            Ok(()) => {
                self.config.shortcuts = draft;
                self.error.clear();
                self.end_recording(cx);
            }
            Err(error) => self.error = error.to_string(),
        }
        cx.notify();
    }

    fn shortcuts(&self, cx: &Context<Self>) -> Div {
        let mut rows = card();
        for action in ShortcutAction::ALL {
            let mut buttons = div().flex().gap_2().child(
                Button::new(("record", action as usize))
                    .label(
                        self.config
                            .shortcuts
                            .get(action)
                            .unwrap_or("Set shortcut")
                            .to_owned(),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.recording = Some(action);
                        this.typing = false;
                        this.error.clear();
                        if let Err(error) = hotkey::ensure_registry(cx) {
                            this.error = error.to_string();
                        } else {
                            cx.global_mut::<Registry>().recorder = Some(cx.weak_entity());
                        }
                        this.recorder_focus.focus(window);
                        cx.notify();
                    })),
            );
            if action != ShortcutAction::Controls && self.config.shortcuts.get(action).is_some() {
                buttons = buttons.child(
                    Button::new(("clear", action as usize))
                        .label("Clear")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            match action {
                                ShortcutAction::Mute => this.config.shortcuts.mute = None,
                                ShortcutAction::Deafen => this.config.shortcuts.deafen = None,
                                _ => {}
                            }
                            this.end_recording(cx);
                            this.error.clear();
                            cx.notify();
                        })),
                );
            }
            rows = rows.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(action.label())
                    .child(buttons),
            );
            if self.recording == Some(action) {
                let mut editor = div().flex().flex_wrap().items_center().gap_2();
                if self.typing {
                    editor = editor
                        .child(Input::new(&self.shortcut_text).w(px(170.0)))
                        .child(
                            Button::new("check-shortcut")
                                .label("Use shortcut")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.accept_shortcut(
                                        this.shortcut_text.read(cx).value().trim().to_owned(),
                                        cx,
                                    );
                                })),
                        );
                } else {
                    editor = editor.child(note("Press a key combination…")).child(
                        Button::new("type-shortcut")
                            .label("Type instead")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.typing = true;
                                if cx.has_global::<Registry>() {
                                    cx.global_mut::<Registry>().recorder = None;
                                }
                                this.shortcut_text.update(cx, |input, cx| {
                                    input.set_value("", window, cx);
                                    input.focus(window, cx);
                                });
                                cx.notify();
                            })),
                    );
                }
                rows = rows.child(
                    editor.child(Button::new("cancel-shortcut").label("Cancel").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.end_recording(cx);
                            this.error.clear();
                        }),
                    )),
                );
            }
        }
        group(
            "Global shortcuts",
            rows.child(note(
                "Mute and deafen require a call with a compatible integration.",
            )),
        )
    }

    fn general(&self, cx: &Context<Self>) -> Div {
        let startup = card()
            .child(preference_row(
                "Start Yaplay when I sign in",
                "Applies immediately. Install Yaplay in a permanent location first.",
                Switch::new("start-at-login")
                    .checked(self.start_at_login.unwrap_or(false))
                    .disabled(self.start_at_login.is_none())
                    .on_click(
                        cx.listener(|this, value, _, cx| this.set_start_at_login(*value, cx)),
                    ),
            ))
            .when(!self.startup_error.is_empty(), |d| {
                d.child(note(self.startup_error.clone()).text_color(rgb(0xff8c8c)))
            });
        let connection = card().child(preference_row(
            "WebSocket port",
            "Use the same port in your Yapin integration.",
            div().w(px(94.0)).child(Input::new(&self.port)),
        ));
        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(group("Startup", startup))
            .child(group("Connection", connection))
            .child(self.shortcuts(cx))
    }

    fn overlay(&self, cx: &Context<Self>) -> Div {
        let notes = &self.config.notifications;
        let mut corners = div().flex().flex_wrap().gap_2();
        for corner in Corner::ALL {
            corners = corners.child(
                Button::new(SharedString::from(corner.name()))
                    .label(corner.name())
                    .when(self.config.appearance.position == corner, |b| b.primary())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.config.appearance.position = corner;
                        this.reset_panels = true;
                        cx.notify();
                    })),
            );
        }

        let arranging = app_state().read().unwrap().arranging;
        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(group(
                "Display",
                card().child(
                    Select::new(&self.display)
                        .icon(gpui_component::Icon::default().path("icons/chevrons-up-down.svg"))
                        .w_full(),
                ),
            ))
            .child(group(
                "Panel placement",
                card()
                    .child("Roster corner")
                    .child(corners)
                    .child(note(
                        "Choosing a corner resets the voice and notification positions.",
                    ))
                    .child(div().h(px(1.0)).bg(rgb(theme::BORDER)))
                    .child(preference_row(
                        "Move panels freely",
                        "Drag panels on screen to reposition them. Positions save automatically.",
                        Switch::new("arrange-panels").checked(arranging).on_click(
                            cx.listener(|_, value, _, cx| windows::set_arranging(cx, *value)),
                        ),
                    )),
            ))
            .child(group(
                "Notifications",
                card()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child("Show notifications")
                            .child(
                                Switch::new("notifications")
                                    .checked(self.config.notifications_enabled)
                                    .on_click(cx.listener(|this, value, _, cx| {
                                        this.config.notifications_enabled = *value;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(number_preference(
                        "Display time (seconds)",
                        notes.seconds,
                        1..=30,
                        cx,
                        |s, v| s.config.notifications.seconds = v,
                    ))
                    .child(number_preference(
                        "Maximum cards",
                        notes.limit,
                        1..=5,
                        cx,
                        |s, v| s.config.notifications.limit = v,
                    ))
                    .child(preference_row(
                        "Titles only",
                        "Hide the notification body.",
                        Switch::new("title-only")
                            .checked(notes.title_only)
                            .on_click(cx.listener(|this, value, _, cx| {
                                this.config.notifications.title_only = *value;
                                cx.notify();
                            })),
                    )),
            ))
    }

    fn appearance_page(&self, cx: &mut Context<Self>) -> Div {
        let mut preview = self.appearance(cx);
        preview.normalize();
        let mut members = vec![
            VoiceMember {
                id: "self".into(),
                name: "Alex".into(),
                avatar_url: None,
                color: (106, 121, 154),
                speaking: false,
                muted: true,
                is_self: true,
            },
            VoiceMember {
                id: "friend".into(),
                name: "Yapin friend".into(),
                avatar_url: None,
                color: (180, 120, 100),
                speaking: true,
                muted: false,
                is_self: false,
            },
        ];
        members.retain(|m| !preview.speakers_only || m.speaking);
        let display = windows::display_bounds(cx).size;
        let layout = roster_layout(
            &members,
            &preview,
            (display.width.into(), display.height.into()),
            preview.margin as f32,
        );
        let preview_panel = card().items_center().bg(rgb(theme::BORDER)).child(
            div()
                .w(layout.size.width)
                .max_w_full()
                .h(layout.size.height)
                .child(roster_panel(&layout, false, &preview, false, cx)),
        );
        let style = &self.config.appearance;
        let mut roster = card();
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
            ("Speakers only", style.speakers_only, |s, v| {
                s.speakers_only = v
            }),
            ("Active speakers first", style.speakers_first, |s, v| {
                s.speakers_first = v
            }),
        ] {
            roster = roster.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(name)
                    .child(Switch::new(name).checked(value).on_click(cx.listener(
                        move |this, value, _, cx| {
                            setter(&mut this.config.appearance, *value);
                            cx.notify();
                        },
                    ))),
            );
        }

        if style.speakers_only {
            roster = roster.child(number_preference(
                "Keep after speaking (seconds)",
                style.speaker_hold_seconds,
                0..=10,
                cx,
                |s, v| s.config.appearance.speaker_hold_seconds = v,
            ));
        }

        let mut palette = card().child(note("Enter colors in #RRGGBB format."));
        for (input, name) in self
            .colors
            .iter()
            .zip(["Background", "Text", "Speaking highlight"])
        {
            let value = input.read(cx).value();
            let color = parse_color(value.trim()).map(|(r, g, b)| gpui::Rgba {
                r: r as f32 / 255.,
                g: g as f32 / 255.,
                b: b as f32 / 255.,
                a: 1.,
            });
            palette = palette.child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .size(px(18.))
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(theme::BORDER))
                            .when_some(color, |d, c| d.bg(c)),
                    )
                    .child(div().flex_1().child(name))
                    .child(div().w(px(124.)).child(Input::new(input))),
            );
        }
        let mut dimensions = card();
        for ((name, _, _), slider) in RANGES.iter().zip(&self.sliders) {
            dimensions = dimensions.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div().flex().justify_between().child(*name).child(
                            note(format!("{}", slider.read(cx).value().start() as u16))
                                .text_color(rgb(theme::ACCENT)),
                        ),
                    )
                    .child(Slider::new(slider)),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(group("Roster preview", preview_panel))
            .child(group("Colors", palette))
            .child(group("Size and spacing", dimensions))
            .child(group("Roster details", roster))
            .child(
                Button::new("reset")
                    .label("Reset appearance")
                    .on_click(cx.listener(Self::reset)),
            )
    }

    fn updates(&self, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(group(
                "Installed version",
                card().child(crate::updater::panel(cx)),
            ))
            .child(group(
                "Preferences",
                card().child(preference_row(
                    "Automatically check for updates",
                    "Check GitHub every six hours. You choose when to download and restart.",
                    Switch::new("automatic-update-checks")
                        .checked(self.config.automatic_update_checks)
                        .on_click(cx.listener(|this, value, _, cx| {
                            this.config.automatic_update_checks = *value;
                            cx.notify();
                        })),
                )),
            ))
    }
}

impl Render for Settings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut navigation = div().flex().flex_col().gap_1();
        for section in [
            Section::General,
            Section::Overlay,
            Section::Appearance,
            Section::Updates,
        ] {
            let active = self.section == section;
            let tint = if active { theme::ACCENT } else { theme::MUTED };
            let button_style = if active {
                ButtonCustomVariant::new(cx)
                    .color(rgb(0x39302e).into())
                    .hover(rgb(0x514039).into())
                    .active(rgb(0x60483e).into())
            } else {
                ButtonCustomVariant::new(cx)
                    .hover(rgb(theme::BORDER).into())
                    .active(rgb(0x41414c).into())
            };
            navigation = navigation.child(
                Button::new(section.title())
                    .custom(button_style)
                    .w_full()
                    .h(px(38.0))
                    .px_3()
                    .justify_start()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .text_color(rgb(tint))
                            .when(active, |row| row.font_weight(FontWeight::SEMIBOLD))
                            .child(
                                svg()
                                    .path(section.icon())
                                    .size(px(18.0))
                                    .text_color(rgb(tint)),
                            )
                            .child(section.title()),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.end_recording(cx);
                        this.section = section;
                        cx.notify();
                    })),
            );
        }
        let page = match self.section {
            Section::General => self.general(cx),
            Section::Overlay => self.overlay(cx),
            Section::Appearance => self.appearance_page(cx),
            Section::Updates => self.updates(cx),
        };
        let running = cx.global::<Desktop>().overlay.is_some();
        div()
            .track_focus(&self.recorder_focus)
            .size_full()
            .flex()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::TEXT))
            .child(
                div()
                    .w(px(184.0))
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .px_3()
                    .pt(px(48.))
                    .pb_4()
                    .bg(rgb(theme::SURFACE))
                    .border_r_1()
                    .border_color(rgb(theme::BORDER))
                    .child(
                        div().px_3().pb_6().child(
                            svg()
                                .path("wordmark.svg")
                                .w(px(126.0))
                                .h(px(28.))
                                .text_color(rgb(theme::ACCENT)),
                        ),
                    )
                    .child(navigation)
                    .child(div().flex_1())
                    .child(
                        div()
                            .border_t_1()
                            .border_color(rgb(theme::BORDER))
                            .pt_3()
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(note("Show controls").text_xs())
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(theme::BORDER))
                                    .text_xs()
                                    .text_color(rgb(theme::MUTED))
                                    .child(self.config.shortcuts.controls.clone()),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .pt(px(48.))
                    .child(
                        div()
                            .px_6()
                            .pb_5()
                            .text_size(px(27.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.section.title()),
                    )
                    .child(
                        div()
                            .id(("settings-page", self.section as usize))
                            .min_h_0()
                            .flex_1()
                            .overflow_y_scroll()
                            .child(div().px_6().pb_6().child(page)),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap_3()
                            .px_6()
                            .py_3()
                            .border_t_1()
                            .border_color(rgb(theme::BORDER))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(note(self.error.clone()).text_color(rgb(0xff8c8c))),
                            )
                            .child(
                                Button::new("save")
                                    .disabled(self.recording.is_some())
                                    .primary()
                                    .label(if running { "Save" } else { "Start" })
                                    .on_click(cx.listener(Self::save)),
                            ),
                    ),
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
