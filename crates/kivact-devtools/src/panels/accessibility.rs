//! Role: The Accessibility panel.

use std::collections::HashMap;

use kivact::interaction::{ListOptions, TableFocus, TreeNode};
use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;
use crate::widgets::virtual_tree::virtual_tree;

/// The accessibility tree, its audits, the Tab order toggle and focus history.
pub(crate) fn view(session: Session) -> impl IntoView {
    let tree = signal(None::<p::A11yTree>);
    let audit = signal(None::<p::AuditReport>);
    let focus = signal(None::<p::FocusInfo>);
    let tab_order = signal(None::<p::TabOrder>);
    let expanded = signal(Vec::<p::NodeId>::new());
    let tree_selected = signal(None::<p::NodeId>);
    let audit_selected = signal(Vec::<usize>::new());

    let window = memo(move || session.window());
    effect(move || {
        let window = window.get();
        load(session, window, tree, audit, focus, tab_order, expanded);
    });
    effect_on(audit_selected, move |keys| {
        let Some(index) = keys.first().copied() else {
            return;
        };
        let node = audit.with(|report| {
            report
                .as_ref()
                .and_then(|report| report.findings.get(index))
                .map(|finding| finding.node)
        });
        if let Some(node) = node {
            session.highlight(Some(node), false);
        }
    });

    scroll(
        column((
            row((
                spacer(),
                button("Refresh").on_press(move || {
                    let window = session.window();
                    load(session, window, tree, audit, focus, tab_order, expanded);
                }),
            ))
            .style(style().align_items(Align::Center)),
            tree_section(session, tree, tab_order, expanded, tree_selected),
            audit_section(audit, audit_selected),
            focus_section(focus),
        ))
        .style(style().gap(8)),
    )
    .style(style().grow(1.0).min_height(0.0))
}

/// The virtualized accessibility tree and the Tab order toggle.
fn tree_section(
    session: Session,
    tree: Signal<Option<p::A11yTree>>,
    tab_order: Signal<Option<p::TabOrder>>,
    expanded: Signal<Vec<p::NodeId>>,
    selected: Signal<Option<p::NodeId>>,
) -> impl IntoView {
    let labels = memo(move || tree.with(|tree| tree.as_ref().map(label_map).unwrap_or_default()));
    let nodes = memo(move || {
        tree.with(|tree| match tree {
            Some(tree) => labels.with(|labels| {
                tree.nodes
                    .iter()
                    .map(|node| {
                        let label = labels.get(&node.node).cloned().unwrap_or_default();
                        TreeNode::new(node.node, node.parent, label)
                    })
                    .collect()
            }),
            None => Vec::new(),
        })
    });
    let row_labels = labels;
    let tree_view = virtual_tree(
        "Accessibility tree",
        nodes,
        expanded,
        binding(move || selected.get(), move |node| selected.set(node)),
        move |node| {
            let node = *node;
            text(move || {
                row_labels.with(|labels| {
                    labels
                        .get(&node)
                        .cloned()
                        .unwrap_or_else(|| node.to_string())
                })
            })
            .into_view()
        },
    );
    let tab_binding = binding(
        move || {
            session
                .overlays
                .latest()
                .map(|overlays| overlays.tab_order)
                .unwrap_or(false)
        },
        move |on| session.set_overlay(move |overlays| overlays.tab_order = on),
    );
    column((
        text("Accessibility tree"),
        switch("Tab order numbers", tab_binding).disabled(move || !session.overlays_ready()),
        show_some(tab_order, |order| {
            text(move || format!("Tab order: {} nodes", order.get().nodes.len()))
        })
        .fallback(|| text("Tab order is loading...")),
        node()
            .style(style().height(220.0).min_height(0.0))
            .child(tree_view),
    ))
}

/// The audit findings table.
fn audit_section(
    audit: Signal<Option<p::AuditReport>>,
    selected: Signal<Vec<usize>>,
) -> impl IntoView {
    column((
        text("Audits"),
        show_some(audit, move |report| audit_table(report, selected))
            .fallback(|| text("The audit is loading...")),
    ))
}

/// The focus of the window and its history.
fn focus_section(focus: Signal<Option<p::FocusInfo>>) -> impl IntoView {
    column((
        text("Focus history"),
        show_some(focus, focus_view).fallback(|| text("The focus history is loading...")),
    ))
}

