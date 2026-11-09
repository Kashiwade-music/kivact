//! Role: The timeline of recorded turns and frames.

use kivact::draw::{Draw, RegionId, Stroke, TextFormat};
use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact_devtools_protocol as p;

/// The width of the lane labels on the left.
const LABEL_WIDTH: f64 = 64.0;
/// The padding around the lanes.
const PAD: f64 = 8.0;
/// The gap between two lanes.
const GAP: f64 = 6.0;
/// The height of the input lane.
const INPUT_HEIGHT: f64 = 14.0;
/// The height of the updates lane.
const UPDATE_HEIGHT: f64 = 26.0;
/// The height of the frames lane.
const FRAME_HEIGHT: f64 = 50.0;
/// The height the frame period takes in the frames lane.
const PERIOD_HEIGHT: f64 = 32.0;
/// A bar is at least this wide, so that its turn can be picked.
const MIN_BAR: f64 = 2.0;
/// The width of a frame bar.
const FRAME_BAR: f64 = 4.0;

/// The time lanes of the Updates panel.
pub(crate) fn timeline(
    turns: Signal<Vec<p::TurnGraph>>,
    frames: Signal<Vec<p::FrameInfo>>,
    selected: Signal<Option<u64>>,
) -> impl IntoView {
    canvas(move |draw, size| {
        draw_timeline(draw, size, &turns.get(), &frames.get(), selected.get());
    })
    .regions(
        turns,
        |turn| RegionId(turn.turn.max(1)),
        move |_, turn| {
            let state = memo(move || turn.get().turn);
            kivact::custom::Part::new(state, move |node| {
                node.role(Role::Button)
                    .name(move || format!("Turn {}", turn.get_untracked().turn))
                    .focusable(true)
                    .on_press(move || selected.set(Some(turn.get_untracked().turn)))
            })
        },
    )
    .role(Role::Canvas)
    .name("Timeline")
    .style(style().width(pct(100)).height(132.0))
}

/// Paint the three lanes.
fn draw_timeline(
    draw: &mut Draw,
    size: Size,
    turns: &[p::TurnGraph],
    frames: &[p::FrameInfo],
    selected: Option<u64>,
) {
    let top = PAD;
    let input = lane(draw, size, "Input", top, INPUT_HEIGHT);
    let updates = lane(
        draw,
        size,
        "Updates",
        input.y + input.height + GAP,
        UPDATE_HEIGHT,
    );
    let frame_lane = lane(
        draw,
        size,
        "Frames",
        updates.y + updates.height + GAP,
        FRAME_HEIGHT,
    );
    if turns.is_empty() && frames.is_empty() {
        draw_label(draw, Point::new(LABEL_WIDTH, top), "No turns recorded");
        return;
    }
    let (start, span) = time_range(turns, frames);
    draw_input(draw, input, start, span, turns);
    draw_updates(draw, updates, start, span, turns, selected);
    draw_frames(draw, frame_lane, start, span, frames);
}

/// Paint one lane's background and label, and return its plot rectangle.
fn lane(draw: &mut Draw, size: Size, label: &str, y: f64, height: f64) -> Rect {
    draw_label(draw, Point::new(PAD, y + (height - 10.0) / 2.0), label);
    let x = LABEL_WIDTH;
    let width = (size.width - LABEL_WIDTH - PAD).max(1.0);
    let rect = Rect::new(x, y, width, height);
    draw.fill_rect(rect, Color::token(ColorToken::SurfaceAlt));
    rect
}

/// Paint the input marks of every turn.
fn draw_input(draw: &mut Draw, plot: Rect, start: f64, span: f64, turns: &[p::TurnGraph]) {
    for turn in turns {
        for _ in turn
            .nodes
            .iter()
            .filter(|node| node.kind == p::CauseKind::Input)
        {
            let x = x_at(plot, start, span, turn.started_ns as f64) - 2.0;
            let rect = Rect::new(x, plot.y + 3.0, 5.0, plot.height - 6.0);
            draw.fill_rect(rect, Color::token(ColorToken::Accent));
        }
    }
}

