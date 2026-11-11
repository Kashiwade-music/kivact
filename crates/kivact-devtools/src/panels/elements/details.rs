//! Role: The detail tabs of the Elements panel.

use kivact::prelude::*;
use kivact::ui::{description, description_list};
use kivact_devtools_protocol as p;

use crate::panels::elements::styles::styles_tab;
use crate::state::Session;
use crate::widgets::box_model::box_model;
use crate::widgets::graph;

/// One tab of the selected node's details.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DetailTab {
    /// The style layers and overrides.
    Styles,
    /// The final style values.
    Computed,
    /// The box model and the layout overlay.
    Layout,
    /// The accessible semantics.
    Semantics,
    /// The values the effects read.
    Why,
    /// The event handlers.
    Handlers,
}

impl DetailTab {
    /// Every tab, in tab order.
    const ALL: [DetailTab; 6] = [
        DetailTab::Styles,
        DetailTab::Computed,
        DetailTab::Layout,
        DetailTab::Semantics,
        DetailTab::Why,
        DetailTab::Handlers,
    ];

    /// The tab title.
    fn title(self) -> &'static str {
        match self {
            DetailTab::Styles => "Styles",
            DetailTab::Computed => "Computed",
            DetailTab::Layout => "Layout",
            DetailTab::Semantics => "Semantics",
            DetailTab::Why => "Why",
            DetailTab::Handlers => "Handlers",
        }
    }
}

/// The tabs of the selected node's details.
pub(crate) fn tabs(session: Session, layout: Binding<bool>) -> impl IntoView {
    let active = signal(DetailTab::Styles);
    let items = DetailTab::ALL
        .iter()
        .map(|tab| ListItem::new(*tab, tab.title()))
        .collect::<Vec<_>>();
    kivact::ui::tabs("Element details", items, active)
        .content(move |tab| panel(tab, session, layout))
        .style(style().size(pct(100), pct(100)))
}

/// The content of one detail tab.
fn panel(tab: DetailTab, session: Session, layout: Binding<bool>) -> View {
    match tab {
        DetailTab::Styles => styles_tab(session, session.detail).into_view(),
        DetailTab::Computed => computed(session.detail),
        DetailTab::Layout => layout_tab(session, layout),
        DetailTab::Semantics => semantics(session.detail),
        DetailTab::Why => why(session, session.detail),
        DetailTab::Handlers => handlers(session.detail),
    }
}

/// The computed style values of the selected node.
fn computed(detail: Resource<Option<p::NodeDetail>>) -> View {
    dynamic(
        move || detail.latest().flatten(),
        |detail| match detail {
            None => empty("No node selected"),
            Some(detail) if detail.computed.is_empty() => empty("No computed values"),
            Some(detail) => page(description_list(
                detail
                    .computed
                    .into_iter()
                    .map(|property| description(property.name, text(property.value)))
                    .collect::<Vec<_>>(),
            )),
        },
    )
    .into_view()
}

/// The box model and the layout overlay toggle.
fn layout_tab(session: Session, layout: Binding<bool>) -> View {
    let bounds = memo(move || {
        session
            .detail
            .latest()
            .flatten()
            .map(|detail| detail.bounds)
    });
    column((checkbox("Show layout overlay", layout), box_model(bounds)))
        .style(style().size(pct(100), pct(100)).gap(8).padding(8))
        .into_view()
}

/// The accessible semantics of the selected node.
fn semantics(detail: Resource<Option<p::NodeDetail>>) -> View {
    dynamic(
        move || detail.latest().flatten(),
        |detail| match detail {
            None => empty("No node selected"),
            Some(detail) => page(description_list(semantics_rows(detail.semantics))),
        },
    )
    .into_view()
}

fn semantics_rows(info: p::SemanticsInfo) -> Vec<kivact::ui::Description> {
    let states = join_labels(&info.states);
    let actions = join_actions(&info.actions);
    let labelled_by = join_ids(&info.labelled_by);
    let described_by = join_ids(&info.described_by);
    let controls = join_ids(&info.controls);
    vec![
        description("Role", text(option_role(info.role))),
        description("Name", text(option_text(info.name))),
        description("Description", text(option_text(info.description))),
        description("Value", text(option_text(info.value))),
        description("States", text(states)),
        description("Actions", text(actions)),
        description("Labelled by", text(labelled_by)),
        description("Described by", text(described_by)),
        description("Controls", text(controls)),
        description("Focusable", text(if info.focusable { "yes" } else { "no" })),
    ]
}

