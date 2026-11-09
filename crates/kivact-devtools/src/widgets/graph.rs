//! Role: The cause graph of one recorded turn.

use std::rc::Rc;

use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact::style::Position;
use kivact::view::{MouseButton, PointerPhase};
use kivact_devtools_protocol as p;

/// The width of a card.
const CARD_WIDTH: f64 = 150.0;
/// The height of a card.
const CARD_HEIGHT: f64 = 34.0;
/// The gap between two columns.
const COLUMN_GAP: f64 = 34.0;
/// The gap between two rows.
const ROW_GAP: f64 = 8.0;
/// The padding around the graph.
const PAD: f64 = 8.0;
/// The number of columns: cause, state change, recompute, changed elements, band.
const COLUMNS: usize = 5;
/// The width of the whole graph.
const GRAPH_WIDTH: f64 =
    PAD * 2.0 + COLUMNS as f64 * CARD_WIDTH + (COLUMNS - 1) as f64 * COLUMN_GAP;

/// A graph of causes whose cards select a node or open a source location.
pub(crate) fn cause_graph(
    graph: Value<Option<p::TurnGraph>>,
    on_select: impl Fn(p::CauseNode) + 'static,
    on_open: impl Fn(p::SourceLocation) + 'static,
) -> impl IntoView {
    let on_select: Rc<dyn Fn(p::CauseNode)> = Rc::new(on_select);
    let on_open: Rc<dyn Fn(p::SourceLocation)> = Rc::new(on_open);
    dynamic(
        move || graph.get(),
        move |turn| match turn {
            Some(turn) => graph_body(turn, on_select.clone(), on_open.clone()).into_view(),
            None => empty("Select a turn to see its causes").into_view(),
        },
    )
}

/// The cards and the lines of one turn.
fn graph_body(
    turn: p::TurnGraph,
    on_select: Rc<dyn Fn(p::CauseNode)>,
    on_open: Rc<dyn Fn(p::SourceLocation)>,
) -> impl IntoView {
    if turn.nodes.is_empty() {
        return empty("This turn has no causes").into_view();
    }
    let (cards, height) = place(&turn.nodes);
    let lines = lines(&cards, &turn.edges);
    let band = band_rect(height);
    let canvas = canvas(move |draw, _size| {
        draw.fill_rect(band, Color::token(ColorToken::Selection).with_alpha(0.10));
        for (from, to) in &lines {
            draw.line(
                *from,
                *to,
                kivact::draw::Stroke::new(1.0),
                Color::token(ColorToken::Border),
            );
        }
    })
    .hit_testable(false)
    .style(
        style()
            .position(Position::Absolute)
            .left(0)
            .top(0)
            .width(pct(100))
            .height(pct(100)),
    );
    let mut root = node()
        .style(
            style()
                .position(Position::Relative)
                .width(GRAPH_WIDTH)
                .height(height),
        )
        .child(canvas);
    for card in cards {
        root = root.child(card_node(card, on_select.clone(), on_open.clone()));
    }
    root.into_view()
}

/// One card placed in the graph.
struct Card {
    /// The cause the card stands for.
    cause: p::CauseNode,
    /// The left edge.
    x: f64,
    /// The top edge.
    y: f64,
}

/// Assign each node to its column and row.
fn place(nodes: &[p::CauseNode]) -> (Vec<Card>, f64) {
    let mut rows = [0usize; COLUMNS];
    let mut cards = Vec::with_capacity(nodes.len());
    for cause in nodes {
        let column = column_of(cause.kind);
        let row = rows[column];
        rows[column] += 1;
        cards.push(Card {
            cause: cause.clone(),
            x: PAD + column as f64 * (CARD_WIDTH + COLUMN_GAP),
            y: PAD + row as f64 * (CARD_HEIGHT + ROW_GAP),
        });
    }
    let rows = rows.iter().copied().max().unwrap_or(0);
    let height = PAD * 2.0 + rows as f64 * (CARD_HEIGHT + ROW_GAP);
    (cards, height.max(CARD_HEIGHT + PAD * 2.0))
}

