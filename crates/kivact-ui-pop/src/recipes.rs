//! Role: Pop styling for standard UI components.
//!
//! The Pop look of the standard components of `kivact::ui`: what
//! [`theme`](crate::theme) installs so that `kivact::ui::button`,
//! `toggle`, `checkbox`, `switch`, `text_input`, `select`, `menu`, `dialog`,
//! `tooltip`, `tabs`, `radio_group`, `slider`, `toast_region`, `progress` and
//! `disclosure` are drawn
//! like this kit's own.

use crate::Tokens;
use crate::button::{ButtonTone, Slant, button_style};
use crate::choice::check_style;
use crate::choice::{draw_check_indicator, draw_switch_indicator};
use crate::form::input_style;
use kivact::draw::Draw;
use kivact::draw::{Layer, Transform};
use kivact::geometry::Size;
use kivact::style::{Style, Theme, style};
use kivact::ui::Look;
use kivact::ui::{
    ButtonState, ButtonVariant, CheckableState, ControlSize, TextInputState, UiRecipes,
};
use kivact::view::CheckedState;

/// The size of a check box's box and the room a switch's track takes.
const CHECK: (f64, f64) = (16.0, 16.0);
const SWITCH: (f64, f64) = (32.0, 18.0);
/// The gap between the drawn part and the label, as in `ui::checkbox`.
const GAP: f64 = 12.0;

/// The recipes of the kit's look.
///
/// The table is built once and shared, so two themes of the kit compare
/// their recipes by identity and installing the kit's theme again never
/// restyles.
pub(crate) fn recipes() -> UiRecipes {
    static RECIPES: std::sync::OnceLock<UiRecipes> = std::sync::OnceLock::new();
    RECIPES.get_or_init(build).clone()
}

fn build() -> UiRecipes {
    UiRecipes::default()
        .breadcrumb(crate::recipes_navigation::breadcrumb)
        .pagination(crate::recipes_navigation::pagination)
        .stepper(crate::recipes_navigation::stepper)
        .navigation_menu(crate::recipes_navigation::navigation_menu)
        .item_list(crate::recipes_items::item_list)
        .table(crate::recipes_items::table)
        .tree(crate::recipes_items::tree)
        .calendar(crate::recipes_calendar::calendar)
        .date_input(crate::recipes_calendar::date_input)
        .date_range_input(crate::recipes_calendar::date_range_input)
        .kbd(|_| crate::recipes_indicators::kbd)
        .empty_state(crate::recipes_feedback::empty_state)
        .alert(crate::recipes_feedback::alert)
        .card(crate::recipes_feedback::card)
        .input_group(crate::recipes_feedback::input_group)
        .avatar(|_| crate::recipes_avatar::avatar)
        .badge(|_| crate::recipes_badge::badge)
        .badge_label(|_| crate::recipes_badge::label)
        .skeleton(|_| crate::recipes_indicators::skeleton)
        .spinner(|_| crate::recipes_indicators::spinner)
        .toolbar(crate::recipes_groups::toolbar)
        .toggle_group(crate::recipes_groups::toggle_group)
        .button_group(crate::recipes_groups::button_group)
        .dropzone(crate::recipes_files::dropzone)
        .field(crate::recipes_field::field)
        .button(|_| button)
        .button_content(|_| crate::recipes_extra::button_content)
        .button_busy(|_| crate::recipes_extra::button_busy)
        .toggle(|_| crate::recipes_small::toggle)
        .checkbox(|_| checkbox)
        .switch(|_| switch)
        .text_input(|_| text_input)
        .text_area(|_| text_area)
        .select(crate::recipes_extra::select)
        .menu(crate::recipes_extra::menu)
        .dialog(crate::recipes_extra::dialog)
        .tooltip(|tooltip| tooltip.surface(|_| crate::recipes_extra::tooltip))
        .tabs(crate::recipes_extra::tabs)
        .radio_group(crate::recipes_extra::radio_group)
        .slider(crate::recipes_extra::slider)
        .toast(crate::recipes_extra::toast)
        .list(crate::recipes_collections::list)
        .data_table(crate::recipes_collections::data_table)
        .progress(crate::recipes_small::progress)
        .disclosure(crate::recipes_small::disclosure)
        .command_palette(crate::recipes_palette::command_palette)
        .tag_input(crate::recipes_input::tag_input)
        .number_input(crate::recipes_input::number_input)
}

