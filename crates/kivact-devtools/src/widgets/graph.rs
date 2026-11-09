//! Role: The cause graph of one recorded turn or one `why` dependency list.

use std::collections::{BTreeSet, HashMap};
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
/// The height of one member row of an expanded card.
const MEMBER_HEIGHT: f64 = 20.0;
/// The space between a card and its expanded member rows.
const MEMBER_GAP: f64 = 4.0;
/// The width of the `×N` button on a grouped card.
const COUNT_WIDTH: f64 = 30.0;
/// The number of columns: cause, state change, recompute, changed elements, band.
const COLUMNS: usize = 5;
/// The width of the whole graph.
const GRAPH_WIDTH: f64 =
    PAD * 2.0 + COLUMNS as f64 * CARD_WIDTH + (COLUMNS - 1) as f64 * COLUMN_GAP;

/// The cards and the edges of one graph.
#[derive(Clone, PartialEq)]
pub(crate) struct Causes {
    /// The cards.
    pub(crate) nodes: Vec<p::CauseNode>,
    /// The edges, from the cause to the card it feeds.
    pub(crate) edges: Vec<p::CauseEdge>,
}

/// A graph of causes. A click on a card highlights its element and a double click opens its
/// source; the `×N` button of a grouped card lists its members, and a click on a member highlights it.
pub(crate) fn cause_graph(
    causes: impl IntoValue<Option<Causes>>,
    on_highlight: impl Fn(p::NodeId) + 'static,
    on_open: impl Fn(p::SourceLocation) + 'static,
) -> impl IntoView {
    let causes = causes.into_value();
    let expanded = signal(BTreeSet::<u32>::new());
    effect_on(causes, move |_| expanded.set(BTreeSet::new()));
    let on_highlight: Rc<dyn Fn(p::NodeId)> = Rc::new(on_highlight);
    let on_open: Rc<dyn Fn(p::SourceLocation)> = Rc::new(on_open);
    dynamic(
        move || (causes.get(), expanded.get()),
        move |(causes, open)| match causes {
            Some(causes) => graph_body(
                causes,
                open,
                expanded,
                on_highlight.clone(),
                on_open.clone(),
            )
            .into_view(),
            None => empty("Select a turn to see its causes").into_view(),
        },
    )
}

/// The cards, the lines and the member rows of one graph.
fn graph_body(
    causes: Causes,
    open: BTreeSet<u32>,
    expanded: Signal<BTreeSet<u32>>,
    on_highlight: Rc<dyn Fn(p::NodeId)>,
    on_open: Rc<dyn Fn(p::SourceLocation)>,
) -> impl IntoView {
    if causes.nodes.is_empty() {
        return empty("This turn has no causes").into_view();
    }
    let labels = member_labels(&causes.nodes);
    let (cards, height) = place(&causes.nodes, &open);
    let lines = lines(&cards, &causes.edges);
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
    for card in &cards {
        root = root.child(card_node(card, on_highlight.clone(), on_open.clone()));
        if card.cause.count > 1 {
            root = root.child(count_button(card, expanded));
        }
        if open.contains(&card.cause.id) {
            for (index, member) in card.cause.members.iter().enumerate() {
                root = root.child(member_row(
                    card,
                    index,
                    *member,
                    &labels,
                    on_highlight.clone(),
                ));
            }
        }
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

/// Assign each card to its column, stacking the cards of a column below each other; an expanded
/// card makes room for its member rows.
fn place(nodes: &[p::CauseNode], expanded: &BTreeSet<u32>) -> (Vec<Card>, f64) {
    let mut next_y = [PAD; COLUMNS];
    let mut cards = Vec::with_capacity(nodes.len());
    for cause in nodes {
        let column = column_of(cause.kind);
        let y = next_y[column];
        let members = if expanded.contains(&cause.id) {
            cause.members.len() as f64 * MEMBER_HEIGHT + MEMBER_GAP
        } else {
            0.0
        };
        next_y[column] += CARD_HEIGHT + members + ROW_GAP;
        cards.push(Card {
            cause: cause.clone(),
            x: PAD + column as f64 * (CARD_WIDTH + COLUMN_GAP),
            y,
        });
    }
    let bottom = next_y.iter().copied().fold(PAD, f64::max);
    let height = (bottom + PAD).max(CARD_HEIGHT + PAD * 2.0);
    (cards, height)
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

/// The label of each element a member stands for: the label of a cause that names it, if any.
fn member_labels(nodes: &[p::CauseNode]) -> HashMap<p::NodeId, String> {
    let mut labels = HashMap::new();
    for cause in nodes {
        if let Some(node) = cause.node
            && !cause.label.is_empty()
        {
            labels.entry(node).or_insert_with(|| cause.label.clone());
        }
    }
    labels
}

/// One absolute card; a click highlights its node and a double click opens its source.
fn card_node(
    card: &Card,
    on_highlight: Rc<dyn Fn(p::NodeId)>,
    on_open: Rc<dyn Fn(p::SourceLocation)>,
) -> Node {
    let (x, y) = (card.x, card.y);
    let label = card_label(&card.cause);
    let color = column_color(column_of(card.cause.kind));
    let cause = card.cause.clone();
    let highlight = cause.node;
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
            } else if let Some(node) = highlight {
                on_highlight(node);
            }
            Handled::Yes
        })
        .child(text(label).style(style().font_size(11.0).single_line()))
}

