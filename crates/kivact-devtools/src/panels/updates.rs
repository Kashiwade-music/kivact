//! Role: The Updates panel.

use kivact::draw::{Draw, TextFormat};
use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;
use crate::widgets::{graph, timeline};

/// The recorded turns and cause graphs.
pub(crate) fn view(session: Session) -> impl IntoView {
    let selected = signal(None::<u64>);
    let turn = memo(move || {
        let selected = selected.get();
        session
            .turns
            .with(|turns| selected.and_then(|turn| turns.iter().find(|t| t.turn == turn).cloned()))
    });
    let causes = memo(move || {
        turn.with(|turn| {
            turn.as_ref().map(|turn| graph::Causes {
                nodes: turn.nodes.clone(),
                edges: turn.edges.clone(),
            })
        })
    });
    let graph = graph::cause_graph(
        causes,
        move |node| session.highlight(Some(node), false),
        move |source| session.open_source(source),
    );
    column((
        timeline::timeline(session.turns, session.frames, selected),
        text(move || header(turn.get().as_ref())).style(style().font_size(13.0).weight(600)),
        scroll(graph).style(style().grow(1).min_height(0)),
        breakdown(turn),
        row((
            count_box("Changed elements", move || {
                turn.get().map_or(0, |turn| turn.changed_nodes.len() as u64)
            }),
            count_box("Recomputed", move || {
                turn.get().map_or(0, |turn| recomputed(&turn))
            }),
            count_box("Equal writes", move || {
                turn.get().map_or(0, |turn| turn.wasted.equal_writes as u64)
            }),
        ))
        .style(style().gap(8)),
        overlays(session),
    ))
    .style(style().gap(8).padding(8).height(pct(100)))
}

/// The header of the selected turn.
fn header(turn: Option<&p::TurnGraph>) -> String {
    let Some(turn) = turn else {
        return "Select a turn to see its causes".to_owned();
    };
    let ms = turn.duration_ns as f64 / 1_000_000.0;
    match turn.nodes.iter().find(|node| is_cause(node.kind)) {
        Some(cause) => format!("#{} {}   total {ms:.1} ms", turn.turn, cause.label),
        None => format!("#{}   total {ms:.1} ms", turn.turn),
    }
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

/// How many memos re-ran in the turn.
fn recomputed(turn: &p::TurnGraph) -> u64 {
    turn.nodes
        .iter()
        .filter(|node| node.kind == p::CauseKind::MemoRun)
        .map(|node| node.count as u64)
        .sum()
}

/// One framed count of the selected turn.
fn count_box(label: &'static str, value: impl Fn() -> u64 + 'static) -> impl IntoView {
    column((
        text(label).style(style().font_size(10.0).color(ColorToken::TextMuted)),
        text(move || value().to_string()).style(style().font_size(16.0).weight(600)),
    ))
    .style(
        style()
            .gap(2)
            .padding_x(10)
            .padding_y(6)
            .min_width(130)
            .border(1, ColorToken::Border)
            .radius(4),
    )
}

/// The stage and frame time bands of the selected turn.
fn breakdown(turn: Value<Option<p::TurnGraph>>) -> impl IntoView {
    canvas(move |draw, size| draw_breakdown(draw, size, turn.get().as_ref()))
        .style(style().width(pct(100)).height(62.0))
}

/// Paint the stages on one row and the frame phases on another.
fn draw_breakdown(draw: &mut Draw, size: Size, turn: Option<&p::TurnGraph>) {
    let label = || TextFormat {
        size: 10.0,
        color: Color::token(ColorToken::TextMuted),
        ..TextFormat::default()
    };
    draw.text("Stages", Point::new(0.0, 2.0), label());
    draw.text("Frame", Point::new(0.0, 34.0), label());
    let Some(turn) = turn else {
        return;
    };
    let total: u64 = turn.stages.iter().map(|stage| stage.duration_ns).sum();
    let mut x = 0.0;
    for (index, stage) in turn.stages.iter().enumerate() {
        let width = if total == 0 {
            0.0
        } else {
            size.width * stage.duration_ns as f64 / total as f64
        };
        let color = STAGE_COLORS[index % STAGE_COLORS.len()];
        draw.fill_rect(Rect::new(x, 16.0, width, 14.0), Color::token(color));
        x += width;
    }
    if let Some(frame) = turn.frames.first() {
        let phases = [
            (frame.scene_ns, ColorToken::Accent),
            (frame.raster_ns, ColorToken::Warning),
            (frame.present_ns, ColorToken::Success),
        ];
        let total: u64 = phases.iter().map(|(ns, _)| *ns).sum();
        let mut x = 0.0;
        for (ns, color) in phases {
            let width = if total == 0 {
                0.0
            } else {
                size.width * ns as f64 / total as f64
            };
            draw.fill_rect(Rect::new(x, 48.0, width, 12.0), Color::token(color));
            x += width;
        }
    }
}

/// The colors of the turn stages, in order.
const STAGE_COLORS: [ColorToken; 8] = [
    ColorToken::Accent,
    ColorToken::Focus,
    ColorToken::Success,
    ColorToken::Warning,
    ColorToken::Danger,
    ColorToken::Selection,
    ColorToken::Border,
    ColorToken::TextMuted,
];

/// The debugging overlays the Updates panel offers.
fn overlays(session: Session) -> impl IntoView {
    row((
        overlay_toggle(
            session,
            "Paint flashing",
            |settings| settings.paint_flashing,
            |settings, on| settings.paint_flashing = on,
        ),
        overlay_toggle(
            session,
            "Selected element only",
            |settings| settings.selected_only,
            |settings, on| settings.selected_only = on,
        ),
        overlay_toggle(
            session,
            "Layout lint",
            |settings| settings.layout_lint,
            |settings, on| settings.layout_lint = on,
        ),
        overlay_toggle(
            session,
            "Update heatmap",
            |settings| settings.update_heatmap,
            |settings, on| settings.update_heatmap = on,
        ),
    ))
    .style(style().gap(8).align_items(Align::Center))
}

/// One overlay toggle, shown from the agent's settings and disabled until they load.
fn overlay_toggle(
    session: Session,
    label: &'static str,
    read: fn(&p::OverlaySettings) -> bool,
    write: fn(&mut p::OverlaySettings, bool),
) -> impl IntoView {
    let pressed = binding(
        move || {
            session
                .overlays
                .latest()
                .is_some_and(|settings| read(&settings))
        },
        move |on: bool| session.set_overlay(move |settings| write(settings, on)),
    );
    kivact::ui::toggle_button(label, pressed).disabled(move || !session.overlays_ready())
}
