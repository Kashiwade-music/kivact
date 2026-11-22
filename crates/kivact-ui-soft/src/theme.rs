//! Role: The Astryx Neutral theme: the kit's variables on the core theme and on the
//! standard components, for each appearance.
use crate::color as c;
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, FAST, MEDIUM, MEDIUM_MIN, SLOW_MIN};
use crate::radius as r;
use crate::space as s;
use crate::type_scale as t;
use kivact::motion::{Easing, Spring, StepPosition, Transition};
use kivact::style::{
    Color, FontFamily, LineHeight, Theme, ThemeExtension, ThemeMode, color, control, font, radius,
    rgb, space,
};
use kivact::ui::theme::{
    AvatarTheme, DataTableTheme, DialogCenter, DialogTheme, DisclosureTheme, ItemListTheme,
    ProgressTheme, SkeletonTheme, SpinnerTheme, ToastTheme, ToggleButtonTheme, avatar_group,
    button, button_group, calendar, carousel, combobox, data_table, date_input, dropzone,
    hover_card, sidebar, skeleton, spinner, stepper, syntax, time_input, toast, toggle_group,
    toolbar,
};
use kivact::ui::{DisclosureMotion, ProgressMotion, ToggleLabelReservation};

thread_local! {
    /// The kit's light theme, built once per UI thread on first use.
    static LIGHT: Theme = build(ThemeMode::Light);
    /// The kit's dark theme, built once per UI thread on first use.
    static DARK: Theme = build(ThemeMode::Dark);
}

/// Astryx Neutral in the requested appearance. The same appearance returns the
/// same shared theme, so installing it again never restyles.
///
/// Register Figtree with the application to use the theme's body family.
pub fn theme(mode: ThemeMode) -> Theme {
    if mode == ThemeMode::Dark {
        DARK.with(Theme::clone)
    } else {
        LIGHT.with(Theme::clone)
    }
}

fn build(mode: ThemeMode) -> Theme {
    let theme = crate::color::set_colors(Theme::light().with_mode(mode), mode);
    let theme = s::set_space(theme);
    let theme = r::set_radius(theme);
    let theme = set_core(theme);
    let theme = set_sidebar(theme);
    let theme = set_syntax(theme, mode);
    let theme = set_ui_layout(theme);
    let theme = set_ui_table(theme, mode);
    let theme = crate::recipes_controls::install(theme);
    let theme = crate::recipes_overlays::install(theme);
    let theme = crate::recipes_collections::install(theme);
    let theme = crate::recipes_navigation::install(theme);
    // Before `settings`: `settings` re-installs the tables it tunes.
    let theme = crate::recipes_feedback::install(theme, self::theme);
    let theme = crate::recipes_metrics::install(theme);
    let theme = crate::recipes_chat::install(theme);
    let theme = crate::recipes_input::install(theme);
    let theme = crate::recipes_calendar::install(theme);
    let theme = crate::recipes_code::install(theme);
    let theme = settings(theme);
    theme
        .with_focus_ring(m::FOCUS_RING_WIDTH)
        .with_focus_ring_offset(m::FOCUS_RING_OFFSET)
        .with_icons(crate::icons::set())
        .with(crate::theme_policy::policy())
}

/// The sidebar's variables: the kit's surface, text, accent and border colors, its row radius
/// and the width of an open sidebar (the width of the kit's side navigation).
fn set_sidebar(theme: Theme) -> Theme {
    theme
        .set(&sidebar::BACKGROUND_VAR, &c::BG_SURFACE_VAR)
        .set(&sidebar::FOREGROUND_VAR, &c::TEXT_PRIMARY_VAR)
        .set(&sidebar::ACCENT_VAR, &c::OVERLAY_HOVER_VAR)
        .set(&sidebar::ACCENT_FOREGROUND_VAR, &c::TEXT_PRIMARY_VAR)
        .set(&sidebar::PRIMARY_VAR, &c::ACCENT_VAR)
        .set(&sidebar::PRIMARY_FOREGROUND_VAR, &c::ON_ACCENT_VAR)
        .set(&sidebar::BORDER_VAR, &c::BORDER_VAR)
        .set(&sidebar::RING_VAR, &c::FOCUS_RING_VAR)
        .set(&sidebar::PAGE_VAR, &c::BG_BODY_VAR)
        .set(&sidebar::ITEM_RADIUS_VAR, &r::RADIUS_ELEMENT_VAR)
        .set(&sidebar::WIDTH_VAR, 260.0)
}

