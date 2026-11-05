//! Role: Expose the spacing, typography, motion and color tokens of Astryx Neutral.
use crate::color::Colors;
use crate::shadows::Shadows;
use kivact::motion::Easing;
use kivact::style::Theme;
use kivact::style::ThemeExtension;
use kivact::style::ThemeMode;

/// Astryx Neutral tokens resolved for a theme mode.
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    pub colors: Colors,
    pub typography: TypeTokens,
    pub spacing: SpaceTokens,
    pub radius: RadiusTokens,
    pub sizes: Sizes,
    pub shadows: Shadows,
    pub motion: MotionTokens,
}

impl Tokens {
    /// The Neutral token values for the selected appearance.
    pub fn new(mode: ThemeMode) -> Self {
        let dark = mode == ThemeMode::Dark;
        Self {
            colors: if dark {
                Colors::dark()
            } else {
                Colors::light()
            },
            typography: TypeTokens::NEUTRAL,
            spacing: SpaceTokens::NEUTRAL,
            radius: RadiusTokens::NEUTRAL,
            sizes: Sizes::NEUTRAL,
            shadows: Shadows::new(dark),
            motion: MotionTokens::NEUTRAL,
        }
    }
}

impl ThemeExtension for Tokens {
    fn fallback(theme: &Theme) -> Self {
        Self::new(theme.mode())
    }
}

/// A typography role's size, line height, weight and family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    pub size_px: f64,
    pub line_height_px: f64,
    pub weight: u16,
    pub family: &'static str,
}

/// Neutral font roles and size scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeTokens {
    pub family_body: &'static str,
    pub family_heading: &'static str,
    pub family_code: &'static str,
    pub weight_normal: u16,
    pub weight_medium: u16,
    pub weight_semibold: u16,
    pub weight_bold: u16,
    pub size_4xs: f64,
    pub size_3xs: f64,
    pub size_2xs: f64,
    pub size_xs: f64,
    pub size_sm: f64,
    pub size_md: f64,
    pub size_lg: f64,
    pub size_xl: f64,
    pub size_2xl: f64,
    pub size_3xl: f64,
    pub size_4xl: f64,
    pub size_5xl: f64,
    pub display1: TypeStyle,
    pub display2: TypeStyle,
    pub display3: TypeStyle,
    pub heading1: TypeStyle,
    pub heading2: TypeStyle,
    pub heading3: TypeStyle,
    pub heading4: TypeStyle,
    pub heading5: TypeStyle,
    pub heading6: TypeStyle,
    pub large: TypeStyle,
    pub body: TypeStyle,
    pub label: TypeStyle,
    pub code: TypeStyle,
    pub supporting: TypeStyle,
    pub letter_spacing: f64,
}

impl TypeTokens {
    const NEUTRAL: Self = Self {
        family_body: "Figtree, sans-serif",
        family_heading: "Figtree, sans-serif",
        family_code: "ui-monospace, SF Mono, Monaco, Consolas, Liberation Mono, Courier New, monospace",
        weight_normal: 400,
        weight_medium: 500,
        weight_semibold: 600,
        weight_bold: 700,
        size_4xs: 6.0,
        size_3xs: 7.0,
        size_2xs: 8.0,
        size_xs: 10.0,
        size_sm: 12.0,
        size_md: 14.0,
        size_lg: 17.0,
        size_xl: 20.0,
        size_2xl: 24.0,
        size_3xl: 29.0,
        size_4xl: 35.0,
        size_5xl: 42.0,
        display1: TypeStyle {
            size_px: 42.0,
            line_height_px: 52.0,
            weight: 400,
            family: "Figtree, sans-serif",
        },
        display2: TypeStyle {
            size_px: 35.0,
            line_height_px: 44.0,
            weight: 400,
            family: "Figtree, sans-serif",
        },
        display3: TypeStyle {
            size_px: 29.0,
            line_height_px: 40.0,
            weight: 400,
            family: "Figtree, sans-serif",
        },
        heading1: TypeStyle {
            size_px: 24.0,
            line_height_px: 32.0,
            weight: 600,
            family: "Figtree, sans-serif",
        },
        heading2: TypeStyle {
            size_px: 20.0,
            line_height_px: 28.0,
            weight: 600,
            family: "Figtree, sans-serif",
        },
        heading3: TypeStyle {
            size_px: 17.0,
            line_height_px: 24.0,
            weight: 700,
            family: "Figtree, sans-serif",
        },
        heading4: TypeStyle {
            size_px: 14.0,
            line_height_px: 20.0,
            weight: 700,
            family: "Figtree, sans-serif",
        },
        heading5: TypeStyle {
            size_px: 12.0,
            line_height_px: 20.0,
            weight: 600,
            family: "Figtree, sans-serif",
        },
        heading6: TypeStyle {
            size_px: 10.0,
            line_height_px: 16.0,
            weight: 600,
            family: "Figtree, sans-serif",
        },
        large: TypeStyle {
            size_px: 17.0,
            line_height_px: 24.0,
            weight: 600,
            family: "Figtree, sans-serif",
        },
        body: TypeStyle {
            size_px: 14.0,
            line_height_px: 20.0,
            weight: 400,
            family: "Figtree, sans-serif",
        },
        label: TypeStyle {
            size_px: 14.0,
            line_height_px: 20.0,
            weight: 500,
            family: "Figtree, sans-serif",
        },
        code: TypeStyle {
            size_px: 14.0,
            line_height_px: 20.0,
            weight: 400,
            family: "ui-monospace, SF Mono, Monaco, Consolas, Liberation Mono, Courier New, monospace",
        },
        supporting: TypeStyle {
            size_px: 12.0,
            line_height_px: 20.0,
            weight: 400,
            family: "Figtree, sans-serif",
        },
        letter_spacing: 0.0,
    };
}

