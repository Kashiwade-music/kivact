//! Role: Draw application-owned trend samples without owning statistic state.
use crate::Tokens;
use kivact::draw::{Path, Stroke};
use kivact::geometry::Point;
use kivact::state::{IntoValue, Value};
use kivact::style::style;
use kivact::view::{Node, node};

/// A compact trend chart for a statistic's application-owned chart slot.
#[track_caller]
pub fn sparkline(samples: impl IntoValue<Vec<f64>>) -> Node {
    let samples: Value<Vec<f64>> = samples.into_value();
    node()
        .hidden(move || samples.get().is_empty())
        .style(style().size(96, 28))
        .draw(move |d, s| {
            let values = samples.get();
            if values.is_empty() {
                return;
            }
            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let span = (max - min).max(f64::EPSILON);
            let points: Vec<Point> = values
                .iter()
                .enumerate()
                .map(|(i, y)| {
                    Point::new(
                        i as f64 * s.width / values.len().saturating_sub(1).max(1) as f64,
                        s.height * (1.0 - ((y - min) / span)),
                    )
                })
                .collect();
            let mut area = Path::new().move_to(Point::new(0., s.height));
            for point in &points {
                area = area.line_to(*point);
            }
            d.fill_path(
                &area.line_to(Point::new(s.width, s.height)).close(),
                d.theme().ext::<Tokens>().primary_weak,
            );
            d.polyline(points, &Stroke::new(1.5), d.theme().colors().accent);
        })
}
