//! Role: Lyra recipes of the standard button: root, content row, icon box, busy ring.

use crate::recipe_parts::{ButtonLook, metrics, stroke_loader, trigger_style};
use kivact::geometry::Point;
use kivact::style::{Align, FlexDirection, Justify, Position, Style, Theme, pct, style};
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::theme::{ButtonState, ButtonTheme, Look};

/// The button recipe: the trigger look of its variant, with room for its icons.
fn button(t: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    let look = ButtonLook::of(state.variant::<ButtonVariant>().unwrap_or_default());
    let s = trigger_style(t, look, state.size, &state.input, state.expanded);

    // `has-data-[icon=inline-start]:pl-2` (`pl-1.5` in `sm` and `xs`), and
    // `pr` likewise beside a trailing icon.
    let edge = match state.size {
        ControlSize::ExtraSmall | ControlSize::Small => 6.0,
        ControlSize::Medium | ControlSize::Large => 8.0,
    };
    let s = if state.leading_icon {
        s.padding_left(edge)
    } else {
        s
    };
    let s = if state.trailing_icon {
        s.padding_right(edge)
    } else {
        s
    };
    let s = if state.busy {
        // Room for the leading spinner that `button_busy` places absolutely.
        let (_, pad, gap, icon) = metrics(state.size);
        s.padding_left(pad + icon + gap)
    } else {
        s
    };
    // button.tsx `size="icon"`: a square, as wide as the button is high.
    let s = if state.is_icon_only() {
        s.aspect_ratio(1.0).padding_x(0.0)
    } else {
        s
    };
    // A button inside an input group takes that group's size.
    match crate::recipes_input::button(state) {
        Some(look) => s.merge(&look),
        None => s,
    }
}

/// The row of the button's icon and label.
fn button_content(_: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    let (_, _, gap, _) = metrics(state.size);
    let gap = if crate::recipes_input::button_icon(state).is_some() {
        4.0
    } else {
        gap
    };
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(gap)
}

/// The box of a button's icons: `size-4`, `size-3.5` in a small button.
fn button_icon(_: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    let (_, _, _, side) = metrics(state.size);
    let side = crate::recipes_input::button_icon(state).unwrap_or(side);
    style()
        .size(side, side)
        .flex_shrink(0.0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// A three-quarter ring the button turns while busy.
fn button_busy(_: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    let (_, pad, _, side) = metrics(state.size);
    let color = ButtonLook::of(state.variant::<ButtonVariant>().unwrap_or_default()).foreground();
    // The button turns the indicator while busy.
    style()
        .position(Position::Absolute)
        .left(pad)
        .top(pct(50.0) - side / 2.0)
        .size(side, side)
        .paint(move |d, size| {
            stroke_loader(d, Point::new(0.0, 0.0), size.width, 0.0, color);
        })
}

/// Installs the button parts of the kit.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        ButtonTheme::default()
            .root(button)
            .content(button_content)
            .busy(button_busy)
            .icon(button_icon),
    )
}
