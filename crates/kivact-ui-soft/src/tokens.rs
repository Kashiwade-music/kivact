//! Role: Expose the spacing, typography, motion and color tokens of Astryx Neutral.
use crate::color::Colors;
use crate::shadows::Shadows;
use kivact::draw::Shadow;
use kivact::motion::Easing;
use kivact::style::Theme;
use kivact::style::ThemeExtension;
use kivact::style::ThemeMode;
use kivact::style::{rgb, rgba};

/// Astryx Neutral tokens resolved for a theme mode.
///
/// Install these as a theme extension and read current values with
/// `kivact::style::use_tokens::<Tokens>()` inside reactive styling closures.
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    /// Appearance-dependent surface, foreground and semantic palettes.
    pub colors: Colors,
    /// Font families, weights, sizes and typography roles.
    pub typography: TypeTokens,
    /// Logical-pixel spacing scale and container padding.
    pub spacing: SpaceTokens,
    /// Logical-pixel corner radii for controls and surfaces.
    pub radius: RadiusTokens,
    /// Control dimensions, icon metrics and interaction-state amounts.
    pub sizes: Sizes,
    /// Appearance-dependent elevation, inset and input-outline shadows.
    pub shadows: Shadows,
    /// Transition durations in milliseconds and easing curves.
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
    /// Font size in logical pixels.
    pub size_px: f64,
    /// Line height in logical pixels, rather than a font-size multiplier.
    pub line_height_px: f64,
    /// Numeric font weight, such as 400 for regular or 600 for semibold.
    pub weight: u16,
    /// Font-family fallback list used to shape this typography role.
    pub family: &'static str,
}

