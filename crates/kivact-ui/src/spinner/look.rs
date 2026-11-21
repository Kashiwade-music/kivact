//! Role: The spinner recipe state, tokens, recipe table and default looks.
use crate::look_common::side;

use crate::control_size::ControlSize;
use kivact_core::draw::{LineCap, Path, Stroke};
use kivact_core::geometry::Point;
use kivact_core::style::{Style, Theme, style};
pub(crate) fn spinner(_: &Theme, s: &SpinnerState) -> Style {
    let side = side(s.size) * 0.6;
    let turn = s.turn;
    style()
        .size(side, side)
        .flex_shrink(0)
        .paint(move |d, size| {
            let center = Point::new(size.width / 2.0, size.height / 2.0);
            let arc = Path::new().arc(
                center,
                size.width.min(size.height) / 2.0 - 2.0,
                turn,
                turn + 270.0,
                false,
            );
            d.stroke_path(
                &arc,
                &Stroke::new(2.0).cap(LineCap::Round),
                kivact_core::style::color::TEXT,
            );
        })
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpinnerState {
    pub size: ControlSize,
    pub turn: f64,
}
