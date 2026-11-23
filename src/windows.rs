use crate::{
    components::{
        controls::{Controls, HEIGHT as CONTROLS_HEIGHT, WIDTH as CONTROLS_WIDTH},
        notifications::{self, Notifications},
        overlay::Overlay,
        roster::Roster,
        settings::Settings,
        voice_list,
    },
    data::{
        appearance::Appearance,
        layout::{Panel, PanelPosition},
    },
    state::{OverlaySnapshot, app_state},
};
use gpui::*;
use gpui_component::Root;
use std::time::{Duration, Instant};

/// Wait for a drag to settle before writing the layout to disk.
const LAYOUT_SAVE_DELAY: Duration = Duration::from_millis(400);

#[derive(Default)]
pub struct Desktop {
    last_toggle: Option<Instant>,
    pub overlay: Option<AnyWindowHandle>,
    pub controls: Option<AnyWindowHandle>,
    pub roster: Option<AnyWindowHandle>,
    pub notifications: Option<AnyWindowHandle>,
    pub settings: Option<AnyWindowHandle>,
    /// Last arrangement mode applied to the panel windows.
    pub arranging: bool,
    layout_changed: Option<Instant>,
}
impl Global for Desktop {}

fn selected_display(cx: &App) -> Option<std::rc::Rc<dyn PlatformDisplay>> {
    let selected = app_state().read().unwrap().config.selected_monitor_id;
    cx.displays()
        .into_iter()
        .find(|d| u32::from(d.id()) == selected)
        .or_else(|| cx.primary_display())
}

pub fn display_bounds(cx: &App) -> Bounds<Pixels> {
    selected_display(cx)
        .map(|d| d.bounds())
        .unwrap_or(Bounds::new(
            point(px(0.0), px(0.0)),
            size(px(1280.0), px(720.0)),
        ))
}

/// The display a panel lives on: its saved display when it still exists,
/// otherwise the display selected in settings.
fn panel_display(cx: &App, panel: Panel) -> Option<std::rc::Rc<dyn PlatformDisplay>> {
    let saved = app_state()
        .read()
        .unwrap()
        .config
        .layout
        .get(panel)
        .map(|position| position.display_id);
    saved
        .and_then(|id| cx.displays().into_iter().find(|d| u32::from(d.id()) == id))
        .or_else(|| selected_display(cx))
}

fn corner_origin(
    position: crate::data::appearance::Corner,
    area: Bounds<Pixels>,
    panel: Size<Pixels>,
    appearance: &Appearance,
) -> Point<Pixels> {
    let margin = px(appearance.margin as f32);
    let x = if position.is_right() {
        area.origin.x + area.size.width - panel.width - margin
    } else {
        area.origin.x + margin
    };
    let y = if position.is_bottom() {
        area.origin.y + area.size.height - panel.height - margin
    } else {
        area.origin.y + margin
    };
    point(x, y)
}

fn default_origin(
    panel: Panel,
    area: Bounds<Pixels>,
    size: Size<Pixels>,
    appearance: &Appearance,
) -> Point<Pixels> {
    match panel {
        Panel::Controls => point(
            area.origin.x + (area.size.width - size.width) / 2.0,
            area.origin.y + px(48.0),
        ),
        Panel::Roster => corner_origin(appearance.position, area, size, appearance),
        Panel::Notifications => {
            corner_origin(appearance.position.opposite_side(), area, size, appearance)
        }
    }
}

fn clamp_origin(area: Bounds<Pixels>, size: Size<Pixels>, origin: Point<Pixels>) -> Point<Pixels> {
    let min_x = f32::from(area.origin.x);
    let min_y = f32::from(area.origin.y);
    let max_x = (min_x + f32::from(area.size.width) - f32::from(size.width)).max(min_x);
    let max_y = (min_y + f32::from(area.size.height) - f32::from(size.height)).max(min_y);
    point(
        px(f32::from(origin.x).clamp(min_x, max_x)),
        px(f32::from(origin.y).clamp(min_y, max_y)),
    )
}