/// The core variables of the kit: the kit's colors, spacing, radii and typography.
fn set_core(theme: Theme) -> Theme {
    theme
        .set(&color::SURFACE_VAR, &c::BG_SURFACE_VAR)
        .set(&color::SURFACE_ALT_VAR, &c::BG_MUTED_VAR)
        .set(&color::TEXT_VAR, &c::TEXT_PRIMARY_VAR)
        .set(&color::TEXT_MUTED_VAR, &c::TEXT_SECONDARY_VAR)
        .set(&color::ACCENT_VAR, &c::ACCENT_VAR)
        .set(&color::ACCENT_TEXT_VAR, &c::ON_ACCENT_VAR)
        .set(&color::BORDER_VAR, &c::BORDER_VAR)
        .set(&color::FOCUS_VAR, &c::FOCUS_RING_VAR)
        .set(&color::DANGER_VAR, &c::ERROR_VAR)
        .set(&color::SUCCESS_VAR, &c::SUCCESS_VAR)
        .set(&color::WARNING_VAR, &c::WARNING_VAR)
        .set(&color::OVERLAY_VAR, &c::OVERLAY_SCRIM_VAR)
        .set(&color::SHADOW_VAR, &c::SHADOW_VAR)
        .set(&color::SELECTION_VAR, &c::INFO_MUTED_VAR)
        .set(&color::HOVER_VAR, &c::OVERLAY_HOVER_VAR)
        .set(&color::PRESSED_VAR, &c::OVERLAY_PRESSED_VAR)
        .set(&color::RAISED_VAR, &c::BG_POPOVER_VAR)
        .set(&space::XS_VAR, &s::SPACE_S1_VAR)
        .set(&space::SM_VAR, &s::SPACE_S2_VAR)
        .set(&space::MD_VAR, &s::SPACE_S3_VAR)
        .set(&space::LG_VAR, &s::SPACE_S4_VAR)
        .set(&space::XL_VAR, &s::SPACE_S6_VAR)
        .set(&radius::SM_VAR, &r::RADIUS_INNER_VAR)
        .set(&radius::MD_VAR, &r::RADIUS_ELEMENT_VAR)
        .set(&radius::LG_VAR, &r::RADIUS_CONTAINER_VAR)
        .set(&font::FAMILY_VAR, FontFamily::from(t::FAMILY_BODY))
        .set(&font::SIZE_VAR, t::BODY.size_px)
        .set(&font::SIZE_SMALL_VAR, t::SUPPORTING.size_px)
        .set(&font::SIZE_LARGE_VAR, t::LARGE.size_px)
        .set(&font::WEIGHT_VAR, t::BODY.weight)
        .set(&font::WEIGHT_STRONG_VAR, t::WEIGHT_SEMIBOLD)
        .set(
            &font::LINE_HEIGHT_VAR,
            LineHeight::Scale(t::BODY.line_height_px / t::BODY.size_px),
        )
        .set(&control::HEIGHT_VAR, m::CONTROL_MD)
        .set(&control::DISABLED_OPACITY_VAR, m::DISABLED_OPACITY)
        .set(
            &control::OVERLAY_MARGIN_VAR,
            &s::SPACE_LAYER_VIEWPORT_GUTTER_VAR,
        )
}

/// The token colors of source code, one variable per syntax kind of the core theme.
fn set_syntax(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    theme
        .set(&syntax::KEYWORD_VAR, pick(0x6b187c, 0xefa8ff))
        .set(&syntax::TYPE_VAR, pick(0x6b187c, 0xefa8ff))
        .set(&syntax::STRING_VAR, pick(0x0b5615, 0xa4d6a3))
        .set(&syntax::COMMENT_VAR, pick(0x6a6a6a, 0x9e9e9e))
        .set(&syntax::OPERATOR_VAR, pick(0x6a6a6a, 0x9e9e9e))
        .set(&syntax::PUNCTUATION_VAR, pick(0x6a6a6a, 0x9e9e9e))
        .set(&syntax::NUMBER_VAR, pick(0x733100, 0xffb37e))
        .set(&syntax::CONSTANT_VAR, pick(0x733100, 0xffb37e))
        .set(&syntax::FUNCTION_VAR, pick(0x00458c, 0xa1caff))
        .set(&syntax::VARIABLE_VAR, pick(0x111111, 0xe2e2e2))
        .set(&syntax::TAG_VAR, pick(0x8a0011, 0xffaea7))
        .set(&syntax::ATTRIBUTE_VAR, pick(0x584400, 0xeec448))
        .set(&syntax::PROPERTY_VAR, pick(0x005348, 0x90d7c8))
        .set(&syntax::MACRO_VAR, pick(0x00458c, 0xa1caff))
        .set(&syntax::LIFETIME_VAR, pick(0x733100, 0xffb37e))
}

