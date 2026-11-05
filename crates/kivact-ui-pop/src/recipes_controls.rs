//! Role: Style persistent Pop check, switch and split control parts.
use crate::Tokens;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::style::{Align, Color, Cursor, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{AxisState, CheckableState, Look};
use kivact::view::CheckedState;
use kivact::view::Orientation;

fn indicator(width: f64, height: f64) -> Style {
    style()
        .position(Position::Absolute)
        .left(0)
        .top(pct(50))
        .translate(0, -height / 2.0)
        .size(width, height)
}

pub(crate) fn checkbox_indicator(t: &Theme, look: &Look<CheckableState>) -> Style {
    let p = t.ext::<Tokens>();
    let active = look.checked != CheckedState::Unchecked;
    indicator(16.0, 16.0)
        .radius(4)
        .background(if active { p.blue } else { p.surface })
        .border(1, if active { p.blue } else { p.line_strong })
}

pub(crate) fn checkbox_check(_: &Theme, look: &Look<CheckableState>) -> Style {
    let checked = look.checked == CheckedState::Checked;
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

pub(crate) fn checkbox_mixed(_: &Theme, look: &Look<CheckableState>) -> Style {
    let mixed = look.checked == CheckedState::Indeterminate;
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

pub(crate) fn switch_track(t: &Theme, look: &Look<CheckableState>) -> Style {
    let p = t.ext::<Tokens>();
    indicator(32.0, 18.0)
        .radius(9)
        .background(if look.checked.is_checked() {
            p.blue
        } else {
            p.line_strong
        })
}

pub(crate) fn switch_thumb(_: &Theme, look: &Look<CheckableState>) -> Style {
    style()
        .position(Position::Absolute)
        .top(2)
        .left(if look.checked.is_checked() { 16 } else { 2 })
        .size(14, 14)
        .radius(7)
        .background(Color::WHITE)
}

pub(crate) fn splitter_handle(t: &Theme, look: &Look<AxisState>) -> Style {
    let p = t.ext::<Tokens>();
    let s = style()
        .shrink(0)
        .direction(Direction::Row)
        .justify_content(Justify::Center)
        .background(p.line)
        .hover(style().background(p.blue))
        .pressed(style().background(p.blue));
    match look.orientation {
        Orientation::Vertical => s.height(6).cursor(Cursor::NsResize),
        _ => s.width(6).cursor(Cursor::EwResize),
    }
}

pub(crate) fn split_grip(t: &Theme, look: &Look<AxisState>) -> Style {
    let s = style()
        .shrink(0)
        .align_self(Align::Center)
        .radius(2)
        .background(t.ext::<Tokens>().line_strong);
    match look.orientation {
        Orientation::Vertical => s.size(24, 4),
        _ => s.size(4, 24),
    }
}
