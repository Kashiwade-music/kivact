//! Role: Supply theme-based default recipes for metrics and description pairs.
use crate::{HeadingState, Look, ProgressState, ProgressVariant, StatState};
use kivact_core::draw::{LineCap, Path, Stroke};
use kivact_core::geometry::{Point, Rect};
use kivact_core::style::{
    Align, ColorToken, Direction, Justify, Position, Style, Theme, pct, style,
};

pub(super) fn heading(t: &Theme, s: &Look<HeadingState>) -> Style {
    let scale = match s.level {
        1 => 2.0,
        2 => 1.65,
        3 => 1.4,
        4 => 1.2,
        5 => 1.1,
        _ => 1.0,
    };
    style()
        .font_size(t.text().size * scale)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
}
pub(super) fn stat_root(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .padding(t.space().md)
        .min_width(0)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
}
pub(super) fn stat_label(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}
pub(super) fn stat_value(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.text().size * 2.0)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
}
pub(super) fn stat_delta(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}
pub(super) fn stat_slot(_: &Theme, _: &Look<StatState>) -> Style {
    style().min_width(0)
}
pub(super) fn description_root(t: &Theme, _: &Look<()>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .min_width(0)
}
pub(super) fn description_item(t: &Theme, _: &Look<()>) -> Style {
    style()
        .direction(Direction::Row)
        .gap(t.space().md)
        .align_items(Align::Center)
        .min_width(0)
}
pub(super) fn description_term(t: &Theme, _: &Look<()>) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
        .shrink(0)
}
pub(super) fn description_detail(_: &Theme, _: &Look<()>) -> Style {
    style().grow(1).min_width(0)
}

fn ring(s: &Look<ProgressState>) -> bool {
    s.variant::<ProgressVariant>() == Some(&ProgressVariant::Ring)
}
fn gauge(s: &Look<ProgressState>) -> bool {
    s.variant::<ProgressVariant>() == Some(&ProgressVariant::Gauge)
}
fn height(s: &ProgressState) -> f64 {
    match s.size {
        crate::ControlSize::ExtraSmall => 3.0,
        crate::ControlSize::Small => 4.0,
        crate::ControlSize::Medium => 6.0,
        crate::ControlSize::Large => 10.0,
    }
}
pub(super) fn progress_track(_: &Theme, s: &Look<ProgressState>) -> Style {
    if ring(s) {
        style()
            .size(height(s) * 12.0, height(s) * 12.0)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .paint(|d, size| {
                let radius = size.width.min(size.height) * 0.4;
                d.stroke_path(
                    &Path::new().arc(size.to_rect().center(), radius, 0.0, 359.99, false),
                    &Stroke::new(radius / 5.0),
                    ColorToken::Border,
                );
            })
    } else if gauge(s) {
        style()
            .width(pct(100))
            .height(height(s) * 3.0)
            .min_width(0)
            .paint(|d, size| {
                for i in 0..20 {
                    d.fill_rect(
                        Rect::new(
                            i as f64 * size.width / 20.0,
                            0.0,
                            size.width / 20.0 * 0.75,
                            size.height,
                        ),
                        ColorToken::Border,
                    );
                }
            })
    } else {
        style()
            .width(pct(100))
            .height(height(s))
            .min_width(0)
            .radius(height(s) / 2.0)
            .background(ColorToken::Border)
            .clip(true)
    }
}
pub(super) fn progress_fill(_: &Theme, s: &Look<ProgressState>) -> Style {
    let state = **s;
    let is_ring = ring(s);
    let is_gauge = gauge(s);
    style()
        .position(Position::Absolute)
        .inset(0)
        .paint(move |d, size| {
            let (start, length) = if state.value.is_some() {
                (0.0, state.shown.get().clamp(0.0, 1.0))
            } else {
                state.sweep()
            };
            if is_ring {
                let radius = size.width.min(size.height) * 0.4;
                let center = Point::new(size.width / 2.0, size.height / 2.0);
                if length > 0.0 {
                    d.stroke_path(
                        &Path::new().arc(
                            center,
                            radius,
                            -90.0 + start * 360.0,
                            -90.0 + (start + length.min(0.9999)) * 360.0,
                            false,
                        ),
                        &Stroke::new(radius / 5.0).cap(LineCap::Round),
                        ColorToken::Accent,
                    );
                }
            } else if is_gauge {
                for i in 0..20 {
                    let fraction = (i as f64 + 0.5) / 20.0;
                    if fraction >= start && fraction <= start + length {
                        d.fill_rect(
                            Rect::new(
                                i as f64 * size.width / 20.0,
                                0.0,
                                size.width / 20.0 * 0.75,
                                size.height,
                            ),
                            ColorToken::Accent,
                        );
                    }
                }
            } else {
                d.rounded_rect(
                    Rect::new(start * size.width, 0.0, length * size.width, size.height),
                    size.height / 2.0,
                    ColorToken::Accent,
                );
            }
        })
}
pub(super) fn progress_label(t: &Theme, _: &Look<ProgressState>) -> Style {
    style()
        .font_size(t.text().size_small)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
}
