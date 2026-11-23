//! Role: Style Astryx Neutral's select and combobox: the trigger, the popup, the option
//! rows and the keyed suggestion list.
use crate::color as c;
use crate::metrics as m;
use crate::motion::{EASE_ENTRANCE, EASE_STANDARD, FAST, FAST_MAX};
use crate::radius as r;
use crate::recipe_parts::{frame, shadowed, text};
use crate::recipes_field::height;
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, PopupState};
use kivact::motion::Transition;
use kivact::style::{
    Align, Color, FlexDirection, Justify, Position, Style, Theme, ThemeMode, TransitionProp, pct,
    style,
};
use kivact::ui::theme::{ComboboxTheme, Look, SelectTheme, SelectTriggerState};
use kivact::ui::{ControlSize, SelectSheetOption};

/// The select's trigger: a framed field, its text muted while it shows the placeholder.
fn trigger(theme: &Theme, look: &Look<SelectTriggerState>, _base: Style) -> Style {
    let h = height(look.size);
    let block = (h - t::BODY.line_height_px) / 2.0 - m::BORDER_WIDTH;
    let color = if look.placeholder {
        c::TEXT_SECONDARY
    } else {
        c::TEXT_PRIMARY
    };
    text(frame(theme, &look.input, false), t::BODY, color)
        .width(pct(100))
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .min_height(h)
        .padding_x(s::SPACE_S3)
        .padding_y(block)
        .gap(s::SPACE_S2)
}

/// The floating list: the popup surface with its low shadow, entering from above.
fn surface(theme: &Theme, _look: &Look<PopupState>, _base: Style) -> Style {
    const MAX_LIST_HEIGHT: f64 = 300.0;
    const ENTER_SCALE: f64 = 0.95;
    let dark = theme.mode() == ThemeMode::Dark;
    let base = style()
        .max_height(MAX_LIST_HEIGHT)
        .padding_x(theme.get(&s::SPACE_S1_VAR) + m::BORDER_WIDTH)
        .padding_y(s::SPACE_S1)
        .background(c::BG_POPOVER)
        .border_radius(r::RADIUS_CONTAINER)
        .enter(
            style()
                .opacity(0.0)
                .translate_y(-theme.get(&s::SPACE_S2_VAR))
                .scale(ENTER_SCALE),
        )
        .transition(
            TransitionProp::All,
            Transition::tween(FAST_MAX as f64, EASE_ENTRANCE),
        );
    // The popup keeps the standard below-anchor placement and key behavior.
    shadowed(base, shadows::low(dark))
}

/// An option row: its label in the body text, highlighted while it is active.
fn option(theme: &Theme, look: &Look<ListItemState>, _base: Style) -> Style {
    let mut role = t::BODY;
    if look.selected {
        role.weight = t::WEIGHT_MEDIUM;
    }
    let mut base = style();
    let padding_y = if let Some(SelectSheetOption(size)) = look.variant::<SelectSheetOption>() {
        role.size_px = 16.0;
        role.line_height_px = 24.0;
        let row_height = match size {
            ControlSize::ExtraSmall | ControlSize::Small => 32.0,
            ControlSize::Medium => 36.0,
            ControlSize::Large => 40.0,
        };
        base = base.min_height(row_height);
        (row_height - role.line_height_px) / 2.0
    } else {
        theme.get(&s::SPACE_S1_5_VAR)
    };
    let base = text(base, role, c::TEXT_PRIMARY)
        .width(pct(100))
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .padding_x(s::SPACE_S2)
        .padding_y(padding_y)
        .gap(s::SPACE_S2)
        .border_radius(r::RADIUS_ELEMENT);
    if look.active && !look.input.is_disabled() {
        base.background(c::OVERLAY_HOVER)
    } else {
        base
    }
}

/// The box of an option's check: shown at the row's end while the option is selected.
fn check(_theme: &Theme, look: &Look<ListItemState>, _base: Style) -> Style {
    let size = if look.selected { m::ICON_SM } else { 0.0 };
    let color = if look.input.is_disabled() {
        c::ICON_DISABLED
    } else {
        c::ACCENT
    };
    let base = style().size(size, size).flex_shrink(0).color(color);
    if look.selected {
        base
    } else {
        base.position(Position::Absolute)
    }
}