/// The values the effects of the selected node read, as a dependency graph.
fn why(session: Session, detail: Resource<Option<p::NodeDetail>>) -> View {
    dynamic(
        move || detail.latest().flatten(),
        move |detail| match detail {
            None => empty("No node selected"),
            Some(detail) if detail.why.is_empty() => empty("No dependencies"),
            Some(detail) => page(graph::cause_graph(
                Some(why_causes(detail.why)),
                |_| (),
                move |source| session.open_source(source),
            )),
        },
    )
    .into_view()
}

/// The graph of a `why` list: each prop is a card on the right, and each value it reads is a card
/// on its left, with the deepest values furthest left.
fn why_causes(props: Vec<p::WhyProp>) -> graph::Causes {
    let mut causes = graph::Causes {
        nodes: Vec::new(),
        edges: Vec::new(),
    };
    for prop in props {
        let reader = next_id(&causes);
        causes.nodes.push(p::CauseNode {
            id: reader,
            kind: p::CauseKind::PropChange,
            label: prop.prop,
            ty: None,
            name_hint: None,
            source: Some(prop.effect_at),
            node: None,
            window: None,
            count: 1,
            wasted: false,
            members: Vec::new(),
        });
        for dep in &prop.reads {
            add_dep(&mut causes, dep, reader);
        }
    }
    causes
}

/// Add one dependency and the values it reads, with an edge from each to the card it feeds.
fn add_dep(causes: &mut graph::Causes, dep: &p::WhyDep, reader: u32) {
    let id = next_id(causes);
    let kind = if dep.last_write.is_some() {
        p::CauseKind::SignalWrite
    } else {
        p::CauseKind::MemoRun
    };
    let label = match &dep.name_hint {
        Some(name) => format!("{} {} · {name}", dep.kind, dep.ty),
        None => format!("{} {}", dep.kind, dep.ty),
    };
    causes.nodes.push(p::CauseNode {
        id,
        kind,
        label,
        ty: Some(dep.ty.clone()),
        name_hint: dep.name_hint.clone(),
        source: Some(dep.created_at.clone()),
        node: None,
        window: None,
        count: 1,
        wasted: false,
        members: Vec::new(),
    });
    causes.edges.push(p::CauseEdge {
        source: id,
        target: reader,
    });
    for read in &dep.reads {
        add_dep(causes, read, id);
    }
}

/// The id of the next card of `causes`.
fn next_id(causes: &graph::Causes) -> u32 {
    u32::try_from(causes.nodes.len()).expect("invariant: a graph has fewer than 2^32 cards")
}

/// The event handlers of the selected node.
fn handlers(detail: Resource<Option<p::NodeDetail>>) -> View {
    dynamic(
        move || detail.latest().flatten(),
        |detail| match detail {
            None => empty("No node selected"),
            Some(detail) if detail.handlers.is_empty() => empty("No handlers"),
            Some(detail) => page(description_list(
                detail
                    .handlers
                    .into_iter()
                    .map(|handler| description("Event", text(handler.event)))
                    .collect::<Vec<_>>(),
            )),
        },
    )
    .into_view()
}

/// A scrollable page with padding.
fn page(content: impl IntoView) -> View {
    scroll(content)
        .style(style().size(pct(100), pct(100)).padding(8))
        .into_view()
}

/// A muted message for a tab without data.
fn empty(message: &'static str) -> View {
    text(message)
        .style(style().color(ColorToken::TextMuted).padding(8))
        .into_view()
}

fn option_text(value: Option<String>) -> String {
    value.unwrap_or_else(|| "none".to_owned())
}

fn option_role(role: Option<p::Role>) -> String {
    role.map_or_else(|| "none".to_owned(), |role| format!("{role:?}"))
}

fn join_labels(labels: &[String]) -> String {
    if labels.is_empty() {
        return "none".to_owned();
    }
    labels.join(", ")
}

fn join_actions(actions: &[p::A11yAction]) -> String {
    if actions.is_empty() {
        return "none".to_owned();
    }
    actions
        .iter()
        .map(|action| format!("{action:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn join_ids(ids: &[p::NodeId]) -> String {
    if ids.is_empty() {
        return "none".to_owned();
    }
    ids.iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
