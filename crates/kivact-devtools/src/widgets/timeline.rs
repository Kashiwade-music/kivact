//! Role: The timeline of recorded turns and frames, with a drag-to-zoom range.

use std::cell::Cell;
use std::rc::Rc;

use kivact::draw::{Draw, RegionId, Stroke, TextFormat};
use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact::view::{MouseButton, PointerEvent, PointerPhase};
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
/// A press that moves less than this many pixels is a click, not a drag.
const CLICK_SLOP: f64 = 4.0;

/// The time range the timeline shows.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Zoom {
    /// Every turn and frame.
    Full,
    /// The times from `start_ns` to `end_ns` (the app's clock, nanoseconds).
    Range {
        /// The first time shown.
        start_ns: u64,
        /// The last time shown.
        end_ns: u64,
    },
}

/// The primary button on the timeline.
#[derive(Clone, Copy, PartialEq)]
enum Drag {
    /// No button is held.
    Idle,
    /// The button went down at `from` and the pointer is now at `to` (both in node-local x).
    Held {
        /// Where the button went down.
        from: f64,
        /// Where the pointer is.
        to: f64,
    },
}

/// The time lanes of the Updates panel, with the reset button of the zoom under them.
pub(crate) fn timeline(
    turns: Signal<Vec<p::TurnGraph>>,
    frames: Signal<Vec<p::FrameInfo>>,
    selected: Signal<Option<u64>>,
) -> impl IntoView {
    let zoom = signal(Zoom::Full);
    let drag = signal(Drag::Idle);
    let width = Rc::new(Cell::new(1.0));
    let paint_width = width.clone();
    let pointing = Pointing {
        turns,
        frames,
        zoom,
        drag,
        width,
    };
    let plot = canvas(move |draw, size| {
        paint_width.set(plot_width(size));
        let turns = turns.get();
        let frames = frames.get();
        let (start, span) = view_range(zoom.get(), &turns, &frames);
        let scene = Scene {
            turns: &turns,
            frames: &frames,
            selected: selected.get(),
            start,
            span,
            drag: drag.get(),
        };
        draw_timeline(draw, size, &scene);
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
    .on_pointer(move |event: PointerEvent| pointing.handle(event))
    .role(Role::Canvas)
    .name("Timeline")
    .style(style().width(pct(100)).height(132.0));
    column((
        plot,
        row((
            spacer(),
            button("Reset zoom")
                .disabled(move || zoom.get() == Zoom::Full)
                .on_press(move || zoom.set(Zoom::Full)),
        ))
        .style(style().align_items(Align::Center)),
    ))
    .style(style().gap(2))
}

/// The signals and the plot width the pointer handler reads.
#[derive(Clone)]
struct Pointing {
    /// The turns, for the time range of the full timeline.
    turns: Signal<Vec<p::TurnGraph>>,
    /// The frames, for the time range of the full timeline.
    frames: Signal<Vec<p::FrameInfo>>,
    /// The range shown.
    zoom: Signal<Zoom>,
    /// The primary button.
    drag: Signal<Drag>,
    /// The width of the plot as last painted.
    width: Rc<Cell<f64>>,
}

impl Pointing {
    /// Track the primary button: a drag across the plot zooms to its range, while a click
    /// is left to the turn under it; a double click returns to the full range.
    fn handle(&self, event: PointerEvent) -> Handled {
        let x = event.pos().x;
        match event.phase() {
            PointerPhase::Down(press) if press.button == MouseButton::Primary => {
                if press.clicks >= 2 {
                    self.zoom.set(Zoom::Full);
                }
                self.drag.set(Drag::Held { from: x, to: x });
                event.capture();
                Handled::No
            }
            PointerPhase::Move => {
                if let Drag::Held { from, .. } = self.drag.get_untracked() {
                    self.drag.set(Drag::Held { from, to: x });
                }
                Handled::No
            }
            PointerPhase::Up(press) if press.button == MouseButton::Primary => {
                let held = self.drag.get_untracked();
                self.drag.set(Drag::Idle);
                event.release_capture();
                let Drag::Held { from, .. } = held else {
                    return Handled::No;
                };
                if (x - from).abs() <= CLICK_SLOP {
                    return Handled::No;
                }
                self.zoom_to(from, x);
                Handled::Yes
            }
            PointerPhase::Cancel => {
                self.drag.set(Drag::Idle);
                Handled::No
            }
            _ => Handled::No,
        }
    }

    /// Show the times between two node-local x positions, within the range shown now.
    fn zoom_to(&self, from: f64, to: f64) {
        let width = self.width.get();
        if width <= 0.0 {
            return;
        }
        let (start, span) = self.turns.with(|turns| {
            self.frames
                .with(|frames| view_range(self.zoom.get_untracked(), turns, frames))
        });
        let at = |x: f64| start + (x - LABEL_WIDTH) / width * span;
        let first = at(from.min(to)).max(start);
        let last = at(from.max(to)).min(start + span);
        if last - first < 1.0 {
            return;
        }
        self.zoom.set(Zoom::Range {
            start_ns: first as u64,
            end_ns: last as u64,
        });
    }
}

/// What one paint shows.
struct Scene<'a> {
    /// The turns.
    turns: &'a [p::TurnGraph],
    /// The frames.
    frames: &'a [p::FrameInfo],
    /// The selected turn.
    selected: Option<u64>,
    /// The first time shown.
    start: f64,
    /// The length of the time shown.
    span: f64,
    /// The primary button on the timeline.
    drag: Drag,
}

impl Scene<'_> {
    /// Whether the time from `from` to `to` overlaps the time shown.
    fn shown(&self, from: f64, to: f64) -> bool {
        to >= self.start && from <= self.start + self.span
    }
}

