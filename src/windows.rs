use display_info::DisplayInfo;
use freya::prelude::*;
use freya::winit::{
    dpi::LogicalPosition,
    window::WindowLevel,
};

use crate::components::{controls::controls, overlay::overlay};

#[derive(Clone, Copy, Debug)]
pub struct Monitor {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn target_monitor(id: u32) -> Monitor {
    let displays = DisplayInfo::all().unwrap_or_default();
    let display = displays
        .iter()
        .find(|display| display.id == id)
        .or_else(|| displays.iter().find(|display| display.is_primary))
        .or_else(|| displays.first());

    match display {
        Some(display) => Monitor {
            id: display.id,
            x: display.x as f64,
            y: display.y as f64,
            width: display.width as f64,
            height: display.height as f64,
        },
        None => Monitor {
            id: 0,
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        },
    }
}

impl Monitor {
    pub fn bottom_center(&self, width: f64, height: f64, margin: f64) -> (f64, f64) {
        (
            self.x + (self.width - width) / 2.0,
            self.y + self.height - height - margin,
        )
    }
}

pub const CONTROLS_WIDTH: f64 = 460.0;
pub const CONTROLS_HEIGHT: f64 = 76.0;

pub fn overlay_window(monitor: Monitor) -> WindowConfig {
    let (x, y) = (monitor.x, monitor.y);

    WindowConfig::new(overlay)
        .with_title("Yaplay Overlay")
        .with_size(monitor.width, monitor.height)
        .with_decorations(false)
        .with_resizable(false)
        .with_transparency(true)
        .with_background(Color::TRANSPARENT)
        .with_window_attributes(move |attributes, _| {
            attributes
                .with_position(LogicalPosition::new(x, y))
                .with_window_level(WindowLevel::AlwaysOnTop)
                .with_active(false)
        })
        .with_window_handle(|window| {
            let _ = window.set_cursor_hittest(false);
        })
        .with_on_close(|mut context, _| {
            context.exit();
            CloseDecision::Close
        })
}

pub fn controls_window(monitor: Monitor) -> WindowConfig {
    let (x, y) = monitor.bottom_center(CONTROLS_WIDTH, CONTROLS_HEIGHT, 32.0);

    WindowConfig::new(controls)
        .with_title("Yaplay Controls")
        .with_size(CONTROLS_WIDTH, CONTROLS_HEIGHT)
        .with_decorations(false)
        .with_resizable(false)
        .with_transparency(true)
        .with_background(Color::TRANSPARENT)
        .with_window_attributes(move |attributes, _| {
            attributes
                .with_position(LogicalPosition::new(x, y))
                .with_window_level(WindowLevel::AlwaysOnTop)
        })
        .with_window_handle(|window| {
            let _ = window.set_cursor_hittest(false);
        })
        .with_on_close(|_, _| CloseDecision::KeepOpen)
}
