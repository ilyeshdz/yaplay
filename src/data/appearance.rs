use gpui::{Rgba, rgb, rgba};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub const ALL: [Self; 4] = [
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::TopLeft => "Top left",
            Self::TopRight => "Top right",
            Self::BottomLeft => "Bottom left",
            Self::BottomRight => "Bottom right",
        }
    }
    pub fn is_right(self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }
    pub fn is_bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::BottomRight)
    }
    pub fn opposite_side(self) -> Self {
        match self {
            Self::TopLeft => Self::TopRight,
            Self::TopRight => Self::TopLeft,
            Self::BottomLeft => Self::BottomRight,
            Self::BottomRight => Self::BottomLeft,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub compact: bool,
    pub scale: u16,
    pub width: u16,
    pub opacity: u8,
    pub radius: u8,
    pub margin: u8,
    pub max_members: u8,
    pub position: Corner,
    pub show_avatars: bool,
    pub show_names: bool,
    pub show_header: bool,
    pub show_self: bool,
    pub speakers_first: bool,
    pub background: String,
    pub foreground: String,
    pub accent: String,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            compact: false,
            scale: 100,
            width: 260,
            opacity: 92,
            radius: 12,
            margin: 24,
            max_members: 10,
            position: Corner::TopLeft,
            show_avatars: true,
            show_names: true,
            show_header: true,
            show_self: true,
            speakers_first: true,
            background: "#1c1c22".into(),
            foreground: "#f3f1f0".into(),
            accent: "#ffb49b".into(),
        }
    }
}

impl Appearance {
    pub fn normalize(&mut self) {
        self.scale = self.scale.clamp(75, 150);
        self.width = self.width.clamp(180, 400);
        self.opacity = self.opacity.min(100);
        self.radius = self.radius.min(24);
        self.margin = self.margin.clamp(8, 80);
        self.max_members = self.max_members.clamp(1, 20);
        if !self.show_names && !self.show_avatars {
            self.show_avatars = true;
        }
        let defaults = Self::default();
        for (value, fallback) in [
            (&mut self.background, defaults.background),
            (&mut self.foreground, defaults.foreground),
            (&mut self.accent, defaults.accent),
        ] {
            if parse_color(value).is_none() {
                *value = fallback;
            }
        }
    }
    pub fn factor(&self) -> f32 {
        self.scale as f32 / 100.0
    }
    pub fn row_height(&self) -> f32 {
        (if self.compact { 26.0 } else { 38.0 }) * self.factor()
    }
    pub fn surface(&self) -> Rgba {
        let (r, g, b) = parse_color(&self.background).unwrap_or((28, 28, 34));
        rgba(
            ((r as u32) << 24)
                | ((g as u32) << 16)
                | ((b as u32) << 8)
                | (self.opacity as u32 * 255 / 100),
        )
    }
    pub fn text(&self) -> Rgba {
        let (r, g, b) = parse_color(&self.foreground).unwrap_or((243, 241, 240));
        rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }
    pub fn highlight(&self) -> Rgba {
        let (r, g, b) = parse_color(&self.accent).unwrap_or((255, 180, 155));
        rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }
    pub fn capacity(&self, height: f32) -> usize {
        let overhead = 24.0
            + (if self.show_header { 26.0 } else { 0.0 }) * self.factor()
            + 24.0 * self.factor();
        (((height - 2.0 * self.margin as f32 - overhead).max(0.0) / self.row_height()) as usize)
            .min(self.max_members as usize)
    }
}

pub fn parse_color(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    Some((
        u8::from_str_radix(&hex[..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..], 16).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_values_are_bounded_and_roster_fits_small_screens() {
        let mut style = Appearance {
            scale: 0,
            max_members: 255,
            width: 65535,
            opacity: 255,
            background: "bad".into(),
            show_names: false,
            show_avatars: false,
            ..Default::default()
        };
        style.normalize();
        assert_eq!(
            (style.scale, style.max_members, style.width, style.opacity),
            (75, 20, 400, 100)
        );
        assert!(style.show_avatars);
        assert_eq!(style.background, "#1c1c22");
        assert_eq!(style.capacity(50.0), 0);
        assert!(style.capacity(300.0) < 10);
        assert_eq!(parse_color("#ééé"), None);
    }
}