fn panel_bounds(cx: &App, panel: Panel, size: Size<Pixels>) -> Bounds<Pixels> {
    let (layout, appearance) = {
        let shared = app_state();
        let state = shared.read().unwrap();
        (state.config.layout.clone(), state.config.appearance.clone())
    };
    let display = panel_display(cx, panel);
    let Some(display) = display else {
        return Bounds::new(point(px(0.0), px(0.0)), size);
    };
    let area = display.bounds();
    let origin = layout
        .get(panel)
        .map(|saved| {
            point(
                area.origin.x + px(saved.x as f32),
                area.origin.y + px(saved.y as f32),
            )
        })
        .unwrap_or_else(|| default_origin(panel, area, size, &appearance));
    Bounds::new(clamp_origin(area, size, origin), size)
}

fn panel_size(panel: Panel, snapshot: &OverlaySnapshot, display: Size<Pixels>) -> Size<Pixels> {
    let viewport = (f32::from(display.width), f32::from(display.height));
    match panel {
        Panel::Controls => size(px(CONTROLS_WIDTH), px(CONTROLS_HEIGHT)),
        Panel::Roster => {
            voice_list::roster_size(&snapshot.voice_members, &snapshot.appearance, viewport)
        }
        Panel::Notifications => notifications::window_size(
            snapshot.notifications.len(),
            &snapshot.appearance,
            &snapshot.notification_style,
            viewport,
            snapshot.arranging,
        ),
    }
}

fn panel_options(
    bounds: Bounds<Pixels>,
    focus: bool,
    display_id: Option<DisplayId>,
) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        display_id,
        titlebar: None,
        // GPUI creates Windows popups with WS_EX_TOOLWINDOW before showing them.
        kind: if cfg!(target_os = "windows") {
            WindowKind::PopUp
        } else {
            WindowKind::Normal
        },
        window_background: WindowBackgroundAppearance::Transparent,
        is_resizable: false,
        is_movable: false,
        is_minimizable: false,
        focus,
        ..Default::default()
    }
}

#[cfg(target_os = "macos")]
pub fn set_dock_icon_visible(visible: bool) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

    let main_thread = MainThreadMarker::new().expect("GPUI runs on the main thread");
    let app = NSApplication::sharedApplication(main_thread);
    let policy = if visible {
        NSApplicationActivationPolicy::Regular
    } else {
        NSApplicationActivationPolicy::Accessory
    };
    if !app.setActivationPolicy(policy) {
        eprintln!("Cannot change the macOS Dock visibility");
    }
}

