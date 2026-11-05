//! Role: Lyra circular and segmented progress recipes.
use crate::Tokens;
use kivact::draw::{LineCap, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::style::{Align, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{ControlSize, Look, ProgressState, ProgressVariant};
fn height(s: &Look<ProgressState>) -> f64 {
    match s.size {
        ControlSize::ExtraSmall => 3.0,
        ControlSize::Small => 4.0,
        ControlSize::Medium => 6.0,
        ControlSize::Large => 10.0,
    }
}
pub(crate) fn track(t: &Theme, s: &Look<ProgressState>) -> Style {
    let muted = t.ext::<Tokens>().muted;
    if s.variant::<ProgressVariant>() == Some(&ProgressVariant::Ring) {
        style()
            .size(height(s) * 12.0, height(s) * 12.0)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .paint(move |d, size| {
                let r = size.width.min(size.height) * 0.4;
                d.stroke_path(
                    &Path::new().arc(size.to_rect().center(), r, 0.0, 359.99, false),
                    &Stroke::new(r / 5.0),
                    muted,
                );
            })
    } else {
        style()
            .width(pct(100))
            .height(height(s) * 3.0)
            .min_width(0)
            .paint(move |d, size| {
                for i in 0..20 {
                    d.fill_rect(
                        Rect::new(
                            i as f64 * size.width / 20.0,
                            0.0,
                            size.width / 20.0 * 0.75,
                            size.height,
                        ),
                        muted,
                    );
                }
            })
    }
}
pub(crate) fn fill(t: &Theme, s: &Look<ProgressState>) -> Style {
    let state = **s;
    let ring = s.variant::<ProgressVariant>() == Some(&ProgressVariant::Ring);
    let primary = t.ext::<Tokens>().primary;
    style()
        .position(Position::Absolute)
        .inset(0)
        .paint(move |d, size| {
            let (start, len) = if state.value.is_some() {
                (0.0, state.shown.get().clamp(0.0, 1.0))
            } else {
                state.sweep()
            };
            if ring {
                let r = size.width.min(size.height) * 0.4;
                if len > 0.0 {
                    d.stroke_path(
                        &Path::new().arc(
                            Point::new(size.width / 2.0, size.height / 2.0),
                            r,
                            -90.0 + start * 360.0,
                            -90.0 + (start + len.min(0.9999)) * 360.0,
                            false,
                        ),
                        &Stroke::new(r / 5.0).cap(LineCap::Round),
                        primary,
                    );
                }
            } else {
                for i in 0..20 {
                    let f = (i as f64 + 0.5) / 20.0;
                    if f >= start && f <= start + len {
                        d.fill_rect(
                            Rect::new(
                                i as f64 * size.width / 20.0,
                                0.0,
                                size.width / 20.0 * 0.75,
                                size.height,
                            ),
                            primary,
                        );
                    }
                }
            }
        })
}