/// The edges between the placed cards, from the source's right to the target's left.
fn lines(cards: &[Card], edges: &[p::CauseEdge]) -> Vec<(Point, Point)> {
    let center = |card: &Card, right: bool| {
        Point::new(
            if right { card.x + CARD_WIDTH } else { card.x },
            card.y + CARD_HEIGHT / 2.0,
        )
    };
    edges
        .iter()
        .filter_map(|edge| {
            let source = cards.iter().find(|card| card.cause.id == edge.source)?;
            let target = cards.iter().find(|card| card.cause.id == edge.target)?;
            Some((center(source, true), center(target, false)))
        })
        .collect()
}

/// The faint band behind the layout, paint and frame cards.
fn band_rect(height: f64) -> Rect {
    let x = PAD + (COLUMNS - 1) as f64 * (CARD_WIDTH + COLUMN_GAP) - 6.0;
    Rect::new(x, PAD - 4.0, CARD_WIDTH + 12.0, height - PAD * 2.0 + 8.0)
}

/// One absolute card; a click selects its node and a double click opens its source.
fn card_node(
    card: Card,
    on_select: Rc<dyn Fn(p::CauseNode)>,
    on_open: Rc<dyn Fn(p::SourceLocation)>,
) -> Node {
    let (x, y) = (card.x, card.y);
    let label = card_label(&card.cause);
    let color = column_color(column_of(card.cause.kind));
    let cause = card.cause;
    let select_cause = cause.clone();
    let open_source = cause.source.clone();
    node()
        .role(Role::Button)
        .name(label.clone())
        .style(move || {
            let base = style()
                .position(Position::Absolute)
                .left(x)
                .top(y)
                .width(CARD_WIDTH)
                .height(CARD_HEIGHT)
                .padding_x(6)
                .padding_y(3)
                .align_items(Align::Center)
                .background(ColorToken::SurfaceAlt)
                .border(1, color)
                .radius(4);
            if cause.wasted {
                base.opacity(0.6)
            } else {
                base
            }
        })
        .on_pointer(move |event| {
            let PointerPhase::Up(press) = event.phase() else {
                return Handled::No;
            };
            if press.button != MouseButton::Primary {
                return Handled::No;
            }
            if press.clicks >= 2 {
                if let Some(source) = &open_source {
                    on_open(source.clone());
                }
            } else {
                on_select(select_cause.clone());
            }
            Handled::Yes
        })
        .child(text(label).style(style().font_size(11.0).single_line()))
}

/// The text of a card, with its group size.
fn card_label(cause: &p::CauseNode) -> String {
    let name = if cause.label.is_empty() {
        kind_label(cause.kind).to_owned()
    } else {
        cause.label.clone()
    };
    if cause.count > 1 {
        format!("{name} ×{}", cause.count)
    } else {
        name
    }
}

/// The column of a cause kind.
fn column_of(kind: p::CauseKind) -> usize {
    match kind {
        p::CauseKind::Input
        | p::CauseKind::Timer
        | p::CauseKind::Task
        | p::CauseKind::Mount
        | p::CauseKind::Devtools => 0,
        p::CauseKind::SignalWrite => 1,
        p::CauseKind::MemoRun | p::CauseKind::EffectRun => 2,
        p::CauseKind::PropChange => 3,
        p::CauseKind::Layout | p::CauseKind::Paint | p::CauseKind::Frame => 4,
    }
}

/// The border color of a column.
fn column_color(column: usize) -> ColorToken {
    match column {
        0 => ColorToken::Accent,
        1 => ColorToken::Warning,
        2 => ColorToken::Focus,
        3 => ColorToken::Success,
        _ => ColorToken::Border,
    }
}

/// A short name for a kind whose label is empty.
fn kind_label(kind: p::CauseKind) -> &'static str {
    match kind {
        p::CauseKind::Input => "input",
        p::CauseKind::Timer => "timer",
        p::CauseKind::Task => "task",
        p::CauseKind::Mount => "mount",
        p::CauseKind::Devtools => "devtools",
        p::CauseKind::SignalWrite => "write",
        p::CauseKind::MemoRun => "memo",
        p::CauseKind::EffectRun => "effect",
        p::CauseKind::PropChange => "prop",
        p::CauseKind::Layout => "layout",
        p::CauseKind::Paint => "paint",
        p::CauseKind::Frame => "frame",
    }
}

/// A quiet line for a graph without cards.
fn empty(label: &'static str) -> impl IntoView {
    text(label).style(
        style()
            .padding(8)
            .font_size(11.0)
            .color(ColorToken::TextMuted),
    )
}