/// The variables of the layout components and the feedback components.
fn set_ui_layout(theme: Theme) -> Theme {
    theme
        .set(&button::BUSY_SPIN_MS_VAR, SLOW_MIN as f64)
        .set(&button::BUSY_REDUCED_SPIN_MS_VAR, 3000.0)
        .set(&button::BUSY_REVEAL_DELAY_MS_VAR, MEDIUM_MIN as f64)
        .set(&button::BUSY_REVEAL_FADE_MS_VAR, FAST as f64)
        .set(&avatar_group::OVERLAP_VAR, 9.0)
        .set(&calendar::CELL_SIZE_VAR, 32.0)
        .set(&calendar::MONTH_GAP_VAR, &s::SPACE_S4_VAR)
        .set(&calendar::WEEK_GAP_VAR, &s::SPACE_S2_VAR)
        .set(&carousel::GAP_VAR, &s::SPACE_S1_VAR)
        .set(&carousel::PADDING_VAR, 0.0)
        .set(&carousel::EDGE_FADE_WIDTH_VAR, &s::SPACE_S1_VAR)
        .set(&dropzone::PADDING_VAR, &s::SPACE_S3_VAR)
        .set(&dropzone::MIN_HEIGHT_VAR, 96.0)
        .set(&toolbar::BACKGROUND_VAR, Color::TRANSPARENT)
        .set(&toolbar::BORDER_VAR, &c::BORDER_VAR)
        .set(&toolbar::PADDING_VAR, &s::SPACE_S3_VAR)
        .set(&toolbar::GAP_VAR, &s::SPACE_S1_VAR)
        .set(&toolbar::MIN_HEIGHT_VAR, m::CONTROL_MD + 2.0 * 12.0)
        .set(&button_group::RADIUS_VAR, &r::RADIUS_ELEMENT_VAR)
        .set(&button_group::DIVIDER_VAR, &c::BORDER_VAR)
        .set(&button_group::SOLID_DIVIDER_VAR, &c::ON_ACCENT_VAR)
        .set(&button_group::TEXT_BACKGROUND_VAR, &c::BG_MUTED_VAR)
        .set(&button_group::TEXT_COLOR_VAR, &c::TEXT_PRIMARY_VAR)
        .set(&toggle_group::GAP_VAR, &s::SPACE_S1_VAR)
        .set(&hover_card::OFFSET_VAR, &s::SPACE_S1_VAR)
        .set(
            &hover_card::VIEWPORT_MARGIN_VAR,
            &s::SPACE_LAYER_VIEWPORT_GUTTER_VAR,
        )
        .set(&date_input::WHEEL_ROW_HEIGHT_VAR, 28.0)
        .set(&date_input::WHEEL_WIDTH_VAR, 80.0)
        .set(&time_input::WHEEL_ROW_HEIGHT_VAR, 28.0)
        .set(&time_input::WHEEL_WIDTH_VAR, 80.0)
        .set(&stepper::GAP_VAR, 4.0)
        .set(&stepper::PADDING_VAR, 0.0)
        .set(&stepper::MIN_STEP_WIDTH_VAR, 112.0)
        .set(&stepper::CONNECTOR_GAP_VAR, 0.0)
        .set(&spinner::PERIOD_MS_VAR, SLOW_MIN as f64)
        .set(&skeleton::PERIOD_MS_VAR, SLOW_MIN as f64)
        .set(&skeleton::DELAY_MS_VAR, 1000.0)
        .set(&skeleton::STAGGER_MS_VAR, 100.0)
        .set(&skeleton::GAP_VAR, &s::SPACE_S2_VAR)
        .set(&toast::ROW_GAP_VAR, &s::SPACE_S2_VAR)
}

