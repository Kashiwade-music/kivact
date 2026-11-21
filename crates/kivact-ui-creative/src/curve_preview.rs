//! Role: A read-only finite sampled easing curve with time and progress axes.
use crate::bezier::geometry::Plane;
use crate::color::{ACCENT, DIM, HAIR, TEXT};
use kivact::component;
use kivact::draw::{Path, Stroke, TextFormat};
use kivact::geometry::Point;
use kivact::i18n::message;
use kivact::state::Value;
use kivact::style::style;
use kivact::view::{Role, canvas};

/// Draw up to 513 finite samples without writing application state.
/// Points map normalized time (X) to progress (Y).
/// X is clamped to 0..=1. Non-finite points and Y outside +/-1,000,000 are
/// discarded; finite overshoot is included in the progress range. The label
/// and axes use `curve_preview.label`, `.time` and `.progress` message keys.
#[component]
pub fn curve_preview(samples: Value<Vec<Point>>) -> impl Element {
    let time = message("creative.curve_preview.time", "Time");
    let progress = message("creative.curve_preview.progress", "Progress");
    canvas(move |draw, size| {
        let samples = samples
            .get()
            .into_iter()
            .filter(|p| p.x.is_finite() && p.y.is_finite() && p.y.abs() <= 1e6)
            .take(513)
            .map(|p| Point::new(p.x.clamp(0.0, 1.0), p.y))
            .collect::<Vec<_>>();
        let low = samples.iter().fold(0.0_f64, |low, p| low.min(p.y));
        let high = samples.iter().fold(1.0_f64, |high, p| high.max(p.y));
        let area = Plane::new(size, low, high);
        for y in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let at = area.point(0.0, y);
            draw.line(at, area.point(1.0, y), 1.0, HAIR);
            draw.text(
                &format!("{y:.2}"),
                Point::new(0.0, at.y),
                TextFormat {
                    size: 9.0,
                    color: DIM,
                    ..Default::default()
                },
            );
        }
        for x in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let bottom = area.point(x, low);
            let top = area.point(x, high);
            draw.line(top, bottom, 1.0, HAIR);
            draw.text(
                &format!("{x:.2}"),
                bottom,
                TextFormat {
                    size: 9.0,
                    color: DIM,
                    ..Default::default()
                },
            );
        }
        let bottom = area.point(0.5, low);
        draw.text(
            &time.get(),
            Point::new(bottom.x, bottom.y + 9.0),
            TextFormat {
                size: 10.0,
                color: TEXT,
                ..Default::default()
            },
        );
        draw.text(
            &progress.get(),
            area.point(0.0, high),
            TextFormat {
                size: 10.0,
                color: TEXT,
                ..Default::default()
            },
        );
        let mut path = Path::new();
        for (i, point) in samples.into_iter().enumerate() {
            let point = area.point(point.x, point.y);
            path = if i == 0 {
                path.move_to(point)
            } else {
                path.line_to(point)
            };
        }
        draw.stroke_path(&path, &Stroke::new(2), ACCENT);
    })
    .role(Role::Canvas)
    .name(message("creative.curve_preview.label", "Curve preview"))
    .style(style().size(240, 160).min_width(0).min_height(0))
}