/// Neutral font roles and size scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeTokens {
    /// Body-text font-family fallback list; defaults to Figtree and sans-serif.
    pub family_body: &'static str,
    /// Heading font-family fallback list; defaults to Figtree and sans-serif.
    pub family_heading: &'static str,
    /// Monospace font-family fallback list for source code.
    pub family_code: &'static str,
    /// Numeric font weight for normal text; defaults to 400.
    pub weight_normal: u16,
    /// Numeric font weight for medium text; defaults to 500.
    pub weight_medium: u16,
    /// Numeric font weight for semibold text; defaults to 600.
    pub weight_semibold: u16,
    /// Numeric font weight for bold text; defaults to 700.
    pub weight_bold: u16,
    /// 4XS font-size step in logical pixels; defaults to 6.
    pub size_4xs: f64,
    /// 3XS font-size step in logical pixels; defaults to 7.
    pub size_3xs: f64,
    /// 2XS font-size step in logical pixels; defaults to 8.
    pub size_2xs: f64,
    /// XS font-size step in logical pixels; defaults to 10.
    pub size_xs: f64,
    /// SM font-size step in logical pixels; defaults to 12.
    pub size_sm: f64,
    /// MD font-size step in logical pixels; defaults to 14.
    pub size_md: f64,
    /// LG font-size step in logical pixels; defaults to 17.
    pub size_lg: f64,
    /// XL font-size step in logical pixels; defaults to 20.
    pub size_xl: f64,
    /// 2XL font-size step in logical pixels; defaults to 24.
    pub size_2xl: f64,
    /// 3XL font-size step in logical pixels; defaults to 29.
    pub size_3xl: f64,
    /// 4XL font-size step in logical pixels; defaults to 35.
    pub size_4xl: f64,
    /// 5XL font-size step in logical pixels; defaults to 42.
    pub size_5xl: f64,
    /// Typography for display scale 1, independent of semantic heading level.
    pub display1: TypeStyle,
    /// Typography for display scale 2, independent of semantic heading level.
    pub display2: TypeStyle,
    /// Typography for display scale 3, independent of semantic heading level.
    pub display3: TypeStyle,
    /// Typography for semantic level-1 headings.
    pub heading1: TypeStyle,
    /// Typography for semantic level-2 headings.
    pub heading2: TypeStyle,
    /// Typography for semantic level-3 headings.
    pub heading3: TypeStyle,
    /// Typography for semantic level-4 headings.
    pub heading4: TypeStyle,
    /// Typography for semantic level-5 headings.
    pub heading5: TypeStyle,
    /// Typography for semantic level-6 headings.
    pub heading6: TypeStyle,
    /// Typography for large body text.
    pub large: TypeStyle,
    /// Typography for ordinary body text.
    pub body: TypeStyle,
    /// Typography for control labels.
    pub label: TypeStyle,
    /// Typography for inline and block source code.
    pub code: TypeStyle,
    /// Typography for supporting descriptions and secondary text.
    pub supporting: TypeStyle,
    /// Additional character spacing in logical pixels; defaults to zero.
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
    /// Spacing step 0 in logical pixels; defaults to 0.
    pub s0: f64,
    /// Spacing step 0.5 in logical pixels; defaults to 2.
    pub s0_5: f64,
    /// Spacing step 1 in logical pixels; defaults to 4.
    pub s1: f64,
    /// Spacing step 1.5 in logical pixels; defaults to 6.
    pub s1_5: f64,
    /// Spacing step 2 in logical pixels; defaults to 8.
    pub s2: f64,
    /// Spacing step 3 in logical pixels; defaults to 12.
    pub s3: f64,
    /// Spacing step 4 in logical pixels; defaults to 16.
    pub s4: f64,
    /// Spacing step 5 in logical pixels; defaults to 20.
    pub s5: f64,
    /// Spacing step 6 in logical pixels; defaults to 24.
    pub s6: f64,
    /// Spacing step 7 in logical pixels; defaults to 28.
    pub s7: f64,
    /// Spacing step 8 in logical pixels; defaults to 32.
    pub s8: f64,
    /// Spacing step 9 in logical pixels; defaults to 36.
    pub s9: f64,
    /// Spacing step 10 in logical pixels; defaults to 40.
    pub s10: f64,
    /// Spacing step 11 in logical pixels; defaults to 44.
    pub s11: f64,
    /// Spacing step 12 in logical pixels; defaults to 48.
    pub s12: f64,
    /// Inner padding of cards in logical pixels; defaults to 12.
    pub card_padding: f64,
    /// Content inset of sections in logical pixels; defaults to 12.
    pub section_padding: f64,
    /// Inner padding of dialogs in logical pixels; defaults to 16.
    pub dialog_padding: f64,
    /// Minimum overlay-to-viewport inset in logical pixels; defaults to 16.
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
    /// Square corners: zero logical-pixel radius.
    pub none: f64,
    /// Radius for nested inner surfaces; defaults to 6 logical pixels.
    pub inner: f64,
    /// Radius for controls and small elements; defaults to 10 logical pixels.
    pub element: f64,
    /// Radius for containing surfaces; defaults to 12 logical pixels.
    pub container: f64,
    /// Radius for page-scale surfaces; defaults to 28 logical pixels.
    pub page: f64,
    /// Radius for chat surfaces; defaults to 28 logical pixels.
    pub chat: f64,
    /// Pill or circular rounding radius; defaults to 9999 logical pixels.
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
    /// Extra-small control height; defaults to 24 logical pixels.
    pub control_xs: f64,
    /// Small control height; defaults to 28 logical pixels.
    pub control_sm: f64,
    /// Medium control height; defaults to 32 logical pixels.
    pub control_md: f64,
    /// Large control height; defaults to 36 logical pixels.
    pub control_lg: f64,
    /// Default outline thickness; defaults to 1 logical pixel.
    pub border_width: f64,
    /// Focus-outline thickness; defaults to 2 logical pixels.
    pub focus_ring_width: f64,
    /// Gap between a control and its focus outline; defaults to 3 logical pixels.
    pub focus_ring_offset: f64,
    /// Extra-small icon side length; defaults to 12 logical pixels.
    pub icon_xsm: f64,
    /// Small icon side length; defaults to 16 logical pixels.
    pub icon_sm: f64,
    /// Medium icon side length; defaults to 20 logical pixels.
    pub icon_md: f64,
    /// Large icon side length; defaults to 24 logical pixels.
    pub icon_lg: f64,
    /// Icon stroke width as a fraction of its side length; defaults to 2/24.
    pub icon_stroke_ratio: f64,
    /// Default badge height; defaults to 20 logical pixels.
    pub badge_height: f64,
    /// Minimum touch-target side length; defaults to 44 logical pixels.
    pub min_touch_target: f64,
    /// Opacity multiplier for disabled controls; defaults to 0.5.
    pub disabled_opacity: f64,
    /// Scale multiplier during a press; defaults to 0.98.
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
#[derive(Clone, Debug, PartialEq)]
pub struct MotionTokens {
    /// Shortest fast-transition duration in milliseconds; defaults to 95.
    pub fast_min: u64,
    /// Standard fast-transition duration in milliseconds; defaults to 125.
    pub fast: u64,
    /// Longest fast-transition duration in milliseconds; defaults to 165.
    pub fast_max: u64,
    /// Shortest medium-transition duration in milliseconds; defaults to 225.
    pub medium_min: u64,
    /// Standard medium-transition duration in milliseconds; defaults to 300.
    pub medium: u64,
    /// Longest medium-transition duration in milliseconds; defaults to 400.
    pub medium_max: u64,
    /// Shortest slow-transition duration in milliseconds; defaults to 525.
    pub slow_min: u64,
    /// Standard slow-transition duration in milliseconds; defaults to 700.
    pub slow: u64,
    /// Longest slow-transition duration in milliseconds; defaults to 935.
    pub slow_max: u64,
    /// Default easing for ordinary state transitions.
    pub ease_standard: Easing,
    /// Easing used when content enters.
    pub ease_entrance: Easing,
    /// Easing used when content changes position.
    pub ease_move: Easing,
    /// Easing used when content leaves.
    pub ease_exit: Easing,
    /// Press feedback onset duration in milliseconds; defaults to 150.
    pub press_onset: u64,
    /// Press feedback release duration in milliseconds; defaults to 200.
    pub press_release: u64,
    /// Decelerating easing for the release of press feedback.
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

impl Shadows {
    pub(super) fn new(dark: bool) -> Self {
        let outside = |y, blur, alpha| Shadow::new(0, y, blur, rgba(0, alpha));
        let inset = |spread, color| Shadow::new(0, 0, 0, color).spread(spread).inset();
        let (low, med, high) = if dark {
            (
                vec![
                    outside(2, 4, 64.0 / 255.0),
                    outside(4, 8, 102.0 / 255.0),
                    inset(1, rgba(0xffffff, 20.0 / 255.0)),
                ],
                vec![
                    outside(2, 4, 89.0 / 255.0),
                    outside(4, 12, 128.0 / 255.0),
                    inset(1, rgba(0xffffff, 31.0 / 255.0)),
                ],
                vec![
                    outside(4, 6, 128.0 / 255.0),
                    outside(12, 24, 179.0 / 255.0),
                    inset(1, rgba(0xffffff, 38.0 / 255.0)),
                ],
            )
        } else {
            (
                vec![outside(2, 4, 13.0 / 255.0), outside(4, 8, 26.0 / 255.0)],
                vec![outside(2, 4, 13.0 / 255.0), outside(4, 12, 26.0 / 255.0)],
                vec![outside(4, 6, 26.0 / 255.0), outside(12, 24, 38.0 / 255.0)],
            )
        };
        Self {
            low,
            med,
            high,
            inset_hover: inset(2, rgba(0x0074e2, 77.0 / 255.0)),
            inset_selected: inset(2, rgba(0x0074e2, 128.0 / 255.0)),
            inset_success: inset(2, rgba(0x2f7d33, 77.0 / 255.0)),
            inset_warning: inset(2, rgba(0xf8d36a, 77.0 / 255.0)),
            inset_error: inset(2, rgba(0xde4745, 77.0 / 255.0)),
            input_hover_ring: inset(2, rgba(if dark { 0x525252 } else { 0xd4d4d4 }, 0.3)),
            input_focus_ring: inset(2, rgb(if dark { 0x262626 } else { 0xf1f1f1 })),
        }
    }
}
