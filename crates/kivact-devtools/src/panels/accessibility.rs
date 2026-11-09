//! Role: The Accessibility panel.

use std::collections::HashMap;
use std::rc::Rc;

use kivact::interaction::{ListOptions, TableFocus, TreeNode};
use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;
use crate::widgets::virtual_tree::virtual_tree;

/// The accessibility data of one window, loaded together.
#[derive(Clone, Debug, PartialEq)]
struct A11yData {
    tree: p::A11yTree,
    audit: p::AuditReport,
    focus: p::FocusInfo,
    tab_order: p::TabOrder,
}

/// The accessibility tree, its audits, the Tab order toggle and focus history.
pub(crate) fn view(session: Session) -> impl IntoView {
    let expanded = signal(Vec::<p::NodeId>::new());
    let tree_selected = signal(None::<p::NodeId>);
    let audit_selected = signal(Vec::<usize>::new());
    let data = resource(
        move || session.window(),
        move |window| load(session, window, expanded),
    );
    effect_on(audit_selected, move |keys| {
        let Some(index) = keys.first().copied() else {
            return;
        };
        let node = data
            .latest()
            .and_then(|data| data.audit.findings.get(index).map(|finding| finding.node));
        if let Some(node) = node {
            session.highlight(Some(node), false);
        }
    });

    scroll(
        column((
            row((spacer(), button("Refresh").on_press(move || data.refetch())))
                .style(style().align_items(Align::Center)),
            dynamic(
                move || data.state(),
                move |state| match state {
                    AsyncState::Idle | AsyncState::Pending => {
                        text("The accessibility data is loading...").into_view()
                    }
                    AsyncState::Failed(error) => text(error.message().to_owned()).into_view(),
                    AsyncState::Ready(data) => {
                        body(session, data, expanded, tree_selected, audit_selected).into_view()
                    }
                },
            ),
        ))
        .style(style().gap(8)),
    )
    .style(style().grow(1.0).min_height(0.0))
}

/// The tree, audit and focus sections of one load.
fn body(
    session: Session,
    data: A11yData,
    expanded: Signal<Vec<p::NodeId>>,
    tree_selected: Signal<Option<p::NodeId>>,
    audit_selected: Signal<Vec<usize>>,
) -> impl IntoView {
    let A11yData {
        tree,
        audit,
        focus,
        tab_order,
    } = data;
    column((
        tree_section(session, tree, tab_order, expanded, tree_selected),
        audit_section(audit, audit_selected),
        focus_section(focus),
    ))
    .style(style().gap(8))
}

/// The virtualized accessibility tree and the Tab order toggle.
fn tree_section(
    session: Session,
    tree: p::A11yTree,
    tab_order: p::TabOrder,
    expanded: Signal<Vec<p::NodeId>>,
    selected: Signal<Option<p::NodeId>>,
) -> impl IntoView {
    let labels = Rc::new(label_map(&tree));
    let nodes = tree
        .nodes
        .iter()
        .map(|node| {
            let label = labels.get(&node.node).cloned().unwrap_or_default();
            TreeNode::new(node.node, node.parent, label)
        })
        .collect::<Vec<_>>();
    let row_labels = labels;
    let tree_view = virtual_tree(
        "Accessibility tree",
        nodes.into_value(),
        expanded,
        binding(move || selected.get(), move |node| selected.set(node)),
        move |node| {
            let node = *node;
            let labels = row_labels.clone();
            text(move || {
                labels
                    .get(&node)
                    .cloned()
                    .unwrap_or_else(|| node.to_string())
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
        text(format!("Tab order: {} nodes", tab_order.nodes.len())),
        node()
            .style(style().height(220.0).min_height(0.0))
            .child(tree_view),
    ))
}

/// The audit findings table.
fn audit_section(audit: p::AuditReport, selected: Signal<Vec<usize>>) -> impl IntoView {
    column((text("Audits"), audit_table(audit, selected)))
}

/// The focus of the window and its history.
fn focus_section(focus: p::FocusInfo) -> impl IntoView {
    column((text("Focus history"), focus_view(focus)))
}

/// Load the accessibility tree, audit, focus and Tab order of the window.
async fn load(
    session: Session,
    window: p::WindowId,
    expanded: Signal<Vec<p::NodeId>>,
) -> Result<A11yData, UiError> {
    let source = session.source();
    let params = p::WindowParams { window };
    let tree = source.call::<p::GetA11yTree>(params.clone()).await?;
    expanded.set(
        tree.nodes
            .iter()
            .filter(|node| !node.children.is_empty())
            .map(|node| node.node)
            .collect(),
    );
    let audit = source.call::<p::AuditA11y>(params.clone()).await?;
    let focus = source.call::<p::GetFocus>(params.clone()).await?;
    let tab_order = source.call::<p::GetTabOrder>(params).await?;
    Ok(A11yData {
        tree,
        audit,
        focus,
        tab_order,
    })
}

/// The table of the audit findings; choosing a row highlights its node.
fn audit_table(report: p::AuditReport, selected: Signal<Vec<usize>>) -> impl IntoView {
    let rows = report.findings.into_iter().enumerate().collect::<Vec<_>>();
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
fn focus_view(info: p::FocusInfo) -> impl IntoView {
    let rows = info.history.into_iter().enumerate().collect::<Vec<_>>();
    column((
        text(match info.node {
            Some(node) => format!("Focused node: {node}"),
            None => "Focused node: none".to_owned(),
        }),
        text(format!("Window focused: {}", info.visible)),
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
