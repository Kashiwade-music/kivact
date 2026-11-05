//! Role: Install the Astryx Neutral tokens and recipes on the Kivact theme.
use std::time::Duration;

use crate::tokens::Tokens;
use kivact::geometry::Edge;
use kivact::interaction::DisabledButtonFocus;
use kivact::interaction::DisabledMenuFocus;
use kivact::interaction::MenuPress;
use kivact::interaction::SliderTouchMotion;
use kivact::interaction::SubmenuTiming;
use kivact::interaction::ToastSwipe;
use kivact::interaction::TouchSliderGesture;
use kivact::interaction::{
    ButtonPolicy, InteractionPolicy, MenuPolicy, SliderPolicy, SwitchDrag, SwitchPolicy,
    ToastPolicy,
};
use kivact::motion::Spring;
use kivact::motion::Transition;
use kivact::style::LineHeight;
use kivact::style::Theme;
use kivact::style::ThemeMode;
use kivact::style::theme::{Palette, Radii, Spacing, Typography};
use kivact::ui::ButtonBusyMotion;
use kivact::ui::DialogCenter;
use kivact::ui::ProgressMotion;
use kivact::ui::ToastRowMotion;
use kivact::ui::ToggleLabelReservation;
use kivact::ui::{
    ButtonTokens, DataTableTokens, DialogTokens, ProgressTokens, ToastTokens, ToggleButtonTokens,
    UiTokens,
};

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
        .with_icons(crate::icons::set())
        .with_overlay_margin(tokens.spacing.layer_viewport_gutter)
        .with(InteractionPolicy {
            hover_card: kivact::interaction::HoverCardTiming {
                open_delay: Duration::from_millis(300),
                close_delay: Duration::from_millis(200),
            },
            menu: MenuPolicy {
                press: MenuPress(true),
                disabled_focus: DisabledMenuFocus::Retain,
                submenu: SubmenuTiming {
                    open: std::time::Duration::from_millis(150),
                    close: std::time::Duration::from_millis(200),
                },
                ..Default::default()
            },
            button: ButtonPolicy {
                disabled_focus: DisabledButtonFocus::Retain,
            },
            slider: SliderPolicy {
                touch: SliderTouchMotion::Direct {
                    gesture: TouchSliderGesture {
                        slop_px: 10.0,
                        projection: Duration::from_micros(99_600),
                        response: Duration::from_millis(400),
                    },
                    lifted_scale: 1.55,
                    lift: Spring {
                        stiffness: 854.0,
                        damping: 35.07,
                        mass: 1.0,
                    },
                    drop: Spring {
                        stiffness: 157.9,
                        damping: 17.59,
                        mass: 1.0,
                    },
                },
            },
            switch: SwitchPolicy {
                drag: SwitchDrag(true),
            },
            toast: ToastPolicy {
                swipe: ToastSwipe::Enabled {
                    edge: Edge::Bottom,
                    transition: Transition::tween(
                        tokens.motion.fast as f64,
                        tokens.motion.ease_standard,
                    ),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(UiTokens {
            indicators: kivact::ui::IndicatorTokens {
                period_ms: tokens.motion.slow_min,
                reduced_period_ms: Some(3000),
                skeleton_transition: Some(Transition::tween(
                    400,
                    kivact::motion::Easing::steps_end(10),
                )),
                delay_ms: 1000.0,
                stagger_ms: 100.0,
                gap: tokens.spacing.s2,
                ..Default::default()
            },
            toolbar: kivact::ui::ToolbarTokens {
                background: kivact::style::Color::TRANSPARENT,
                border: colors.border,
                padding: tokens.spacing.s3,
                gap: tokens.spacing.s1,
                min_height: tokens.sizes.control_md + tokens.spacing.s3 * 2.0,
            },
            button_group: kivact::ui::ButtonGroupTokens {
                radius: tokens.radius.element,
                divider: colors.border,
                solid_divider: colors.on_accent,
                text_background: colors.bg_muted,
                text_color: colors.text_primary,
                ..Default::default()
            },
            toggle_group: kivact::ui::ToggleGroupTokens {
                gap: tokens.spacing.s1,
            },
            disclosure: kivact::ui::DisclosureTokens {
                motion: kivact::ui::DisclosureMotion {
                    transition: Some(Transition::spring(Spring {
                        stiffness: 157.91,
                        damping: 25.13,
                        mass: 1.0,
                    })),
                    ..Default::default()
                },
            },
            combobox: kivact::ui::ComboboxTokens {
                foreground: colors.text_primary,
                muted: colors.text_secondary,
                background: colors.bg_popover,
                border: colors.border,
                active_background: colors.overlay_hover,
                chip_background: colors.accent_muted,
                focus: colors.accent,
                field_radius: Some(tokens.radius.element),
                option_radius: tokens.radius.element,
                chip_radius: tokens.radius.element,
                focus_ring: Some(tokens.sizes.focus_ring_width),
                disabled_opacity: Some(tokens.sizes.disabled_opacity),
                empty_font_size: Some(tokens.typography.body.size_px),
                ..Default::default()
            },
            hover_card: kivact::ui::HoverCardTokens {
                offset: tokens.spacing.s1,
                viewport_margin: tokens.spacing.layer_viewport_gutter,
            },
            button: ButtonTokens {
                busy_motion: ButtonBusyMotion {
                    spin_ms: tokens.motion.slow_min,
                    reduced_spin_ms: 3000,
                    reveal_delay_ms: tokens.motion.medium_min,
                    reveal_fade_ms: tokens.motion.fast,
                },
            },
            dialog: DialogTokens {
                center: DialogCenter::Window,
            },
            progress: ProgressTokens {
                motion: ProgressMotion {
                    animate_initial: false,
                    determinate_ms: tokens.motion.medium,
                    determinate_easing: tokens.motion.ease_standard,
                    period_ms: 1500,
                    reduced_period_ms: Some(4000),
                },
            },
            toast: ToastTokens {
                row_motion: ToastRowMotion {
                    transition: Some(Transition::tween(
                        tokens.motion.fast as f64,
                        tokens.motion.ease_standard,
                    )),
                    gap: tokens.spacing.s2,
                },
            },
            toggle_button: ToggleButtonTokens {
                label_reservation: ToggleLabelReservation::Reserve {
                    weight: tokens.typography.weight_semibold,
                    size: tokens.typography.label.size_px,
                    extra_small_size: tokens.typography.supporting.size_px,
                },
            },
            data_table: DataTableTokens {
                sticky: kivact::ui::DataTableStickyPaint {
                    surface: colors.bg_card,
                    hover: colors.overlay_hover,
                    selected: colors.accent_muted,
                    shadow: colors.shadow,
                    shadow_width: 6.0,
                    shadow_transition: Some(kivact::motion::Transition::tween(
                        150,
                        tokens.motion.ease_standard,
                    )),
                    overlay_transition: Some(kivact::motion::Transition::tween(
                        125,
                        tokens.motion.ease_standard,
                    )),
                },
                row_status: kivact::ui::DataTableRowStatusPaint {
                    neutral: colors.icon_secondary,
                    info: kivact::style::rgb(if mode == ThemeMode::Dark {
                        0x88bcff
                    } else {
                        0x003978
                    }),
                    success: kivact::style::rgb(if mode == ThemeMode::Dark {
                        0x90ca90
                    } else {
                        0x00490b
                    }),
                    warning: kivact::style::rgb(if mode == ThemeMode::Dark {
                        0xff9e55
                    } else {
                        0x622700
                    }),
                    danger: kivact::style::rgb(if mode == ThemeMode::Dark {
                        0xff9890
                    } else {
                        0x76000c
                    }),
                    accent: colors.icon_accent,
                    header_key: "soft.table.row_status.column_header",
                },
            },
            ..Default::default()
        })
        .with(tokens)
        .with(crate::recipes::recipes())
}
