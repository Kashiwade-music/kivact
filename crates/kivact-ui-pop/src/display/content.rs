//! Role: Draw an application-owned sparkline for chart slots and standalone use.
use crate::Tokens;
use kivact::draw::{LineCap, LineJoin, Path, Stroke, linear_gradient};
use kivact::geometry::Point;
use kivact::state::IntoValue;
use kivact::style::{Color, pct, style};
use kivact::view::node;

#[track_caller]
pub fn sparkline(points: impl IntoValue<Vec<f64>>) -> impl kivact::view::Component {
    let points = points.into_value();
    node()
        .style(style().height(52).min_width(1))
        .draw(move |d, s| {
            let points = points.get();
            if points.len() < 2 {
                return;
            }
            let min = points.iter().copied().fold(f64::INFINITY, f64::min);
            let max = points.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let p = d.theme().ext::<Tokens>();
            let mut line = Path::new();
            let mut area = kivact::draw::Path::new().move_to(Point::new(0.0, s.height));
            for (i, v) in points.iter().enumerate() {
                let pt = Point::new(
                    s.width * i as f64 / (points.len() - 1) as f64,
                    6.0 + (1.0 - (*v - min) / (max - min).max(1.0)) * (s.height - 12.0),
                );
                line = if i == 0 {
                    line.move_to(pt)
                } else {
                    line.line_to(pt)
                };
                area = area.line_to(pt);
            }
            d.fill_path(
                &area.line_to(Point::new(s.width, s.height)).close(),
                linear_gradient(180)
                    .stop(kivact::style::rgba(0x2b8bd8, 0.32), pct(0))
                    .stop(Color::TRANSPARENT, pct(100)),
            );
            d.stroke_path(
                &line,
                &Stroke::new(2.4).cap(LineCap::Round).join(LineJoin::Round),
                p.blue,
            );
        })
}