fn button(theme: &Theme, state: &Look<ButtonState>) -> Style {
    let variant = state
        .variant::<ButtonVariant>()
        .copied()
        .unwrap_or_default();
    let hue = state.variant::<ButtonTone>().copied();
    let s = button_style(
        theme,
        variant,
        hue,
        state.size,
        state.variant::<Slant>().is_some(),
    );
    if !state.icon_only {
        return s;
    }
    // An icon button is a square of the button's height with a quiet icon.
    let p = theme.ext::<Tokens>();
    let side = match state.size {
        ControlSize::ExtraSmall => 26,
        ControlSize::Small => 32,
        ControlSize::Medium => 36,
        ControlSize::Large => 40,
    };
    let s = s.width(side).padding_x(0).radius(8);
    if hue.is_none() && matches!(variant, ButtonVariant::Ghost | ButtonVariant::Link) {
        s.color(p.text_muted)
            .hover(style().background(p.surface_active).color(p.text))
            .pressed(style().opacity(0.7))
    } else {
        s
    }
}

/// A kit check row with the room its drawn part needs on the left.
fn row(theme: &Theme, drawn: (f64, f64)) -> Style {
    check_style(theme).weight(500).padding_left(drawn.0 + GAP)
}

fn checkbox(theme: &Theme, state: &Look<CheckableState>) -> Style {
    let (theme, state) = (theme.clone(), **state);
    row(&theme, CHECK).paint(move |d, size| draw_checkbox(&theme, &state, d, size))
}

fn switch(theme: &Theme, state: &Look<CheckableState>) -> Style {
    let (theme, state) = (theme.clone(), **state);
    row(&theme, SWITCH).paint(move |d, size| draw_switch(&theme, &state, d, size))
}

/// Draw `part` centered vertically at the left edge.
fn drawn(d: &mut Draw, size: Size, part: (f64, f64), paint: impl FnOnce(&mut Draw, Size)) {
    d.layer(
        Layer::new()
            .transform(Transform::new().translate(0, ((size.height - part.1) / 2.0).round())),
        |d| {
            paint(d, Size::new(part.0, part.1));
        },
    );
}

fn draw_checkbox(theme: &Theme, look: &CheckableState, d: &mut Draw, size: Size) {
    let p = theme.ext::<Tokens>();
    drawn(d, size, CHECK, |d, s| {
        draw_check_indicator(
            d,
            s,
            look.checked.is_checked(),
            look.checked == CheckedState::Indeterminate,
            p.blue,
            p.surface,
            p.line_strong,
        );
    });
}

fn draw_switch(theme: &Theme, look: &CheckableState, d: &mut Draw, size: Size) {
    let p = theme.ext::<Tokens>();
    drawn(d, size, SWITCH, |d, s| {
        draw_switch_indicator(d, s, look.checked.is_checked(), p.blue, p.line_strong);
    });
}

fn text_input(theme: &Theme, state: &Look<TextInputState>) -> Style {
    let size = state.size;
    let s = input_style(theme);
    match size {
        ControlSize::ExtraSmall => s.height(26),
        ControlSize::Small => s.height(32),
        ControlSize::Medium | ControlSize::Large => s,
    }
}

fn text_area(theme: &Theme, state: &Look<TextInputState>) -> Style {
    text_input(theme, state)
        .height(80)
        .padding_y(8)
        .line_height(1.5)
}
