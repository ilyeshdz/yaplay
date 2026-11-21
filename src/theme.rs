//! Yaplay's desktop palette, matching site/app/global.css.
use freya::prelude::*;

pub const BACKGROUND: Color = Color::from_rgb(20, 20, 24);
pub const SURFACE: Color = Color::from_rgb(28, 28, 34);
pub const HOVER: Color = Color::from_rgb(36, 36, 44);
pub const BORDER: Color = Color::from_rgb(52, 52, 62);
pub const TEXT: Color = Color::from_rgb(243, 241, 240);
pub const MUTED: Color = Color::from_rgb(180, 177, 186);
pub const ACCENT: Color = Color::from_rgb(255, 180, 155);
pub const ACCENT_HOVER: Color = Color::from_rgb(255, 199, 181);
pub const OVERLAY_SURFACE: Color = Color::from_argb(235, 28, 28, 34);

pub fn yaplay_theme() -> Theme {
    // Retain all built-in component registrations, then replace the palette.
    // This also prevents the OS accent from replacing Yaplay's peach branding.
    let mut theme = dark_theme();
    theme.name = "yaplay-dark";
    theme.colors = ColorsSheet {
        primary: ACCENT,
        secondary: ACCENT_HOVER,
        tertiary: ACCENT_HOVER,
        background: BACKGROUND,
        surface_primary: BORDER,
        surface_secondary: HOVER,
        surface_tertiary: SURFACE,
        border: BORDER,
        border_focus: ACCENT,
        border_disabled: BORDER,
        text_primary: TEXT,
        text_secondary: MUTED,
        text_placeholder: MUTED,
        text_inverse: BACKGROUND,
        text_highlight: ACCENT,
        focus: ACCENT,
        active: HOVER,
        disabled: SURFACE,
        ..DARK_COLORS
    };
    theme.set(
        "typography",
        TypographyThemePreference {
            title: Preference::Specific(32.0),
            subtitle: Preference::Specific(18.0),
            body: Preference::Specific(14.0),
            caption: Preference::Specific(12.0),
            overline: Preference::Specific(11.0),
        },
    );
    theme.set(
        "button_layout",
        ButtonLayoutThemePreference {
            padding: Preference::Specific(Gaps::new_symmetric(10.0, 16.0)),
            margin: Preference::Specific(Gaps::new_all(0.0)),
            corner_radius: Preference::Specific(CornerRadius::new_all(10.0)),
            width: Preference::Specific(Size::Inner),
            height: Preference::Specific(Size::Inner),
        },
    );
    theme.set(
        "card_layout",
        CardLayoutThemePreference {
            padding: Preference::Specific(Gaps::new_all(20.0)),
            corner_radius: Preference::Specific(CornerRadius::new_all(12.0)),
        },
    );
    theme.set(
        "input_layout",
        InputLayoutThemePreference {
            corner_radius: Preference::Specific(CornerRadius::new_all(10.0)),
            inner_margin: Preference::Specific(Gaps::new_symmetric(10.0, 12.0)),
        },
    );
    theme.set(
        "switch",
        SwitchColorsThemePreference {
            background: Preference::Specific(BORDER),
            thumb_background: Preference::Specific(MUTED),
            toggled_background: Preference::Specific(ACCENT),
            toggled_thumb_background: Preference::Specific(BACKGROUND),
            focus_border_fill: Preference::Specific(ACCENT_HOVER),
        },
    );
    theme
}
