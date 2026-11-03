//! Role: Install the Astryx Neutral tokens and recipes on the Kivact theme.
use crate::tokens::Tokens;
use kivact::interaction::{DisabledMenuFocus, SubmenuTiming};
use kivact::motion::Transition;
use kivact::style::theme::{Palette, Radii, Spacing, Typography};
use kivact::style::{LineHeight, Theme, ThemeMode};
use kivact::ui::{
    ButtonBusyMotion, DialogCenter, DisabledButtonFocus, ProgressMotion, ToastRowMotion,
    ToggleLabelReservation,
};
use kivact::view::OverlayViewport;

/// Astryx Neutral in the requested appearance.
///
/// Register Figtree with the application to use the theme's body family.
pub fn theme(mode: ThemeMode) -> Theme {
    let tokens = Tokens::new(mode);
    let colors = tokens.colors;
    let palette = Palette {
        surface: colors.bg_surface,
        surface_alt: colors.bg_muted,
        text: colors.text_primary,
        text_muted: colors.text_secondary,
        accent: colors.accent,
        accent_text: colors.on_accent,
        border: colors.border,
        focus: colors.focus_ring,
        danger: colors.error,
        success: colors.success,
        warning: colors.warning,
        overlay: colors.overlay_scrim,
        shadow: colors.shadow,
        selection: colors.info_muted,
        hover: colors.overlay_hover,
        pressed: colors.overlay_pressed,
        raised: colors.bg_popover,
    };
    let body = tokens.typography.body;
    let text = Typography {
        family: Some(body.family.into()),
        size: body.size_px,
        size_small: tokens.typography.supporting.size_px,
        size_large: tokens.typography.large.size_px,
        weight: body.weight,
        weight_strong: tokens.typography.weight_semibold,
        line_height: LineHeight::Scale(body.line_height_px / body.size_px),
    };
    Theme::light()
        .with_mode(mode)
        .with_colors(palette)
        .with_space(Spacing {
            xs: tokens.spacing.s1,
            sm: tokens.spacing.s2,
            md: tokens.spacing.s3,
            lg: tokens.spacing.s4,
            xl: tokens.spacing.s6,
        })
        .with_radius(Radii {
            sm: tokens.radius.inner,
            md: tokens.radius.element,
            lg: tokens.radius.container,
        })
        .with_text(text)
        .with_focus_ring(tokens.sizes.focus_ring_width)
        .with_focus_ring_offset(tokens.sizes.focus_ring_offset)
        .with_control_height(tokens.sizes.control_md)
        .with_disabled_opacity(tokens.sizes.disabled_opacity)
        .with(OverlayViewport::new(tokens.spacing.layer_viewport_gutter))
        .with(DisabledButtonFocus::Retain)
        .with(DisabledMenuFocus::Retain)
        .with(DialogCenter::Window)
        .with(SubmenuTiming {
            open: std::time::Duration::from_millis(150),
            close: std::time::Duration::from_millis(200),
        })
        .with(ButtonBusyMotion {
            spin_ms: tokens.motion.slow_min,
            reduced_spin_ms: 3000,
            reveal_delay_ms: tokens.motion.medium_min,
            reveal_fade_ms: tokens.motion.fast,
        })
        .with(ToastRowMotion {
            transition: Some(Transition::tween(
                tokens.motion.fast as f64,
                tokens.motion.ease_standard,
            )),
            gap: tokens.spacing.s2,
        })
        .with(ProgressMotion {
            animate_initial: false,
            determinate_ms: tokens.motion.medium,
            determinate_easing: tokens.motion.ease_standard,
            period_ms: 1500,
            reduced_period_ms: Some(4000),
        })
        .with(ToggleLabelReservation::Reserve {
            weight: tokens.typography.weight_semibold,
            size: tokens.typography.label.size_px,
            extra_small_size: tokens.typography.supporting.size_px,
        })
        .with_icons(crate::icons::set())
        .with(tokens)
        .with(crate::recipes::recipes())
}
