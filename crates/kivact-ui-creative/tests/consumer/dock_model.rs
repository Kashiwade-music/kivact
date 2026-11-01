//! Role: The dock's layout model: normalization and every operation's rules.

use kivact_ui_creative as ui;
use ui::{DockAxis, DockLayout, DockNode, DockTab};

pub(super) type Layout = DockLayout<&'static str>;

/// A motion-graphics workspace: Project/Text and Viewer above Timeline/Render
/// Manager, with the Inspector on the right.
pub(super) fn editor_layout() -> Layout {
    DockLayout::new(DockNode::split(
        DockAxis::Horizontal,
        [
            (
                0.75,
                DockNode::split(
                    DockAxis::Vertical,
                    [
                        (
                            0.6,
                            DockNode::split(
                                DockAxis::Horizontal,
                                [
                                    (
                                        0.3,
                                        DockNode::tabs(vec![
                                            DockTab::new("project", "Project 1"),
                                            DockTab::new("text", "Text"),
                                        ]),
                                    ),
                                    (
                                        0.7,
                                        DockNode::tabs(vec![
                                            DockTab::new("viewer", "Viewer 1").icon("eye"),
                                        ]),
                                    ),
                                ],
                            ),
                        ),
                        (
                            0.4,
                            DockNode::tabs(vec![
                                DockTab::new("timeline", "Timeline 1"),
                                DockTab::new("render", "Render Manager"),
                            ]),
                        ),
                    ],
                ),
            ),
            (
                0.25,
                DockNode::tabs(vec![
                    DockTab::new("inspector", "Inspector 1").closable(false),
                ]),
            ),
        ],
    ))
}

pub(super) fn container_of(layout: &Layout, id: &'static str) -> Vec<&'static str> {
    layout
        .container(&id)
        .map(|tabs| tabs.tabs.iter().map(|tab| tab.id).collect())
        .unwrap_or_default()
}

pub(super) fn weights(node: &DockNode<&'static str>) -> Vec<f64> {
    match node {
        DockNode::Split { children, .. } => children.iter().map(|child| child.weight).collect(),
        DockNode::Tabs(_) => Vec::new(),
        // Variants added later fall back to no weights.
        _ => Vec::new(),
    }
}
