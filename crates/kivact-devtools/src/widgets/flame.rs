//! Role: The flame chart of recorded turns and frames.

use kivact::draw::{TextBaseline, TextFormat};
use kivact::prelude::*;
use kivact::view::Node;
use kivact_devtools_protocol as p;

/// The height of one row, band and label.
const ROW_HEIGHT: f64 = 20.0;
/// The height of one stage band.
const BAND_HEIGHT: f64 = 12.0;
/// The width of the labels left of the bands.
const LABEL_WIDTH: f64 = 54.0;
/// The most rows of each kind the chart shows.
const MAX_ROWS: usize = 4;

/// The stage bands of the newest turns and frames.
pub(crate) fn flame_chart(
    turns: Signal<Vec<p::TurnGraph>>,
    frames: Value<Vec<p::FrameInfo>>,
) -> Node {
    canvas(move |draw, size| {
        paint(draw, size, &turns.get(), &frames.get());
    })
    .name("Flame chart")
    .style(
        style()
            .width(pct(100))
            .height(220)
            .background(kivact::style::color::SURFACE_ALT),
    )
}

/// Paint the turn and frame rows.
fn paint(draw: &mut Draw, size: Size, turns: &[p::TurnGraph], frames: &[p::FrameInfo]) {
    let width = (size.width - LABEL_WIDTH - 8.0).max(1.0);
    if turns.is_empty() && frames.is_empty() {
        draw.text(
            "No turns or frames recorded",
            Point::new(8.0, 8.0),
            text_style(12.0, kivact::style::color::TEXT_MUTED),
        );
        return;
    }
    let mut y = 18.0;
    if !turns.is_empty() {
        draw.text(
            "Turns",
            Point::new(3.0, 3.0),
            text_style(11.0, kivact::style::color::TEXT_MUTED),
        );
        let shown = tail(turns);
        let max_ns = shown
            .iter()
            .map(|turn| turn.duration_ns)
            .max()
            .unwrap_or(1)
            .max(1);
        for turn in shown {
            draw.text(
                &format!("#{}", turn.turn),
                Point::new(3.0, y),
                text_style(10.0, kivact::style::color::TEXT_MUTED),
            );
            let rect = Rect::new(LABEL_WIDTH, y, width, BAND_HEIGHT);
            paint_turn(draw, rect, turn, max_ns);
            y += ROW_HEIGHT;
        }
        y += 4.0;
    }
    if !frames.is_empty() {
        draw.text(
            "Frames (ms)",
            Point::new(3.0, y),
            text_style(11.0, kivact::style::color::TEXT_MUTED),
        );
        y += 14.0;
        let shown = tail(frames);
        let max_ns = shown
            .iter()
            .map(|frame| frame.work_ns)
            .max()
            .unwrap_or(1)
            .max(1);
        for frame in shown {
            draw.text(
                &format!("{:.1}", ns_to_ms(frame.work_ns)),
                Point::new(3.0, y),
                text_style(10.0, kivact::style::color::TEXT_MUTED),
            );
            let rect = Rect::new(LABEL_WIDTH, y, width, BAND_HEIGHT);
            paint_frame(draw, rect, frame, max_ns);
            y += ROW_HEIGHT;
        }
    }
}

/// The newest rows of `items`.
fn tail<T>(items: &[T]) -> &[T] {
    &items[items.len().saturating_sub(MAX_ROWS)..]
}

/// Paint the stages of one turn.
fn paint_turn(draw: &mut Draw, rect: Rect, turn: &p::TurnGraph, max_ns: u64) {
    draw.fill_rect(rect, kivact::style::color::SURFACE);
    for stage in &turn.stages {
        let offset = stage.start_ns.saturating_sub(turn.started_ns);
        paint_band(
            draw,
            rect,
            offset,
            stage.duration_ns,
            max_ns,
            stage_label(stage.stage),
            stage_color(stage.stage),
        );
    }
}

/// Paint the scene, raster and present stages of one frame.
fn paint_frame(draw: &mut Draw, rect: Rect, frame: &p::FrameInfo, max_ns: u64) {
    draw.fill_rect(rect, kivact::style::color::SURFACE);
    let mut offset = 0;
    for (label, ns, token) in [
        ("scene", frame.scene_ns, kivact::style::color::ACCENT),
        ("raster", frame.raster_ns, kivact::style::color::WARNING),
        ("present", frame.present_ns, kivact::style::color::SUCCESS),
    ] {
        paint_band(draw, rect, offset, ns, max_ns, label, token);
        offset += ns;
    }
    let other = frame.work_ns.saturating_sub(offset);
    if other > 0 {
        paint_band(
            draw,
            rect,
            offset,
            other,
            max_ns,
            "other",
            kivact::style::color::TEXT_MUTED,
        );
    }
}

/// Paint one stage band into `rect` at `offset` of `max_ns`.
fn paint_band(
    draw: &mut Draw,
    rect: Rect,
    offset: u64,
    duration_ns: u64,
    max_ns: u64,
    label: &str,
    token: Color,
) {
    let x = rect.x + fraction(offset, max_ns) * rect.width;
    let width = (fraction(duration_ns, max_ns) * rect.width).min(rect.right() - x);
    if width <= 0.0 {
        return;
    }
    let band = Rect::new(x, rect.y, (width - 1.0).max(1.0), rect.height);
    draw.fill_rect(band, token);
    if band.width >= 30.0 {
        draw.text(
            label,
            Point::new(band.x + 3.0, band.y + band.height / 2.0),
            TextFormat {
                size: 9.0,
                baseline: TextBaseline::Middle,
                color: kivact::style::color::ACCENT_TEXT,
                ..TextFormat::default()
            },
        );
    }
    draw.stroke_rect(band, Stroke::new(1.0), kivact::style::color::SURFACE);
}

/// The name of a stage.
fn stage_label(stage: p::TurnStage) -> &'static str {
    match stage {
        p::TurnStage::Begin => "begin",
        p::TurnStage::Reports => "reports",
        p::TurnStage::Dispatch => "dispatch",
        p::TurnStage::Async => "async",
        p::TurnStage::Commit => "commit",
        p::TurnStage::Mounts => "mounts",
        p::TurnStage::UserEffects => "effects",
        p::TurnStage::Output => "output",
    }
}

/// The color of a stage band.
fn stage_color(stage: p::TurnStage) -> Color {
    match stage {
        p::TurnStage::Begin => kivact::style::color::ACCENT,
        p::TurnStage::Reports => kivact::style::color::FOCUS,
        p::TurnStage::Dispatch => kivact::style::color::WARNING,
        p::TurnStage::Async => kivact::style::color::SUCCESS,
        p::TurnStage::Commit => kivact::style::color::SELECTION,
        p::TurnStage::Mounts => kivact::style::color::TEXT_MUTED,
        p::TurnStage::UserEffects => kivact::style::color::DANGER,
        p::TurnStage::Output => kivact::style::color::BORDER,
    }
}

/// Nanoseconds as milliseconds.
fn ns_to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000.0
}

/// `value / max` clamped to `0.0..=1.0`.
fn fraction(value: u64, max: u64) -> f64 {
    if max == 0 {
        return 0.0;
    }
    (value as f64 / max as f64).clamp(0.0, 1.0)
}

/// A small label style.
fn text_style(size: f64, token: Color) -> TextFormat {
    TextFormat {
        size,
        color: token,
        ..TextFormat::default()
    }
}
