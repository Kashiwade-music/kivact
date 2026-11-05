//! Role: Draw the Pop empty-region illustration for the standard icon slot.
use crate::{Hue, Tokens};
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::style::style;
use kivact::view::{Node, node};

/// Decorative empty-region illustration, mounted through `ui::empty_state(..).icon(..)`.
pub fn empty_illustration() -> Node {
    node().base_style(style().size(160, 110)).draw(|d, _| {
        let p = d.theme().ext::<Tokens>();
        let mut halo = Path::new();
        for i in 0..=64 {
            let a = std::f64::consts::TAU * i as f64 / 64.0;
            let point = Point::new(80.0 + 34.0 * a.cos(), 22.0 + 9.0 * a.sin());
            halo = if i == 0 {
                halo.move_to(point)
            } else {
                halo.line_to(point)
            };
        }
        d.stroke_path(
            &halo,
            &Stroke::new(4.0).cap(LineCap::Round).join(LineJoin::Round),
            p.blue,
        );
        d.rounded_rect(
            Rect {
                x: 38.0,
                y: 42.0,
                width: 84.0,
                height: 56.0,
            },
            12.0,
            p.hue(Hue::Blue).0,
        );
        d.line(
            Point::new(38.0, 60.0),
            Point::new(122.0, 60.0),
            2.0,
            p.line_strong,
        );
        for x in [66.0, 94.0] {
            d.rounded_rect(
                Rect {
                    x: x - 3.5,
                    y: 74.5,
                    width: 7.0,
                    height: 7.0,
                },
                3.5,
                p.text_faint,
            );
        }
        d.stroke_path(
            &Path::new()
                .move_to(Point::new(72.0, 88.0))
                .quad_to(Point::new(80.0, 82.0), Point::new(88.0, 88.0)),
            &Stroke::new(2.5).cap(LineCap::Round).join(LineJoin::Round),
            p.text_faint,
        );
        for (x, y, r, color) in [(132.0, 40.0, 4.0, p.yellow), (27.0, 70.0, 3.0, p.cyan)] {
            d.fill_path(
                &Path::new()
                    .move_to(Point::new(x, y - r * 2.0))
                    .line_to(Point::new(x + r, y))
                    .line_to(Point::new(x, y + r * 2.0))
                    .line_to(Point::new(x - r, y))
                    .close(),
                color,
            );
        }
    })
}
