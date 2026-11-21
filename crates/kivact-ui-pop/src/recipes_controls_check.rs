//! Role: The Pop looks of the check boxes, switches and radio groups.

use crate::color;
use crate::recipe_parts::{check_style, choice_list, choice_option};
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::style::{Color, FlexDirection, Position, Style, Theme, pct, style};
use kivact::ui::RadioGroupVariant;
use kivact::ui::theme::{CheckableState, CheckboxTheme, Look, RadioGroupTheme, SwitchTheme};
use kivact::view::{CheckedState, Orientation};

/// The size of a check box's box and the room a switch's track takes.
const CHECK: (f64, f64) = (16.0, 16.0);
const SWITCH: (f64, f64) = (32.0, 18.0);
/// The gap between the drawn part and the label, as in `ui::checkbox`.
const GAP: f64 = 12.0;

/// Install the kit's recipes of the check boxes, switches and radio groups.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            CheckboxTheme::default()
                .root(checkbox)
                .indicator(checkbox_indicator)
                .check(checkbox_check)
                .mixed(checkbox_mixed),
        )
        .with(
            SwitchTheme::default()
                .root(switch)
                .track(switch_track)
                .thumb(switch_thumb),
        );
    theme.with(
        RadioGroupTheme::default()
            .group(radio_group_layout)
            .item(radio_item)
            .indicator(radio_indicator)
            .description(radio_description),
    )
}

/// A kit check row with the room its drawn part needs on the left.
fn row(drawn: (f64, f64)) -> Style {
    check_style().font_weight(500).padding_left(drawn.0 + GAP)
}

fn checkbox(_theme: &Theme, _: &Look<CheckableState>, _base: Style) -> Style {
    row(CHECK)
}

fn switch(_theme: &Theme, _: &Look<CheckableState>, _base: Style) -> Style {
    row(SWITCH)
}

fn indicator(width: f64, height: f64) -> Style {
    style()
        .position(Position::Absolute)
        .left(0)
        .top(pct(50))
        .translate(0, -height / 2.0)
        .size(width, height)
}

fn checkbox_indicator(_theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let active = look.checked() != CheckedState::Unchecked;
    indicator(16.0, 16.0)
        .border_radius(4)
        .background(if active { color::BLUE } else { color::SURFACE })
        .border(
            1,
            if active {
                color::BLUE
            } else {
                color::LINE_STRONG
            },
        )
}

fn checkbox_check(_theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let checked = look.is_checked();
    style()
        .position(Position::Absolute)
        .inset(0)
        .paint(move |d, _| {
            if checked {
                d.stroke_path(
                    &Path::new()
                        .move_to(Point::new(2.5, 7.0))
                        .line_to(Point::new(5.5, 10.0))
                        .line_to(Point::new(11.5, 3.5)),
                    &Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round),
                    Color::WHITE,
                );
            }
        })
}

fn checkbox_mixed(_theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    let mixed = look.is_mixed();
    style()
        .position(Position::Absolute)
        .inset(0)
        .paint(move |d, _| {
            if mixed {
                d.line(
                    Point::new(3.0, 7.0),
                    Point::new(11.0, 7.0),
                    2.0,
                    Color::WHITE,
                );
            }
        })
}

fn switch_track(_theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    indicator(SWITCH.0, SWITCH.1)
        .border_radius(9)
        .background(if look.is_checked() {
            color::BLUE
        } else {
            color::LINE_STRONG
        })
}

fn switch_thumb(_theme: &Theme, look: &Look<CheckableState>, _base: Style) -> Style {
    style()
        .position(Position::Absolute)
        .top(2)
        .left(if look.is_checked() { 16 } else { 2 })
        .size(14, 14)
        .border_radius(7)
        .background(Color::WHITE)
}

/// The layout of a radio group: a segmented track, a column or a wrapping row.
fn radio_group_layout(_theme: &Theme, orientation: &Look<Orientation>, _base: Style) -> Style {
    if matches!(
        orientation.variant::<RadioGroupVariant>(),
        Some(RadioGroupVariant::Segmented)
    ) {
        return choice_list(true).flex_direction(FlexDirection::Row);
    }
    let s = style().gap(12);
    match **orientation {
        Orientation::Vertical => s.flex_direction(FlexDirection::Column),
        _ => s.flex_direction(FlexDirection::Row).flex_wrap(true),
    }
}

/// A radio row: the indicator beside the label.
fn radio_item(_theme: &Theme, s: &Look<CheckableState>, _base: Style) -> Style {
    if matches!(
        s.variant::<RadioGroupVariant>(),
        Some(RadioGroupVariant::Segmented)
    ) {
        return choice_option(true);
    }
    check_style().font_weight(500)
}

/// The 16 px ring of an option, the accent around the dot while chosen.
fn radio_indicator(_theme: &Theme, s: &Look<CheckableState>, _base: Style) -> Style {
    if matches!(
        s.variant::<RadioGroupVariant>(),
        Some(RadioGroupVariant::Segmented)
    ) {
        // A segment shows no indicator: out of the row, with no size.
        return style().position(Position::Absolute).size(0, 0).clip(true);
    }
    let on = s.is_checked();
    style()
        .size(16, 16)
        .flex_shrink(0)
        .border_radius(8)
        .flex_direction(FlexDirection::Row)
        .align_items(kivact::style::Align::Center)
        .justify_content(kivact::style::Justify::Center)
        .background(color::SURFACE)
        .border(1, if on { color::BLUE } else { color::LINE_STRONG })
        .color(color::BLUE)
}

fn radio_description(_theme: &Theme, _: &Look<CheckableState>, _base: Style) -> Style {
    style().font_size(13).color(color::TEXT_MUTED)
}