pub fn open_settings(cx: &mut App) {
    if let Some(handle) = cx.global::<Desktop>().settings
        && handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        cx.activate(true);
        return;
    }
    match cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(820.0), px(680.0)),
                cx,
            ))),
            window_min_size: Some(size(px(640.0), px(480.0))),
            titlebar: Some(TitlebarOptions {
                title: Some("Yaplay Settings".into()),
                appears_transparent: true,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| {
            let view = cx.new(|cx| Settings::new(window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        },
    ) {
        Ok(handle) => {
            cx.global_mut::<Desktop>().settings = Some(handle.into());
            #[cfg(target_os = "macos")]
            set_dock_icon_visible(true);
            cx.activate(true);
        }
        Err(error) => eprintln!("Cannot open settings: {error}"),
    }
}

fn overlay_options(bounds: Bounds<Pixels>, focus: bool, cx: &App) -> WindowOptions {
    panel_options(
        bounds,
        focus,
        selected_display(cx).map(|display| display.id()),
    )
}

pub fn open_overlay(cx: &mut App) -> anyhow::Result<()> {
    let bounds = display_bounds(cx);
    let old = cx.global::<Desktop>().overlay;
    // Create the replacement before closing the old window, keeping the app alive.
    let handle = cx.open_window(overlay_options(bounds, false, cx), |window, cx| {
        window.set_window_title("Yaplay Overlay");
        configure_overlay(window, true);
        cx.new(|_| Overlay)
    })?;
    cx.global_mut::<Desktop>().overlay = Some(handle.into());
    if let Some(old) = old {
        let _ = old.update(cx, |_, window, _| window.remove_window());
    }
    // Panels are placed on a display. Recreate them so layout and appearance
    // changes take effect immediately.
    let controls_was_open = cx.global::<Desktop>().controls.is_some();
    close_panel(cx, Panel::Controls);
    close_panel(cx, Panel::Roster);
    close_panel(cx, Panel::Notifications);
    if controls_was_open {
        open_controls(cx)?;
    }
    sync(cx);
    Ok(())
}

pub fn open_controls(cx: &mut App) -> anyhow::Result<()> {
    let panel = Panel::Controls;
    let display = panel_display(cx, panel);
    let area = display
        .as_ref()
        .map(|display| display.bounds())
        .unwrap_or_else(|| display_bounds(cx));
    let display_id = display.map(|display| display.id());
    let snapshot = app_state().read().unwrap().snapshot();
    let size = panel_size(panel, &snapshot, area.size);
    let bounds = panel_bounds(cx, panel, size);
    let handle = cx.open_window(panel_options(bounds, true, display_id), |window, cx| {
        window.set_window_title("Yaplay Controls");
        configure_overlay(window, false);
        let view = cx.new(|_| Controls);
        cx.new(|cx| Root::new(view, window, cx))
    })?;
    cx.global_mut::<Desktop>().controls = Some(handle.into());
    Ok(())
}

fn open_roster(cx: &mut App) -> anyhow::Result<()> {
    let panel = Panel::Roster;
    let snapshot = app_state().read().unwrap().snapshot();
    let display = panel_display(cx, panel);
    let area = display
        .as_ref()
        .map(|display| display.bounds())
        .unwrap_or_else(|| display_bounds(cx));
    let display_id = display.map(|display| display.id());
    let size = panel_size(panel, &snapshot, area.size);
    let bounds = panel_bounds(cx, panel, size);
    let arranging = snapshot.arranging;
    let handle = cx.open_window(panel_options(bounds, false, display_id), |window, cx| {
        window.set_window_title("Yaplay Voice");
        configure_overlay(window, !arranging);
        cx.new(|_| Roster)
    })?;
    cx.global_mut::<Desktop>().roster = Some(handle.into());
    Ok(())
}

fn open_notifications(cx: &mut App) -> anyhow::Result<()> {
    let panel = Panel::Notifications;
    let snapshot = app_state().read().unwrap().snapshot();
    let display = panel_display(cx, panel);
    let area = display
        .as_ref()
        .map(|display| display.bounds())
        .unwrap_or_else(|| display_bounds(cx));
    let display_id = display.map(|display| display.id());
    let size = panel_size(panel, &snapshot, area.size);
    let bounds = panel_bounds(cx, panel, size);
    let arranging = snapshot.arranging;
    let handle = cx.open_window(panel_options(bounds, false, display_id), |window, cx| {
        window.set_window_title("Yaplay Notifications");
        configure_overlay(window, !arranging);
        cx.new(|_| Notifications)
    })?;
    cx.global_mut::<Desktop>().notifications = Some(handle.into());
    Ok(())
}

pub fn toggle_controls(cx: &mut App) {
    let desktop = cx.global_mut::<Desktop>();
    // A focused app can receive both the native hotkey and the GPUI key binding.
    if desktop
        .last_toggle
        .is_some_and(|last| last.elapsed() < Duration::from_millis(200))
    {
        return;
    }
    desktop.last_toggle = Some(Instant::now());
    if cx.global::<Desktop>().overlay.is_none() {
        return;
    }
    if let Some(handle) = cx.global_mut::<Desktop>().controls.take() {
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    } else if let Err(error) = open_controls(cx) {
        eprintln!("Cannot open controls: {error}");
    }
}

pub fn toggle_arranging(cx: &mut App) {
    let arranging = {
        let shared = app_state();
        let mut state = shared.write().unwrap();
        state.arranging = !state.arranging;
        state.arranging
    };
    apply_arrangement(cx, arranging);
    sync(cx);
    cx.refresh_windows();
}

pub fn set_arranging(cx: &mut App, arranging: bool) {
    app_state().write().unwrap().arranging = arranging;
    apply_arrangement(cx, arranging);
    sync(cx);
    cx.refresh_windows();
}

fn apply_arrangement(cx: &mut App, arranging: bool) {
    if cx.global::<Desktop>().arranging == arranging {
        return;
    }
    cx.global_mut::<Desktop>().arranging = arranging;
    let handles = {
        let desktop = cx.global::<Desktop>();
        [desktop.roster, desktop.notifications]
    };
    for handle in handles.into_iter().flatten() {
        let _ = handle.update(cx, |_, window, _| configure_overlay(window, !arranging));
    }
}

/// Keep the open panels in step with the application state: display-only
/// panels exist only while they have content or while arranging.
pub fn sync(cx: &mut App) {
    let snapshot = app_state().read().unwrap().snapshot();
    apply_arrangement(cx, snapshot.arranging);
    let desktop = cx.global::<Desktop>();
    let has_controls = desktop.controls.is_some();
    let has_roster = desktop.roster.is_some();
    let has_notifications = desktop.notifications.is_some();
    let want_roster = snapshot.arranging
        || snapshot.in_call
            && (!snapshot.appearance.speakers_only || !snapshot.voice_members.is_empty());
    let want_notifications = !snapshot.notifications.is_empty() || snapshot.arranging;

    if snapshot.arranging
        && !has_controls
        && let Err(error) = open_controls(cx)
    {
        eprintln!("Cannot open controls: {error}");
    }
    if want_roster
        && !has_roster
        && let Err(error) = open_roster(cx)
    {
        eprintln!("Cannot open the voice panel: {error}");
    }
    if !want_roster && has_roster {
        close_panel(cx, Panel::Roster);
    }
    if want_notifications
        && !has_notifications
        && let Err(error) = open_notifications(cx)
    {
        eprintln!("Cannot open notifications: {error}");
    }
    if !want_notifications && has_notifications {
        close_panel(cx, Panel::Notifications);
    }
}

fn close_panel(cx: &mut App, panel: Panel) {
    let handle = {
        let desktop = cx.global_mut::<Desktop>();
        match panel {
            Panel::Controls => desktop.controls.take(),
            Panel::Roster => desktop.roster.take(),
            Panel::Notifications => desktop.notifications.take(),
        }
    };
    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    }
}

/// Read positions back from the platform windows. GPUI has no move event, so
/// the main loop calls this after native drags settle.
pub fn persist_layout(cx: &mut App) {
    let handles: Vec<(Panel, AnyWindowHandle)> = {
        let desktop = cx.global::<Desktop>();
        [
            (Panel::Controls, desktop.controls),
            (Panel::Roster, desktop.roster),
            (Panel::Notifications, desktop.notifications),
        ]
        .into_iter()
        .filter_map(|(panel, handle)| handle.map(|handle| (panel, handle)))
        .collect()
    };
    let mut changed = false;
    for (panel, handle) in handles {
        let Ok(Some(position)) = handle.update(cx, panel_position) else {
            continue;
        };
        let current = app_state().read().unwrap().config.layout.get(panel);
        if current != Some(position) {
            app_state()
                .write()
                .unwrap()
                .config
                .layout
                .set(panel, position);
            changed = true;
        }
    }
    let desktop = cx.global_mut::<Desktop>();
    if changed {
        desktop.layout_changed = Some(Instant::now());
    } else if desktop
        .layout_changed
        .is_some_and(|since| since.elapsed() >= LAYOUT_SAVE_DELAY)
    {
        desktop.layout_changed = None;
        save_config();
    }
}

fn panel_position(_: AnyView, window: &mut Window, cx: &mut App) -> Option<PanelPosition> {
    let WindowBounds::Windowed(bounds) = window.window_bounds() else {
        return None;
    };
    let center = point(
        bounds.origin.x + px(f32::from(bounds.size.width) / 2.0),
        bounds.origin.y + px(f32::from(bounds.size.height) / 2.0),
    );
    let display = cx
        .displays()
        .into_iter()
        .find(|display| display.bounds().contains(&center))
        .or_else(|| selected_display(cx))?;
    let area = display.bounds();
    Some(PanelPosition {
        display_id: u32::from(display.id()),
        x: (f32::from(bounds.origin.x) - f32::from(area.origin.x)).round() as i32,
        y: (f32::from(bounds.origin.y) - f32::from(area.origin.y)).round() as i32,
    })
}

fn save_config() {
    let config = app_state().read().unwrap().config.clone();
    if let Err(error) = config.save() {
        eprintln!("Cannot save the panel layout: {error}");
    }
}

/// Save a layout change immediately, for example before quitting.
pub fn flush_layout(cx: &mut App) {
    persist_layout(cx);
    let pending = cx.global_mut::<Desktop>().layout_changed.take().is_some();
    if pending {
        save_config();
    }
}

/// Start a native window drag for frameless overlay panels.
///
/// GPUI only implements `start_window_move` on Linux, so ask the platform to
/// drag the window with the mouse event it is already handling.
pub fn begin_window_drag(window: &Window) {
    #[cfg(target_os = "macos")]
    unsafe {
        use objc2::{msg_send, runtime::AnyObject};
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let Ok(handle) = HasWindowHandle::window_handle(window) else {
            return;
        };
        if let RawWindowHandle::AppKit(handle) = handle.as_raw() {
            let view = handle.ns_view.as_ptr() as *mut AnyObject;
            let native: *mut AnyObject = msg_send![view, window];
            let app: *mut AnyObject = msg_send![objc2::class!(NSApplication), sharedApplication];
            let event: *mut AnyObject = msg_send![app, currentEvent];
            if !event.is_null() {
                let _: () = msg_send![native, performWindowDragWithEvent: event];
            }
        }
    }
    #[cfg(target_os = "windows")]
    unsafe {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        let Ok(handle) = HasWindowHandle::window_handle(window) else {
            return;
        };
        if let RawWindowHandle::Win32(handle) = handle.as_raw() {
            let hwnd = handle.hwnd.get() as *mut std::ffi::c_void;
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = window;
}

/// GPUI owns window creation and rendering; the native handle is used only for
/// desktop-overlay behavior that GPUI doesn't expose (mouse passthrough).
fn configure_overlay(window: &Window, click_through: bool) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    match handle.as_raw() {
        #[cfg(target_os = "macos")]
        RawWindowHandle::AppKit(handle) => unsafe {
            use objc2::{msg_send, runtime::AnyObject};
            let view = handle.ns_view.as_ptr() as *mut AnyObject;
            let native: *mut AnyObject = msg_send![view, window];
            let _: () = msg_send![native, setIgnoresMouseEvents: click_through];
            let _: () = msg_send![native, setLevel: 3isize];
            let _: () = msg_send![native, setHasShadow: false];
        },
        #[cfg(target_os = "windows")]
        RawWindowHandle::Win32(handle) => unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::*;
            let hwnd = handle.hwnd.get() as *mut std::ffi::c_void;
            let mut style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let passthrough = (WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE) as isize;
            style = if click_through {
                style | passthrough
            } else {
                style & !passthrough
            };
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        },
        _ => {
            let _ = click_through;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Appearance, Panel, clamp_origin, default_origin};
    use crate::data::appearance::Corner;
    use gpui::{Bounds, point, px, size};

    #[test]
    fn saved_positions_are_clamped_onto_the_display() {
        let area = Bounds::new(point(px(0.0), px(0.0)), size(px(1000.0), px(800.0)));
        let panel = size(px(300.0), px(100.0));
        let clamped = clamp_origin(area, panel, point(px(900.0), px(750.0)));
        assert_eq!((f32::from(clamped.x), f32::from(clamped.y)), (700.0, 700.0));
        let negative = clamp_origin(area, panel, point(px(-200.0), px(-40.0)));
        assert_eq!((f32::from(negative.x), f32::from(negative.y)), (0.0, 0.0));
    }

    #[test]
    fn default_positions_follow_the_chosen_corner() {
        let area = Bounds::new(point(px(100.0), px(50.0)), size(px(1000.0), px(800.0)));
        let panel = size(px(200.0), px(100.0));
        let appearance = Appearance {
            margin: 24,
            position: Corner::BottomRight,
            ..Default::default()
        };
        let roster = default_origin(Panel::Roster, area, panel, &appearance);
        assert_eq!((f32::from(roster.x), f32::from(roster.y)), (876.0, 726.0));
        // Notifications use the horizontally opposite corner.
        let notifications = default_origin(Panel::Notifications, area, panel, &appearance);
        assert_eq!(
            (f32::from(notifications.x), f32::from(notifications.y)),
            (124.0, 726.0)
        );
        // Controls stay centered near the top.
        let controls = default_origin(Panel::Controls, area, panel, &appearance);
        assert_eq!(
            (f32::from(controls.x), f32::from(controls.y)),
            (500.0, 98.0)
        );
    }
}