/// Load the accessibility tree, audit, focus and Tab order of the window.
fn load(
    session: Session,
    window: p::WindowId,
    tree: Signal<Option<p::A11yTree>>,
    audit: Signal<Option<p::AuditReport>>,
    focus: Signal<Option<p::FocusInfo>>,
    tab_order: Signal<Option<p::TabOrder>>,
    expanded: Signal<Vec<p::NodeId>>,
) {
    let params = p::WindowParams { window };
    session.send::<p::GetA11yTree>(params.clone(), move |result| {
        expanded.set(
            result
                .nodes
                .iter()
                .filter(|node| !node.children.is_empty())
                .map(|node| node.node)
                .collect(),
        );
        tree.set(Some(result));
    });
    session.send::<p::AuditA11y>(params.clone(), move |result| {
        audit.set(Some(result));
    });
    session.send::<p::GetFocus>(params.clone(), move |result| {
        focus.set(Some(result));
    });
    session.send::<p::GetTabOrder>(params, move |result| {
        tab_order.set(Some(result));
    });
}

/// The table of the audit findings; choosing a row highlights its node.
fn audit_table(report: Value<p::AuditReport>, selected: Signal<Vec<usize>>) -> impl IntoView {
    let rows = memo(move || {
        report
            .get()
            .findings
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>()
    });
    data_table(
        rows,
        |(index, _): &(usize, p::AuditFinding)| *index,
        vec![
            DataColumn::new("Rule", |row: &(usize, p::AuditFinding)| {
                rule_text(row.1.rule)
            }),
            DataColumn::new("Node", |row: &(usize, p::AuditFinding)| {
                row.1.node.to_string()
            }),
            DataColumn::new("Message", |row: &(usize, p::AuditFinding)| {
                row.1.message.clone()
            }),
            DataColumn::new("Value", |row: &(usize, p::AuditFinding)| {
                number(row.1.value)
            }),
        ],
    )
    .row_label(|row| row.1.message.clone())
    .selection(selected)
    .focus(TableFocus::Rows(ListOptions::default()))
    .name("Accessibility audits")
}

/// The focused node and the moves that led to it.
fn focus_view(info: Value<p::FocusInfo>) -> impl IntoView {
    let rows = memo(move || {
        info.get()
            .history
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>()
    });
    column((
        text(move || match info.get().node {
            Some(node) => format!("Focused node: {node}"),
            None => "Focused node: none".to_owned(),
        }),
        text(move || format!("Window focused: {}", info.get().visible)),
        data_table(
            rows,
            |(index, _): &(usize, p::FocusMove)| *index,
            vec![
                DataColumn::new("At", |row: &(usize, p::FocusMove)| millis(row.1.at_ns)),
                DataColumn::new("Before", |row: &(usize, p::FocusMove)| {
                    node_text(row.1.before)
                }),
                DataColumn::new("After", |row: &(usize, p::FocusMove)| {
                    node_text(row.1.after)
                }),
                DataColumn::new("Reason", |row: &(usize, p::FocusMove)| {
                    reason_text(row.1.reason)
                }),
            ],
        )
        .row_label(|row| format!("move {}", row.1.at_ns))
        .name("Focus history"),
    ))
}

/// The label of every node of the tree, built once per tree so that the rows look them up.
fn label_map(tree: &p::A11yTree) -> HashMap<p::NodeId, String> {
    tree.nodes
        .iter()
        .map(|node| (node.node, node_label(node)))
        .collect()
}

/// The label of one accessibility node: its role and its name.
fn node_label(node: &p::A11yNode) -> String {
    let role = node
        .semantics
        .role
        .map(role_text)
        .unwrap_or_else(|| "generic".to_owned());
    match node.semantics.name.as_deref() {
        Some(name) if !name.is_empty() => format!("{role} {name}"),
        _ => role,
    }
}

/// The wire name of a role.
fn role_text(role: p::Role) -> String {
    serde_json::to_value(role)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "generic".to_owned())
}

/// The wire name of an audit rule.
fn rule_text(rule: p::AuditRule) -> String {
    serde_json::to_value(rule)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "rule".to_owned())
}

/// The wire name of a focus reason.
fn reason_text(reason: p::FocusReason) -> String {
    serde_json::to_value(reason)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "programmatic".to_owned())
}

/// A node id, or a dash when there is none.
fn node_text(node: Option<p::NodeId>) -> String {
    node.map(|node| node.to_string())
        .unwrap_or_else(|| "-".to_owned())
}

/// A measured value, or an empty cell when there is none.
fn number(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.2}")).unwrap_or_default()
}

/// A time of the app's clock in milliseconds.
fn millis(ns: u64) -> String {
    format!("{:.3} ms", ns as f64 / 1_000_000.0)
}
