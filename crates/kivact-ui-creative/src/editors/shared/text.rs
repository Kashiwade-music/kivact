//! Role: Text drawing shared by editor surfaces and time rulers.

use kivact::draw::{Draw, Point, TextStyle};
use kivact::style::Color;

pub(crate) fn label(d: &mut Draw, text: &str, x: f64, y: f64, size: f64, color: Color) {
    d.text(
        text,
        Point::new(x, y),
        TextStyle {
            size,
            color,
            ..TextStyle::default()
        },
    );
}