/// The width of the plot beside the lane labels.
fn plot_width(size: Size) -> f64 {
    (size.width - LABEL_WIDTH - PAD).max(1.0)
}

/// Paint the three lanes and the drag band.
fn draw_timeline(draw: &mut Draw, size: Size, scene: &Scene) {
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
    if scene.turns.is_empty() && scene.frames.is_empty() {
        draw_label(draw, Point::new(LABEL_WIDTH, top), "No turns recorded");
        return;
    }
    draw_input(draw, input, scene);
    draw_updates(draw, updates, scene);
    draw_frames(draw, frame_lane, scene);
    if let Drag::Held { from, to } = scene.drag
        && (to - from).abs() > CLICK_SLOP
    {
        let left = from
            .min(to)
            .clamp(LABEL_WIDTH, LABEL_WIDTH + plot_width(size));
        let right = from
            .max(to)
            .clamp(LABEL_WIDTH, LABEL_WIDTH + plot_width(size));
        let bottom = frame_lane.y + frame_lane.height;
        draw.fill_rect(
            Rect::new(left, top, right - left, bottom - top),
            Color::token(ColorToken::Accent).with_alpha(0.20),
        );
    }
}

/// Paint one lane's background and label, and return its plot rectangle.
fn lane(draw: &mut Draw, size: Size, label: &str, y: f64, height: f64) -> Rect {
    draw_label(draw, Point::new(PAD, y + (height - 10.0) / 2.0), label);
    let rect = Rect::new(LABEL_WIDTH, y, plot_width(size), height);
    draw.fill_rect(rect, Color::token(ColorToken::SurfaceAlt));
    rect
}

/// Paint the input marks of every turn in the time shown.
fn draw_input(draw: &mut Draw, plot: Rect, scene: &Scene) {
    for turn in scene.turns {
        let started = turn.started_ns as f64;
        let ended = turn.started_ns.saturating_add(turn.duration_ns) as f64;
        if !scene.shown(started, ended) {
            continue;
        }
        for _ in turn
            .nodes
            .iter()
            .filter(|node| node.kind == p::CauseKind::Input)
        {
            let x = x_at(plot, scene, started) - 2.0;
            let rect = Rect::new(x, plot.y + 3.0, 5.0, plot.height - 6.0);
            draw.fill_rect(rect, Color::token(ColorToken::Accent));
        }
    }
}

/// Paint one bar per turn in the time shown, colored by its first cause.
fn draw_updates(draw: &mut Draw, plot: Rect, scene: &Scene) {
    for turn in scene.turns {
        let started = turn.started_ns as f64;
        let ended = turn.started_ns.saturating_add(turn.duration_ns) as f64;
        if !scene.shown(started, ended) {
            continue;
        }
        let x = x_at(plot, scene, started);
        let end = x_at(plot, scene, ended);
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
        if scene.selected == Some(turn.turn) {
            draw.stroke_rect(rect, 2.0, Color::token(ColorToken::Accent));
        }
        draw.region(RegionId(turn.turn.max(1)), rect);
    }
}

/// Paint the frame bars in the time shown against the frame period.
fn draw_frames(draw: &mut Draw, plot: Rect, scene: &Scene) {
    let period = scene
        .frames
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
    for frame in scene.frames {
        let started = frame.end_ns.saturating_sub(frame.work_ns) as f64;
        if !scene.shown(started, frame.end_ns as f64) {
            continue;
        }
        let height = (frame.work_ns as f64 / period as f64 * PERIOD_HEIGHT)
            .clamp(MIN_BAR, plot.height - 2.0);
        let raw = x_at(plot, scene, frame.end_ns as f64) - FRAME_BAR / 2.0;
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

/// The first time shown and the length of the time shown: the zoom range, or the first and
/// last time of the turns and frames.
fn view_range(zoom: Zoom, turns: &[p::TurnGraph], frames: &[p::FrameInfo]) -> (f64, f64) {
    match zoom {
        Zoom::Full => time_range(turns, frames),
        Zoom::Range { start_ns, end_ns } => (
            start_ns as f64,
            end_ns.saturating_sub(start_ns).max(1) as f64,
        ),
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

/// The x of `ns` on the plot, for the time shown.
fn x_at(plot: Rect, scene: &Scene, ns: f64) -> f64 {
    plot.x + (ns - scene.start) / scene.span * plot.width
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