/// The `×N` button of a grouped card, at its top right; it expands or collapses the members.
fn count_button(card: &Card, expanded: Signal<BTreeSet<u32>>) -> Node {
    let id = card.cause.id;
    let label = format!("×{}", card.cause.count);
    let (x, y) = (card.x + CARD_WIDTH - COUNT_WIDTH - 2.0, card.y + 2.0);
    node()
        .role(Role::Button)
        .name(label.clone())
        .style(
            style()
                .position(Position::Absolute)
                .left(x)
                .top(y)
                .width(COUNT_WIDTH)
                .height(CARD_HEIGHT - 4.0)
                .align_items(Align::Center)
                .background(ColorToken::Surface)
                .border(1, ColorToken::Border)
                .radius(3),
        )
        .on_pointer(move |event| {
            let PointerPhase::Up(press) = event.phase() else {
                return Handled::No;
            };
            if press.button == MouseButton::Primary {
                expanded.update(|ids| {
                    if !ids.remove(&id) {
                        ids.insert(id);
                    }
                });
            }
            Handled::Yes
        })
        .child(text(label).style(style().font_size(10.0).single_line()))
}

/// One member row under an expanded card; a click highlights the member element.
fn member_row(
    card: &Card,
    index: usize,
    member: p::NodeId,
    labels: &HashMap<p::NodeId, String>,
    on_highlight: Rc<dyn Fn(p::NodeId)>,
) -> Node {
    let label = labels
        .get(&member)
        .cloned()
        .unwrap_or_else(|| format!("#{member}"));
    let x = card.x;
    let y = card.y + CARD_HEIGHT + MEMBER_GAP / 2.0 + index as f64 * MEMBER_HEIGHT;
    node()
        .role(Role::Button)
        .name(label.clone())
        .style(
            style()
                .position(Position::Absolute)
                .left(x + 8.0)
                .top(y)
                .width(CARD_WIDTH - 8.0)
                .height(MEMBER_HEIGHT)
                .padding_x(4)
                .align_items(Align::Center)
                .background(ColorToken::Surface)
                .radius(3),
        )
        .on_pointer(move |event| {
            let PointerPhase::Up(press) = event.phase() else {
                return Handled::No;
            };
            if press.button == MouseButton::Primary {
                on_highlight(member);
            }
            Handled::Yes
        })
        .child(text(label).style(style().font_size(10.0).single_line()))
}

/// The text of a card.
fn card_label(cause: &p::CauseNode) -> String {
    if cause.label.is_empty() {
        kind_label(cause.kind).to_owned()
    } else {
        cause.label.clone()
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