/// Neutral spacing steps in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpaceTokens {
    pub s0: f64,
    pub s0_5: f64,
    pub s1: f64,
    pub s1_5: f64,
    pub s2: f64,
    pub s3: f64,
    pub s4: f64,
    pub s5: f64,
    pub s6: f64,
    pub s7: f64,
    pub s8: f64,
    pub s9: f64,
    pub s10: f64,
    pub s11: f64,
    pub s12: f64,
    pub card_padding: f64,
    pub section_padding: f64,
    pub dialog_padding: f64,
    pub layer_viewport_gutter: f64,
}

impl SpaceTokens {
    const NEUTRAL: Self = Self {
        s0: 0.0,
        s0_5: 2.0,
        s1: 4.0,
        s1_5: 6.0,
        s2: 8.0,
        s3: 12.0,
        s4: 16.0,
        s5: 20.0,
        s6: 24.0,
        s7: 28.0,
        s8: 32.0,
        s9: 36.0,
        s10: 40.0,
        s11: 44.0,
        s12: 48.0,
        card_padding: 12.0,
        section_padding: 12.0,
        dialog_padding: 16.0,
        layer_viewport_gutter: 16.0,
    };
}

/// Neutral corner radii in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadiusTokens {
    pub none: f64,
    pub inner: f64,
    pub element: f64,
    pub container: f64,
    pub page: f64,
    pub chat: f64,
    pub full: f64,
}

impl RadiusTokens {
    const NEUTRAL: Self = Self {
        none: 0.0,
        inner: 6.0,
        element: 10.0,
        container: 12.0,
        page: 28.0,
        chat: 28.0,
        full: 9999.0,
    };
}

/// Neutral control metrics and state amounts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizes {
    pub control_xs: f64,
    pub control_sm: f64,
    pub control_md: f64,
    pub control_lg: f64,
    pub border_width: f64,
    pub focus_ring_width: f64,
    pub focus_ring_offset: f64,
    pub icon_xsm: f64,
    pub icon_sm: f64,
    pub icon_md: f64,
    pub icon_lg: f64,
    pub icon_stroke_ratio: f64,
    pub badge_height: f64,
    pub min_touch_target: f64,
    pub disabled_opacity: f64,
    pub press_scale: f64,
}

impl Sizes {
    const NEUTRAL: Self = Self {
        control_xs: 24.0,
        control_sm: 28.0,
        control_md: 32.0,
        control_lg: 36.0,
        border_width: 1.0,
        focus_ring_width: 2.0,
        focus_ring_offset: 3.0,
        icon_xsm: 12.0,
        icon_sm: 16.0,
        icon_md: 20.0,
        icon_lg: 24.0,
        icon_stroke_ratio: 2.0 / 24.0,
        badge_height: 20.0,
        min_touch_target: 44.0,
        disabled_opacity: 0.5,
        press_scale: 0.98,
    };
}

const STANDARD: Easing = Easing::CubicBezier(0.24, 1.0, 0.4, 1.0);

/// Neutral motion durations in milliseconds and easing curves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTokens {
    pub fast_min: u64,
    pub fast: u64,
    pub fast_max: u64,
    pub medium_min: u64,
    pub medium: u64,
    pub medium_max: u64,
    pub slow_min: u64,
    pub slow: u64,
    pub slow_max: u64,
    pub ease_standard: Easing,
    pub ease_entrance: Easing,
    pub ease_move: Easing,
    pub ease_exit: Easing,
    pub press_onset: u64,
    pub press_release: u64,
    pub ease_press_release: Easing,
}

impl MotionTokens {
    const NEUTRAL: Self = Self {
        fast_min: 95,
        fast: 125,
        fast_max: 165,
        medium_min: 225,
        medium: 300,
        medium_max: 400,
        slow_min: 525,
        slow: 700,
        slow_max: 935,
        ease_standard: STANDARD,
        ease_entrance: STANDARD,
        ease_move: STANDARD,
        ease_exit: Easing::CubicBezier(0.3, 0.0, 0.6, 0.6),
        press_onset: 150,
        press_release: 200,
        ease_press_release: Easing::CubicBezier(0.0, 0.0, 0.2, 1.0),
    };
}