/// The variables of the table-like components: the combobox and the data table.
fn set_ui_table(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    theme
        .set(&combobox::FOREGROUND_VAR, &c::TEXT_PRIMARY_VAR)
        .set(&combobox::MUTED_VAR, &c::TEXT_SECONDARY_VAR)
        .set(&combobox::BACKGROUND_VAR, &c::BG_POPOVER_VAR)
        .set(&combobox::BORDER_VAR, &c::BORDER_VAR)
        .set(&combobox::ACTIVE_BACKGROUND_VAR, &c::OVERLAY_HOVER_VAR)
        .set(&combobox::CHIP_BACKGROUND_VAR, &c::ACCENT_MUTED_VAR)
        .set(&combobox::FOCUS_VAR, &c::ACCENT_VAR)
        .set(&combobox::FIELD_RADIUS_VAR, &r::RADIUS_ELEMENT_VAR)
        .set(&combobox::CHIP_RADIUS_VAR, &r::RADIUS_ELEMENT_VAR)
        .set(&combobox::OPTION_RADIUS_VAR, &r::RADIUS_ELEMENT_VAR)
        .set(&combobox::FOCUS_RING_VAR, m::FOCUS_RING_WIDTH)
        .set(&combobox::DISABLED_OPACITY_VAR, m::DISABLED_OPACITY)
        .set(&combobox::EMPTY_FONT_SIZE_VAR, t::BODY.size_px)
        .set(&data_table::STICKY_SURFACE_VAR, &c::BG_CARD_VAR)
        .set(&data_table::STICKY_HOVER_VAR, &c::OVERLAY_HOVER_VAR)
        .set(&data_table::STICKY_SELECTED_VAR, &c::ACCENT_MUTED_VAR)
        .set(&data_table::STICKY_SHADOW_VAR, &c::SHADOW_VAR)
        .set(&data_table::STICKY_SHADOW_WIDTH_VAR, 6.0)
        .set(&data_table::ROW_STATUS_NEUTRAL_VAR, &c::ICON_SECONDARY_VAR)
        .set(
            &data_table::ROW_STATUS_INFO_VAR,
            rgb(if dark { 0x88bcff } else { 0x003978 }),
        )
        .set(
            &data_table::ROW_STATUS_SUCCESS_VAR,
            rgb(if dark { 0x90ca90 } else { 0x00490b }),
        )
        .set(
            &data_table::ROW_STATUS_WARNING_VAR,
            rgb(if dark { 0xff9e55 } else { 0x622700 }),
        )
        .set(
            &data_table::ROW_STATUS_DANGER_VAR,
            rgb(if dark { 0xff9890 } else { 0x76000c }),
        )
        .set(&data_table::ROW_STATUS_ACCENT_VAR, &c::ICON_ACCENT_VAR)
}

/// Change a component table the theme has: clone it, apply `f` and install it again.
fn tune<T>(theme: Theme, f: impl FnOnce(T) -> T) -> Theme
where
    T: ThemeExtension + Clone,
{
    let table = f(theme.ext::<T>().to_owned());
    theme.with(table)
}

/// The component settings: each table's values that are not theme variables.
fn settings(theme: Theme) -> Theme {
    let theme = tune(theme, |t: ItemListTheme| {
        t.title_lines(Some(1)).description_lines(Some(1))
    });
    let theme = tune(theme, |t: AvatarTheme| t.name_tooltip(true));
    let theme = tune(theme, |t: DialogTheme| t.center(DialogCenter::Window));
    let theme = tune(theme, |t: DisclosureTheme| {
        t.motion(DisclosureMotion {
            transition: Some(Transition::spring(Spring {
                stiffness: 157.91,
                damping: 25.13,
                mass: 1.0,
                rest: 1e-3,
            })),
            ..Default::default()
        })
    });
    let theme = tune(theme, |t: SpinnerTheme| t.reduced_period(Some(3000.0)));
    let theme = tune(theme, |t: ProgressTheme| {
        t.motion(ProgressMotion {
            animate_initial: false,
            determinate_ms: MEDIUM,
            determinate_easing: EASE_STANDARD,
            period_ms: 1500,
            reduced_period_ms: Some(4000),
        })
    });
    let theme = tune(theme, |t: SkeletonTheme| {
        t.transition(Some(Transition::tween(
            400,
            Easing::steps(10, StepPosition::JumpEnd),
        )))
    });
    let theme = tune(theme, |t: ToastTheme| {
        t.row_transition(Some(Transition::tween(FAST as f64, EASE_STANDARD)))
    });
    let theme = tune(theme, |t: ToggleButtonTheme| {
        t.label_reservation(ToggleLabelReservation::Reserve {
            weight: t::WEIGHT_SEMIBOLD,
            size: t::LABEL.size_px,
            extra_small_size: t::SUPPORTING.size_px,
        })
    });
    tune(theme, |t: DataTableTheme| {
        t.sticky_shadow_transition(Some(Transition::tween(150, EASE_STANDARD)))
            .sticky_overlay_transition(Some(Transition::tween(125, EASE_STANDARD)))
            .row_status_header_key("soft.table.row_status.column_header")
    })
}