/// Paint one bar per turn, colored by its first cause.
fn draw_updates(
    draw: &mut Draw,
    plot: Rect,
    start: f64,
    span: f64,
    turns: &[p::TurnGraph],
    selected: Option<u64>,
) {
    for turn in turns {
        let x = x_at(plot, start, span, turn.started_ns as f64);
        let end = x_at(
            plot,
            start,
            span,
            turn.started_ns.saturating_add(turn.duration_ns) as f64,
        );
        let width = (end - x).max(MIN_BAR);
        let rect = Rect::new(
            x.min(plot.x + plot.width - width),
            plot.y + 2.0,
            width,
            plot.height - 4.0,
        );
        let kind = turn
            .nodes
            .iter()
            .find(|node| is_cause(node.kind))
            .map(|node| node.kind);
        draw.fill_rect(rect, Color::token(cause_color(kind)));
        if selected == Some(turn.turn) {
            draw.stroke_rect(rect, 2.0, Color::token(ColorToken::Accent));
        }
        draw.region(RegionId(turn.turn.max(1)), rect);
    }
}

/// Paint the frame bars against the frame period.
fn draw_frames(draw: &mut Draw, plot: Rect, start: f64, span: f64, frames: &[p::FrameInfo]) {
    let period = frames
        .first()
        .map_or(16_666_666, |frame| frame.frame_period_ns)
        .max(1);
    let base = plot.y + plot.height;
    let budget = base - PERIOD_HEIGHT;
    let dashed = Stroke::new(1.0).dash([4.0, 4.0]);
    draw.line(
        Point::new(plot.x, budget),
        Point::new(plot.x + plot.width, budget),
        dashed,
        Color::token(ColorToken::TextMuted),
    );
    for frame in frames {
        let height = (frame.work_ns as f64 / period as f64 * PERIOD_HEIGHT)
            .clamp(MIN_BAR, plot.height - 2.0);
        let raw = x_at(plot, start, span, frame.end_ns as f64) - FRAME_BAR / 2.0;
        let room = (plot.width - FRAME_BAR).max(0.0);
        let x = plot.x + (raw - plot.x).clamp(0.0, room);
        let color = if frame.over_budget || frame.work_ns > frame.frame_period_ns {
            ColorToken::Danger
        } else {
            ColorToken::Accent
        };
        draw.fill_rect(
            Rect::new(x, base - height, FRAME_BAR, height),
            Color::token(color),
        );
    }
}

/// The first and last time of the lanes, and the length between them.
fn time_range(turns: &[p::TurnGraph], frames: &[p::FrameInfo]) -> (f64, f64) {
    let mut start = f64::INFINITY;
    let mut end = f64::NEG_INFINITY;
    for turn in turns {
        start = start.min(turn.started_ns as f64);
        end = end.max(turn.started_ns.saturating_add(turn.duration_ns) as f64);
    }
    for frame in frames {
        start = start.min(frame.end_ns.saturating_sub(frame.work_ns) as f64);
        end = end.max(frame.end_ns as f64);
    }
    if !start.is_finite() || !end.is_finite() || end <= start {
        return (0.0, 1.0);
    }
    (start, end - start)
}

/// The x of `ns` on the plot.
fn x_at(plot: Rect, start: f64, span: f64, ns: f64) -> f64 {
    plot.x + (ns - start) / span * plot.width
}

/// Paint one short label in the muted text color.
fn draw_label(draw: &mut Draw, at: Point, label: &str) {
    draw.text(
        label,
        at,
        TextFormat {
            size: 10.0,
            color: Color::token(ColorToken::TextMuted),
            ..TextFormat::default()
        },
    );
}

/// Whether a cause kind starts a turn.
fn is_cause(kind: p::CauseKind) -> bool {
    matches!(
        kind,
        p::CauseKind::Input
            | p::CauseKind::Timer
            | p::CauseKind::Task
            | p::CauseKind::Mount
            | p::CauseKind::Devtools
    )
}

/// The color of a turn by the kind of its first cause.
fn cause_color(kind: Option<p::CauseKind>) -> ColorToken {
    match kind {
        Some(p::CauseKind::Input) => ColorToken::Accent,
        Some(p::CauseKind::Timer) => ColorToken::Warning,
        Some(p::CauseKind::Task) => ColorToken::Focus,
        _ => ColorToken::TextMuted,
    }
}
