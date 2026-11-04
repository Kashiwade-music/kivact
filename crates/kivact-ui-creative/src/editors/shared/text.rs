//! Role: Text drawing shared by editor surfaces and time rulers.

use kivact::draw::{Draw, TextFormat};
use kivact::geometry::Point;
use kivact::style::Color;

pub(crate) fn label(d: &mut Draw, text: &str, x: f64, y: f64, size: f64, color: Color) {
    d.text(
        text,
        Point::new(x, y),
        TextFormat {
            size,
            color,
            ..TextFormat::default()
        },
    );
}
