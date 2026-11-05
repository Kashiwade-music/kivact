//! Role: Render Pop loading indicators through canonical UI recipes.
use crate::Tokens;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::style::{ColorToken, Style, Theme, pct, style};
use kivact::ui::{ControlSize, Look, SkeletonShape, SkeletonState, SpinnerState, Variant};

/// A three-color bouncing dot loading indicator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinnerLook {
    Dots,
}
impl Variant for SpinnerLook {}

pub(crate) fn skeleton(_: &Theme, s: &Look<SkeletonState>) -> Style {
    let base = style()
        .grow(1)
        .min_height(16)
        .width(pct(100))
        .background(ColorToken::SurfaceAlt)
        .opacity(s.pulse);
    match s.shape {
        SkeletonShape::Circle => base.size(36, 36).radius(999),
        SkeletonShape::Rect => base.radius(0),
        _ => base.radius(6),
    }
}
pub(crate) fn spinner(t: &Theme, s: &Look<SpinnerState>) -> Style {
    let p = t.ext::<Tokens>();
    let phase = s.turn.to_radians();
    if s.variant::<SpinnerLook>() == Some(&SpinnerLook::Dots) {
        let colors = [p.blue, p.cyan, p.yellow];
        return style().size(34, 20).paint(move |d, _| {
            for (i, color) in colors.into_iter().enumerate() {
                let bounce = (phase - i as f64 * 0.95).sin().max(0.0) * 6.0;
                d.rounded_rect(
                    Rect::new(i as f64 * 13.0, 10.0 - bounce, 8.0, 8.0),
                    4.0,
                    color,
                );
            }
        });
    }
    let side = match s.size {
        ControlSize::ExtraSmall => 16,
        ControlSize::Small => 18,
        ControlSize::Medium => 20,
        ControlSize::Large => 32,
    };
    let (track, accent, turn) = (p.surface_active, p.blue, s.turn);
    style()
        .size(side, side)
        .radius(999)
        .rotate(turn)
        .paint(move |d, size| {
            let thickness = if size.width.min(size.height) >= 28.0 {
                3.0
            } else {
                2.0
            };
            let center = size.to_rect().center();
            let radius = (size.width.min(size.height) - thickness) / 2.0;
            let arc = |fraction: f64| {
                let mut path = Path::new();
                for i in 0..=64 {
                    let angle = -std::f64::consts::FRAC_PI_2
                        + std::f64::consts::TAU * fraction * i as f64 / 64.0;
                    let point = Point::new(
                        center.x + radius * angle.cos(),
                        center.y + radius * angle.sin(),
                    );
                    path = if i == 0 {
                        path.move_to(point)
                    } else {
                        path.line_to(point)
                    };
                }
                path
            };
            let stroke = Stroke::new(thickness)
                .cap(LineCap::Round)
                .join(LineJoin::Round);
            d.stroke_path(&arc(1.0), &stroke, track);
            d.stroke_path(&arc(0.25), &stroke, accent);
        })
}
