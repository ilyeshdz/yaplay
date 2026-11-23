use gpui::*;
use gpui_component::{Theme, ThemeMode};

pub const BACKGROUND: u32 = 0x141418;
pub const SURFACE: u32 = 0x1c1c22;
pub const BORDER: u32 = 0x34343e;
pub const TEXT: u32 = 0xf3f1f0;
pub const MUTED: u32 = 0xb4b1ba;
pub const ACCENT: u32 = 0xffb49b;

pub fn init(cx: &mut App) {
    gpui_component::init(cx);
    Theme::change(ThemeMode::Dark, None, cx);
    let theme = Theme::global_mut(cx);
    theme.background = rgba(0x00000000).into();
    theme.foreground = rgb(TEXT).into();
    theme.primary = rgb(ACCENT).into();
    theme.primary_foreground = rgb(BACKGROUND).into();
    theme.secondary = rgb(SURFACE).into();
    theme.secondary_foreground = rgb(TEXT).into();
    theme.border = rgb(BORDER).into();
    theme.input = rgb(BORDER).into();
    theme.ring = rgb(ACCENT).into();
    theme.accent = rgb(0x24242c).into();
    theme.accent_foreground = rgb(TEXT).into();
    theme.muted = rgb(SURFACE).into();
    theme.muted_foreground = rgb(MUTED).into();
    theme.popover = rgb(SURFACE).into();
    theme.popover_foreground = rgb(TEXT).into();
    theme.list = rgb(SURFACE).into();
    theme.list_hover = rgb(0x24242c).into();
    theme.list_active = rgb(0x39302e).into();
    theme.list_active_border = rgb(ACCENT).into();
    theme.primary_hover = rgb(0xffc7b5).into();
    theme.primary_active = rgb(0xe99f86).into();
    theme.secondary_hover = rgb(0x24242c).into();
    theme.secondary_active = rgb(BORDER).into();
    theme.slider_bar = rgb(ACCENT).into();
    theme.slider_thumb = rgb(TEXT).into();
    theme.switch = rgb(BORDER).into();
    theme.switch_thumb = rgb(TEXT).into();
    theme.selection = rgba(0xffb49b40).into();
    theme.radius = px(8.0);
    theme.font_size = px(14.0);
}

pub struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        let bytes: &'static [u8] = match path {
            "wordmark.svg" => include_bytes!("../assets/branding/wordmark.svg"),
            "icons/microphone.svg" => include_bytes!("assets/icons/microphone.svg"),
            "icons/microphone-off.svg" => include_bytes!("assets/icons/microphone-off.svg"),
            "icons/headphones.svg" => include_bytes!("assets/icons/headphones.svg"),
            "icons/headphones-off.svg" => include_bytes!("assets/icons/headphones-off.svg"),
            "icons/panels.svg" => include_bytes!("assets/icons/panels.svg"),
            "icons/palette.svg" => include_bytes!("assets/icons/palette.svg"),
            "icons/update.svg" => include_bytes!("assets/icons/update.svg"),
            "icons/plug.svg" => include_bytes!("assets/icons/plug.svg"),
            "icons/settings.svg" => include_bytes!("assets/icons/settings.svg"),
            "icons/phone-off.svg" => include_bytes!("assets/icons/phone-off.svg"),
            "icons/chevrons-up-down.svg" => include_bytes!("assets/icons/chevrons-up-down.svg"),
            "icons/check.svg" => include_bytes!("assets/icons/check.svg"),
            "icons/exit.svg" => include_bytes!("assets/icons/exit.svg"),
            "icons/move.svg" => include_bytes!("assets/icons/move.svg"),
            _ => return Ok(None),
        };
        Ok(Some(std::borrow::Cow::Borrowed(bytes)))
    }
    fn list(&self, _: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}