/// The chevron of the trigger, turned while the list is open.
fn indicator(_theme: &Theme, look: &Look<PopupState>, _base: Style) -> Style {
    style()
        .size(m::ICON_SM, m::ICON_SM)
        .flex_shrink(0)
        .color(c::ICON_SECONDARY)
        .rotate(if look.open { 180 } else { 0 })
        .transition(
            TransitionProp::All,
            Transition::tween(FAST as f64, EASE_STANDARD).essential(),
        )
}

/// The secondary line of an option: the supporting text, smaller in a sheet.
fn description(_theme: &Theme, look: &Look<ListItemState>, _base: Style) -> Style {
    let mut role = t::SUPPORTING;
    if look.variant::<SelectSheetOption>().is_some() {
        role.size_px = 14.0;
    }
    text(style().single_line(), role, c::TEXT_SECONDARY)
}

/// The field of a combobox: a wrapping frame that holds the chips and the input.
fn field(theme: &Theme, look: &Look<PartState>, _base: Style) -> Style {
    let base = frame(theme, look, false)
        .min_height(m::CONTROL_MD)
        .padding_x(s::SPACE_S2)
        .padding_y(theme.get(&s::SPACE_S1_VAR) - m::BORDER_WIDTH)
        .gap(s::SPACE_S1)
        .align_items(Align::Center)
        .flex_wrap(true);
    if look.is_focused() {
        base.border_color(c::ACCENT)
            .shadow(shadows::input_focus_ring(theme.mode() == ThemeMode::Dark))
    } else {
        base
    }
}

/// The text input inside a combobox field: it draws no frame of its own.
fn input(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_grow(1)
        .min_width(64)
        .height(24)
        .padding(0)
        .border(0, Color::TRANSPARENT)
        .outline_none()
        .background(Color::TRANSPARENT)
        .hover(style().outline_none())
        .focus(style().outline_none())
}

/// The combobox popup: the low shadow over the popover fill.
fn combobox_surface(theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    let dark = theme.mode() == ThemeMode::Dark;
    shadowed(
        style()
            .width(pct(100))
            .max_height(300)
            .padding(s::SPACE_S1)
            .border_radius(r::RADIUS_CONTAINER)
            .background(c::BG_POPOVER),
        shadows::low(dark),
    )
}

/// A combobox suggestion row: the body text, highlighted while it is active.
fn combobox_option(_theme: &Theme, look: &Look<ListItemState>, _base: Style) -> Style {
    let base = text(
        style()
            .min_height(32)
            .padding_x(s::SPACE_S2)
            .padding_y(s::SPACE_S1_5)
            .border_radius(r::RADIUS_ELEMENT),
        t::BODY,
        c::TEXT_PRIMARY,
    );
    if look.active {
        base.background(c::OVERLAY_HOVER)
    } else {
        base
    }
}

/// A selected value's chip in the combobox field.
fn chip(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    style()
        .height(24)
        .padding_left(s::SPACE_S2)
        .padding_right(s::SPACE_S1)
        .gap(s::SPACE_S1)
        .align_items(Align::Center)
        .border_radius(r::RADIUS_ELEMENT)
        .background(c::ACCENT_MUTED)
        .color(c::TEXT_PRIMARY)
}

/// The remove button of a chip.
fn remove(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    style().size(16, 16).padding(0)
}

/// The text shown when no suggestion matches.
fn empty(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    style().padding(s::SPACE_S2).color(c::TEXT_SECONDARY)
}

/// Install the kit's recipes of the select and the combobox.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme.with(
        SelectTheme::default()
            .trigger(trigger)
            .surface(surface)
            .option(option)
            .check(check)
            .indicator(indicator)
            .description(description),
    );
    theme.with(
        ComboboxTheme::default()
            .frame(field)
            .input(input)
            .surface(combobox_surface)
            .option(combobox_option)
            .chip(chip)
            .remove(remove)
            .empty(empty),
    )
}
