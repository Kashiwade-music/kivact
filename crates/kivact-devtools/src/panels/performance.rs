//! Role: The Performance panel.

use kivact::draw::{Draw, Stroke, TextFormat};
use kivact::geometry::Rect;
use kivact::prelude::*;
use kivact_devtools_protocol as p;
use kivact_ui_basic as basic;

use crate::state::{Session, push_capped};
use crate::widgets::flame;

/// The most frames the bar chart draws.
const MAX_BARS: usize = 60;
/// The most counter samples the line chart keeps.
const MAX_SAMPLES: usize = 60;
/// The most snapshots the panel keeps.
const MAX_SNAPSHOTS: usize = 10;

/// One row of the snapshot diff table: kind, change, id, name and source.
type DiffRow = (String, String, String, String, String);

/// The frame timings, the flame chart, the counters and the snapshot diff.
pub(crate) fn view(session: Session) -> impl IntoView {
    let window = signal(session.window());
    let counters = signal(Vec::<(u64, p::Counters)>::new());
    let snapshots = signal(Vec::<p::SnapshotInfo>::new());
    let diff = resource(
        move || {
            snapshots.with(|snapshots| {
                let len = snapshots.len();
                (len >= 2).then(|| (snapshots[len - 2].id, snapshots[len - 1].id))
            })
        },
        move |pair: Option<(u64, u64)>| async move {
            match pair {
                None => Ok(None),
                Some((before, after)) => {
                    let source = session.source();
                    source
                        .call::<p::DiffSnapshots>(p::DiffSnapshotsParams { before, after })
                        .await
                        .map(Some)
                }
            }
        },
    );
    on_mount(move || refresh_counters(session, counters));
    let frames = memo(move || {
        session.frames.with(|frames| {
            let window = window.get();
            frames
                .iter()
                .filter(|frame| frame.window == window)
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    scroll(
        column((
            controls(session, window, frames),
            frame_bars(frames),
            flame::flame_chart(session.turns, frames),
            counters_chart(session, counters),
            snapshot_section(session, snapshots, diff),
        ))
        .style(style().gap(12).padding(12).width(pct(100))),
    )
    .style(style().grow(1.0).min_height(0.0).width(pct(100)))
}

/// The window select, the renderer, the trace export and the paint flashing toggle.
fn controls(
    session: Session,
    window: Signal<p::WindowId>,
    frames: Value<Vec<p::FrameInfo>>,
) -> impl IntoView {
    row((
        select(
            "Window",
            move || {
                session.windows.with(|windows| {
                    windows
                        .iter()
                        .map(|window| ListItem::new(window.id, window.title.clone()))
                        .collect::<Vec<_>>()
                })
            },
            binding(
                move || Some(window.get()),
                move |selected: Option<p::WindowId>| {
                    if let Some(selected) = selected {
                        window.set(selected);
                    }
                },
            ),
        ),
        text(move || frame_summary(&frames.get())),
        spacer(),
        button("Export as Perfetto").on_press(move || export_trace(session)),
        paint_flashing_toggle(session),
    ))
    .style(style().gap(8).align_items(Align::Center))
}

/// The frame count, the frames over budget and the renderer in use.
fn frame_summary(frames: &[p::FrameInfo]) -> String {
    let Some(last) = frames.last() else {
        return "No frames recorded".to_owned();
    };
    let over = frames.iter().filter(|frame| frame.over_budget).count();
    let renderer = match last.renderer {
        p::Renderer::Cpu => "CPU (fallback)",
        p::Renderer::Gpu => "GPU",
    };
    format!(
        "{} frame{} · {over} over budget · renderer: {renderer}",
        frames.len(),
        if frames.len() == 1 { "" } else { "s" }
    )
}

/// The frame time bars with the budget line and over-budget red.
fn frame_bars(frames: Value<Vec<p::FrameInfo>>) -> Node {
    canvas(move |draw, size| {
        let frames = frames.get();
        paint_frame_bars(draw, size, &frames);
    })
    .name("Frame times")
    .style(
        style()
            .width(pct(100))
            .height(160)
            .background(Color::token(ColorToken::SurfaceAlt)),
    )
}

/// Paint the frame bars, their scale and the budget line.
fn paint_frame_bars(draw: &mut Draw, size: Size, frames: &[p::FrameInfo]) {
    let plot = Rect::new(
        52.0,
        12.0,
        (size.width - 64.0).max(1.0),
        (size.height - 34.0).max(1.0),
    );
    draw.line(
        Point::new(plot.x, plot.bottom()),
        Point::new(plot.right(), plot.bottom()),
        Stroke::new(1.0),
        Color::token(ColorToken::Border),
    );
    if frames.is_empty() {
        draw.text(
            "No frames recorded",
            Point::new(plot.x, plot.y + plot.height / 2.0),
            label_style(12.0, ColorToken::TextMuted),
        );
        return;
    }
    let shown = &frames[frames.len().saturating_sub(MAX_BARS)..];
    let budget_ns = shown.last().map(|frame| frame.frame_period_ns).unwrap_or(0);
    let max_ns = shown
        .iter()
        .map(|frame| frame.work_ns.max(frame.frame_period_ns))
        .max()
        .unwrap_or(1)
        .max(1);
    draw.text(
        &format!("{:.1} ms", ns_to_ms(max_ns)),
        Point::new(4.0, plot.y),
        label_style(10.0, ColorToken::TextMuted),
    );
    draw.text(
        &format!("budget {:.1} ms", ns_to_ms(budget_ns)),
        Point::new(plot.x, plot.y + 2.0),
        label_style(10.0, ColorToken::TextMuted),
    );
    let budget_y = plot.bottom() - fraction(budget_ns, max_ns) * plot.height;
    draw.line(
        Point::new(plot.x, budget_y),
        Point::new(plot.right(), budget_y),
        Stroke {
            dash: vec![4.0, 4.0],
            ..Stroke::new(1.0)
        },
        Color::token(ColorToken::TextMuted),
    );
    let slot = plot.width / shown.len() as f64;
    let bar_width = (slot * 0.7).max(1.0);
    for (index, frame) in shown.iter().enumerate() {
        let height = fraction(frame.work_ns, max_ns) * plot.height;
        let rect = Rect::new(
            plot.x + index as f64 * slot + (slot - bar_width) / 2.0,
            plot.bottom() - height,
            bar_width,
            height,
        );
        let color = if frame.over_budget {
            Color::token(ColorToken::Danger)
        } else {
            Color::token(ColorToken::Accent)
        };
        draw.fill_rect(rect, color);
    }
}

/// The counters history and a button that samples again.
fn counters_chart(session: Session, counters: Signal<Vec<(u64, p::Counters)>>) -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("reactive", "Reactive"),
        basic::ChartSeries::new("owners", "Owners"),
        basic::ChartSeries::new("objects", "Objects"),
        basic::ChartSeries::new("constants", "Constants"),
        basic::ChartSeries::new("nodes", "Nodes"),
        basic::ChartSeries::new("timers", "Timers"),
        basic::ChartSeries::new("frame_loops", "Frame loops"),
        basic::ChartSeries::new("animations", "Animations"),
        basic::ChartSeries::new("turns", "Turns"),
        basic::ChartSeries::new("blocking", "Blocking"),
    ]);
    let rows = memo(move || {
        counters
            .get()
            .iter()
            .map(|(sample, counters)| {
                basic::ChartRow::new(
                    format!("#{sample}"),
                    [
                        counters.live_reactive as f64,
                        counters.live_owners as f64,
                        counters.live_objects as f64,
                        counters.live_constants as f64,
                        counters.tree_nodes as f64,
                        counters.timers as f64,
                        counters.frame_loops as f64,
                        counters.animations as f64,
                        counters.turns as f64,
                        counters.outstanding_blocking as f64,
                    ],
                )
            })
            .collect::<Vec<_>>()
    });
    column((
        row((
            text("Counters"),
            spacer(),
            button("Refresh counters").on_press(move || refresh_counters(session, counters)),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        basic::chart(config, rows)
            .kind(basic::ChartKind::Line)
            .height(180),
    ))
    .style(style().gap(6))
}

/// Read the counters once and keep the sample.
fn refresh_counters(session: Session, counters: Signal<Vec<(u64, p::Counters)>>) {
    session.send::<p::GetCounters>(p::Empty {}, move |sample| {
        counters.update(|samples| {
            let sample_number = samples.last().map(|(number, _)| number + 1).unwrap_or(1);
            push_capped(samples, (sample_number, sample), MAX_SAMPLES);
        });
    });
}

/// The snapshot buttons and the table of the last diff.
fn snapshot_section(
    session: Session,
    snapshots: Signal<Vec<p::SnapshotInfo>>,
    diff: Resource<Option<p::SnapshotDiff>>,
) -> impl IntoView {
    column((
        row((
            text("Snapshots"),
            button("Take snapshot").on_press(move || take_snapshot(session, snapshots)),
            text(move || format!("{} taken", snapshots.get().len())),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        dynamic(
            move || diff.state(),
            move |state| match state {
                AsyncState::Ready(Some(diff)) => diff_table(diff).into_view(),
                _ => text("Take two snapshots to compare them").into_view(),
            },
        ),
    ))
    .style(style().gap(6))
}

/// Take one snapshot; the diff follows from the pair of snapshots.
fn take_snapshot(session: Session, snapshots: Signal<Vec<p::SnapshotInfo>>) {
    session.send::<p::TakeSnapshot>(p::Empty {}, move |info| {
        snapshots.update(|snapshots| push_capped(snapshots, info, MAX_SNAPSHOTS));
    });
}

/// The rows of a snapshot diff: kind, change, id, name and source.
fn diff_rows(diff: &p::SnapshotDiff) -> Vec<DiffRow> {
    let mut rows = Vec::new();
    for owner in &diff.added_owners {
        rows.push(diff_row("Owner", "Added", owner.id, "", &owner.source));
    }
    for owner in &diff.removed_owners {
        rows.push(diff_row("Owner", "Removed", owner.id, "", &owner.source));
    }
    for object in &diff.added_objects {
        rows.push(diff_row(
            "Object",
            "Added",
            object.id,
            &object.name,
            &object.source,
        ));
    }
    for object in &diff.removed_objects {
        rows.push(diff_row(
            "Object",
            "Removed",
            object.id,
            &object.name,
            &object.source,
        ));
    }
    rows
}

/// One row of the snapshot diff table.
fn diff_row(kind: &str, change: &str, id: u64, name: &str, source: &p::SourceLocation) -> DiffRow {
    (
        kind.to_owned(),
        change.to_owned(),
        id.to_string(),
        name.to_owned(),
        format!("{}:{}:{}", source.file, source.line, source.column),
    )
}

/// The table of one snapshot diff.
fn diff_table(diff: p::SnapshotDiff) -> impl IntoView {
    let rows = diff_rows(&diff);
    data_table(
        rows,
        |row: &DiffRow| format!("{}:{}:{}", row.0, row.1, row.2),
        vec![
            DataColumn::new("Kind", |row: &DiffRow| row.0.clone()),
            DataColumn::new("Change", |row: &DiffRow| row.1.clone()),
            DataColumn::new("Id", |row: &DiffRow| row.2.clone()),
            DataColumn::new("Name", |row: &DiffRow| row.3.clone()),
            DataColumn::new("Source", |row: &DiffRow| row.4.clone()),
        ],
    )
}

/// Write the recorded turns and frames and show the path.
fn export_trace(session: Session) {
    session.send::<p::ExportTrace>(p::ExportTraceParams { path: None }, move |files| {
        let paths = if files.paths.is_empty() {
            "no trace file was written".to_owned()
        } else {
            files.paths.join(", ")
        };
        session.report(paths);
    });
}

/// The paint flashing toggle, read from and written to the agent's overlays.
fn paint_flashing_toggle(session: Session) -> impl IntoView {
    let selection = binding(
        move || {
            let on = session
                .overlays
                .latest()
                .is_some_and(|overlays| overlays.paint_flashing);
            if on { vec![0_u8] } else { Vec::new() }
        },
        move |keys: Vec<u8>| {
            let on = keys.contains(&0);
            session.set_overlay(|overlays| overlays.paint_flashing = on);
        },
    );
    toggle_group(
        "Overlays",
        vec![ListItem::new(0_u8, "Paint flashing")],
        selection,
    )
    .disabled(move || !session.overlays_ready())
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

/// A small muted label used by the canvases.
fn label_style(size: f64, token: ColorToken) -> TextFormat {
    TextFormat {
        size,
        color: Color::token(token),
        ..TextFormat::default()
    }
}
