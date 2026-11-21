use crate::{
    components::{
        controls::{Controls, HEIGHT as CONTROLS_HEIGHT, WIDTH as CONTROLS_WIDTH},
        overlay::Overlay,
        settings::Settings,
    },
    state::app_state,
};
use gpui::*;
use gpui_component::Root;

#[derive(Default)]
pub struct Desktop {
    last_toggle: Option<std::time::Instant>,
    pub overlay: Option<AnyWindowHandle>,
    pub controls: Option<AnyWindowHandle>,
    pub settings: Option<AnyWindowHandle>,
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

pub fn open_settings(cx: &mut App) {
    cx.activate(true);
    if let Some(handle) = cx.global::<Desktop>().settings
        && handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        return;
    }
    match cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(520.0), px(780.0)),
                cx,
            ))),
            window_min_size: Some(size(px(460.0), px(600.0))),
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
            cx.activate(true);
        }
        Err(error) => eprintln!("Cannot open settings: {error}"),
    }
}

fn overlay_options(bounds: Bounds<Pixels>, focus: bool, cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        display_id: selected_display(cx).map(|display| display.id()),
        titlebar: None,
        kind: WindowKind::Normal,
        window_background: WindowBackgroundAppearance::Transparent,
        is_resizable: false,
        is_movable: false,
        is_minimizable: false,
        focus,
        ..Default::default()
    }
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
    let controls = cx.global_mut::<Desktop>().controls.take();
    if let Some(old) = controls {
        let _ = old.update(cx, |_, window, _| window.remove_window());
        open_controls(cx)?;
    }
    Ok(())
}

pub fn open_controls(cx: &mut App) -> anyhow::Result<()> {
    let display = display_bounds(cx);
    let bounds = Bounds::new(
        point(
            display.origin.x + (display.size.width - px(CONTROLS_WIDTH)) / 2.0,
            display.origin.y + px(48.0),
        ),
        size(px(CONTROLS_WIDTH), px(CONTROLS_HEIGHT)),
    );
    let handle = cx.open_window(overlay_options(bounds, true, cx), |window, cx| {
        window.set_window_title("Yaplay Controls");
        configure_overlay(window, false);
        let view = cx.new(|_| Controls);
        cx.new(|cx| Root::new(view, window, cx))
    })?;
    cx.global_mut::<Desktop>().controls = Some(handle.into());
    Ok(())
}

pub fn toggle_controls(cx: &mut App) {
    let desktop = cx.global_mut::<Desktop>();
    // A focused app can receive both the native hotkey and the GPUI key binding.
    if desktop
        .last_toggle
        .is_some_and(|last| last.elapsed() < std::time::Duration::from_millis(200))
    {
        return;
    }
    desktop.last_toggle = Some(std::time::Instant::now());
    if cx.global::<Desktop>().overlay.is_none() {
        return;
    }
    if let Some(handle) = cx.global_mut::<Desktop>().controls.take() {
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    } else if let Err(error) = open_controls(cx) {
        eprintln!("Cannot open controls: {error}");
    }
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
            style |= WS_EX_TOOLWINDOW as isize;
            if click_through {
                style |= (WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE) as isize;
            }
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
