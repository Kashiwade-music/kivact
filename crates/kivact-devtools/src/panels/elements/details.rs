//! Role: The detail tabs of the Elements panel.

use kivact::prelude::*;
use kivact::ui::{description, description_list};
use kivact_devtools_protocol as p;

use crate::panels::elements::styles::styles_tab;
use crate::state::Session;
use crate::widgets::box_model::box_model;

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
pub(crate) fn tabs(session: Session, layout: Signal<bool>) -> impl IntoView {
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
fn panel(tab: DetailTab, session: Session, layout: Signal<bool>) -> View {
    match tab {
        DetailTab::Styles => styles_tab(session, session.detail).into_view(),
        DetailTab::Computed => computed(session.detail),
        DetailTab::Layout => layout_tab(session, layout),
        DetailTab::Semantics => semantics(session.detail),
        DetailTab::Why => why(session.detail),
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
fn layout_tab(session: Session, layout: Signal<bool>) -> View {
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

/// The values the effects of the selected node read.
fn why(detail: Resource<Option<p::NodeDetail>>) -> View {
    dynamic(
        move || detail.latest().flatten(),
        |detail| match detail {
            None => empty("No node selected"),
            Some(detail) if detail.why.is_empty() => empty("No dependencies"),
            Some(detail) => page(description_list(why_rows(detail.why))),
        },
    )
    .into_view()
}

fn why_rows(props: Vec<p::WhyProp>) -> Vec<kivact::ui::Description> {
    props
        .into_iter()
        .map(|prop| {
            let term = format!(
                "{} · {}:{}",
                prop.prop, prop.effect_at.file, prop.effect_at.line
            );
            description(term, lines(read_lines(&prop.reads)))
        })
        .collect()
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

/// The indented lines of a `why` dependency list.
fn read_lines(reads: &[p::WhyDep]) -> Vec<String> {
    let mut lines = Vec::new();
    collect_reads(reads, 0, &mut lines);
    if lines.is_empty() {
        lines.push("no reads".to_owned());
    }
    lines
}

fn collect_reads(reads: &[p::WhyDep], depth: usize, out: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    for read in reads {
        let name = match &read.name_hint {
            Some(name) => format!(" · {name}"),
            None => String::new(),
        };
        out.push(format!("{indent}{} {}{name}", read.kind, read.ty));
        collect_reads(&read.reads, depth + 1, out);
    }
}

fn lines(lines: Vec<String>) -> View {
    column(lines.into_iter().map(text).collect::<Vec<_>>())
        .style(style().gap(2))
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
