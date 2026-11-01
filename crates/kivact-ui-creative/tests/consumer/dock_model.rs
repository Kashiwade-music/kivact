//! Role: The dock's layout model: normalization and every operation's rules.

use super::*;
use ui::{DockAxis, DockLayout, DockNode, DockPlacement, DockSide, DockTab};

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
    }
}

pub(super) fn tabs_of(ids: &[&'static str]) -> DockNode<&'static str> {
    DockNode::tabs(ids.iter().map(|id| DockTab::new(*id, *id)).collect())
}

#[test]
fn normalize_repairs_duplicates_actives_empties_weights_nesting_and_maximized() {
    let mut layout = DockLayout {
        root: DockNode::split(
            DockAxis::Horizontal,
            [
                (f64::NAN, tabs_of(&["a", "b", "a"])),
                (-1.0, DockNode::tabs(Vec::new())),
                (
                    2.0,
                    DockNode::split(
                        DockAxis::Horizontal,
                        [(1.0, tabs_of(&["c"])), (3.0, tabs_of(&["b", "d"]))],
                    ),
                ),
                (
                    2.0,
                    DockNode::split(DockAxis::Vertical, [(1.0, tabs_of(&["e"]))]),
                ),
            ],
        ),
        maximized: Some("gone"),
    };
    layout.normalize();
    assert_eq!(layout.tab_ids(), ["a", "b", "c", "d", "e"]);
    assert_eq!(layout.maximized, None);
    assert_eq!(
        container_of(&layout, "d"),
        ["d"],
        "the second b is dropped, the container keeps d"
    );
    let DockNode::Split { axis, children } = &layout.root else {
        panic!("root stays a split: {layout:?}");
    };
    assert_eq!(*axis, DockAxis::Horizontal);
    assert_eq!(
        children.len(),
        4,
        "the empty container left, the nested row merged, the single child collapsed"
    );
    assert!(near(
        children.iter().map(|child| child.weight).sum::<f64>(),
        1.0,
        1e-9
    ));
    assert!(
        children.iter().all(|child| child.weight > 0.0),
        "invalid weights became the mean of the valid ones"
    );
    assert!(
        near(children[1].weight / children[2].weight, 1.0 / 3.0, 1e-9),
        "merged children keep their relative sizes"
    );
    assert_eq!(
        layout.container(&"a").unwrap().active,
        Some("a"),
        "the active key is repaired"
    );

    let mut active = DockLayout::new(DockNode::Tabs(
        ui::DockTabs::new(vec![DockTab::new("x", "X"), DockTab::new("y", "Y")]).active("zzz"),
    ));
    assert_eq!(active.container(&"x").unwrap().active, Some("x"));
    active.root = DockNode::split(DockAxis::Vertical, [(1.0, tabs_of(&[]))]);
    active.normalize();
    assert!(active.is_empty());
    assert_eq!(Layout::default(), active);
}

#[test]
fn queries_activate_rename_and_insert() {
    let mut layout = editor_layout();
    assert!(!layout.is_empty());
    assert_eq!(layout.containers().len(), 4);
    assert_eq!(
        layout.tab_ids(),
        [
            "project",
            "text",
            "viewer",
            "timeline",
            "render",
            "inspector"
        ]
    );
    assert_eq!(
        layout.tab(&"viewer").unwrap().icon,
        Some(kivact::style::Icon::named("eye"))
    );
    assert!(layout.tab(&"nope").is_none());
    assert!(layout.activate(&"text"));
    assert!(!layout.activate(&"text"), "already active");
    assert!(!layout.activate(&"nope"));
    assert_eq!(layout.container(&"project").unwrap().active, Some("text"));
    assert!(layout.rename(&"viewer", "Hero"));
    assert!(!layout.rename(&"viewer", "Hero"), "unchanged");
    assert!(!layout.rename(&"viewer", "   "), "blank is rejected");
    assert!(!layout.rename(&"nope", "X"));
    assert_eq!(layout.tab(&"viewer").unwrap().title, "Hero");

    assert!(!layout.insert(
        DockTab::new("text", "Again"),
        DockPlacement::Tabs {
            container: "viewer",
            index: None
        }
    ));
    assert!(!layout.insert(
        DockTab::new("new", "New"),
        DockPlacement::Tabs {
            container: "nope",
            index: None
        }
    ));
    assert!(layout.insert(
        DockTab::new("new", "New"),
        DockPlacement::Tabs {
            container: "viewer",
            index: Some(0)
        }
    ));
    assert_eq!(container_of(&layout, "viewer"), ["new", "viewer"]);
    assert_eq!(layout.container(&"viewer").unwrap().active, Some("new"));

    let mut empty = Layout::default();
    assert!(empty.insert(
        DockTab::new("first", "First"),
        DockPlacement::Edge(DockSide::Left)
    ));
    assert_eq!(empty.tab_ids(), ["first"]);
    assert!(!empty.insert(
        DockTab::new("first", "Twice"),
        DockPlacement::Edge(DockSide::Left)
    ));
}

#[test]
fn move_tab_reorders_joins_splits_and_docks_to_an_edge() {
    let mut layout = editor_layout();
    // Within the strip: the index counts without the moved tab.
    assert!(layout.move_tab(
        &"project",
        DockPlacement::Tabs {
            container: "text",
            index: Some(1)
        }
    ));
    assert_eq!(container_of(&layout, "text"), ["text", "project"]);

    // Into another container's tabs.
    let mut layout = editor_layout();
    assert!(layout.move_tab(
        &"timeline",
        DockPlacement::Tabs {
            container: "viewer",
            index: None
        }
    ));
    assert_eq!(container_of(&layout, "viewer"), ["viewer", "timeline"]);
    assert_eq!(
        layout.container(&"viewer").unwrap().active,
        Some("timeline")
    );
    assert_eq!(container_of(&layout, "render"), ["render"]);

    // Beside a container: a new container splits it in half.
    let mut layout = editor_layout();
    assert!(layout.move_tab(
        &"text",
        DockPlacement::Split {
            container: "viewer",
            side: DockSide::Right
        }
    ));
    assert_eq!(container_of(&layout, "text"), ["text"]);
    assert_eq!(
        layout.tab_ids(),
        [
            "project",
            "viewer",
            "text",
            "timeline",
            "render",
            "inspector"
        ]
    );
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    let DockNode::Split { children, .. } = &children[0].node else {
        panic!("left column");
    };
    let DockNode::Split { children: row, .. } = &children[0].node else {
        panic!("top row");
    };
    assert_eq!(row.len(), 3, "the new container joined the same-axis row");
    assert!(
        near(row[1].weight, row[2].weight, 1e-9),
        "the target was halved"
    );

    // Above a container of the other axis: a new two-child split.
    let mut layout = editor_layout();
    assert!(layout.move_tab(
        &"render",
        DockPlacement::Split {
            container: "inspector",
            side: DockSide::Top
        }
    ));
    assert_eq!(container_of(&layout, "render"), ["render"]);
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    let DockNode::Split { axis, children } = &children[1].node else {
        panic!("the inspector column became a split");
    };
    assert_eq!(*axis, DockAxis::Vertical);
    assert_eq!(children.len(), 2);
    assert!(
        matches!(&children[0].node, DockNode::Tabs(t) if t.contains(&"render")),
        "Top puts the new container first"
    );

    // Along a dock edge: a third of the same-axis split, a quarter otherwise.
    let mut layout = editor_layout();
    assert!(layout.move_tab(&"timeline", DockPlacement::Edge(DockSide::Left)));
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    assert_eq!(children.len(), 3);
    assert!(matches!(&children[0].node, DockNode::Tabs(t) if t.contains(&"timeline")));
    assert!(near(children[0].weight, 0.25, 1e-9));
    assert_eq!(container_of(&layout, "render"), ["render"]);
    let mut layout = editor_layout();
    assert!(layout.move_tab(&"inspector", DockPlacement::Edge(DockSide::Bottom)));
    assert_eq!(layout.tab_ids().last(), Some(&"inspector"));
    let DockNode::Split { axis, children } = &layout.root else {
        panic!("root is a vertical split now");
    };
    assert_eq!(*axis, DockAxis::Vertical);
    assert_eq!(
        children.len(),
        3,
        "the old column merged into the new split"
    );
    assert!(near(children[2].weight, 0.25, 1e-9));
    assert!(near(children[0].weight + children[1].weight, 0.75, 1e-9));
}

#[test]
fn move_tab_rejects_the_moves_that_cannot_change_anything() {
    let mut layout = editor_layout();
    let before = layout.clone();
    for to in [
        DockPlacement::Tabs {
            container: "inspector",
            index: None,
        },
        DockPlacement::Split {
            container: "inspector",
            side: DockSide::Left,
        },
        DockPlacement::Tabs {
            container: "nope",
            index: None,
        },
        DockPlacement::Split {
            container: "nope",
            side: DockSide::Left,
        },
    ] {
        assert!(!layout.move_tab(&"inspector", to), "the only tab stays");
        assert_eq!(layout, before);
    }
    assert!(!layout.move_tab(&"nope", DockPlacement::Edge(DockSide::Left)));
    // A lone tab of a lone container cannot go to an edge either.
    let mut alone = DockLayout::new(tabs_of(&["only"]));
    assert!(!alone.move_tab(&"only", DockPlacement::Edge(DockSide::Left)));
    // With several tabs it can.
    let mut two = DockLayout::new(tabs_of(&["a", "b"]));
    assert!(two.move_tab(&"b", DockPlacement::Edge(DockSide::Right)));
    assert_eq!(two.containers().len(), 2);
    // A tab can split its own container when it has company.
    let mut split = DockLayout::new(tabs_of(&["a", "b"]));
    assert!(split.move_tab(
        &"a",
        DockPlacement::Split {
            container: "a",
            side: DockSide::Left
        }
    ));
    assert_eq!(container_of(&split, "a"), ["a"]);
    assert_eq!(container_of(&split, "b"), ["b"]);
}

#[test]
fn moving_between_containers_ends_maximization_but_reordering_keeps_it() {
    let mut layout = editor_layout();
    assert!(layout.maximize(&"text"));
    assert!(!layout.maximize(&"text"), "already maximized");
    assert!(layout.move_tab(
        &"project",
        DockPlacement::Tabs {
            container: "project",
            index: Some(1)
        }
    ));
    assert_eq!(layout.maximized, Some("text"), "reordering keeps it");
    assert!(layout.move_tab(
        &"project",
        DockPlacement::Split {
            container: "viewer",
            side: DockSide::Bottom
        }
    ));
    assert_eq!(layout.maximized, None, "leaving the container ends it");
    assert!(!layout.restore());
    assert!(!layout.maximize(&"nope"));
    assert!(layout.maximize(&"viewer"));
    assert_eq!(layout.maximized_container().map(|c| c.tabs.len()), Some(1));
    assert!(layout.restore());
    assert_eq!(layout.maximized, None);
}

#[test]
fn close_hands_over_the_active_tab_and_respects_closable() {
    let mut layout = editor_layout();
    assert!(layout.activate(&"render"));
    assert_eq!(layout.close(&"render").map(|tab| tab.id), Some("render"));
    assert_eq!(
        layout.container(&"timeline").unwrap().active,
        Some("timeline"),
        "the active tab hands over to its neighbor"
    );
    assert!(layout.close(&"render").is_none());
    // The last tab removes its container and collapses the split.
    assert_eq!(
        layout.close(&"timeline").map(|tab| tab.id),
        Some("timeline")
    );
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    assert_eq!(children.len(), 3, "the vertical split merged into the row");
    assert!(near(
        children.iter().map(|c| c.weight).sum::<f64>(),
        1.0,
        1e-9
    ));

    let mut layout = DockLayout::new(DockNode::tabs(vec![
        DockTab::new("a", "A"),
        DockTab::new("b", "B").closable(false),
        DockTab::new("c", "C"),
        DockTab::new("d", "D"),
    ]));
    assert!(layout.activate(&"c"));
    let closed = layout.close_others(&"c");
    assert_eq!(
        closed.iter().map(|tab| tab.id).collect::<Vec<_>>(),
        ["a", "d"],
        "the closable tabs other than c"
    );
    assert_eq!(layout.tab_ids(), ["b", "c"]);
    let closed = layout.close_container(&"c");
    assert_eq!(closed.iter().map(|tab| tab.id).collect::<Vec<_>>(), ["c"]);
    assert_eq!(layout.tab_ids(), ["b"]);
    assert!(layout.close_container(&"b").is_empty());
    assert!(layout.close_others(&"nope").is_empty());

    // Closing a maximized container's tab keeps it maximized while tabs remain.
    let mut layout = editor_layout();
    assert!(layout.maximize(&"project"));
    assert!(layout.close(&"project").is_some());
    assert_eq!(layout.maximized, Some("text"));
    assert!(layout.close(&"text").is_some());
    assert_eq!(layout.maximized, None);
}

#[test]
fn set_weights_checks_the_path_and_count_and_renormalizes() {
    let mut layout = editor_layout();
    assert!(layout.set_weights(&[], &[2.0, 2.0]));
    assert_eq!(weights(&layout.root), [0.5, 0.5]);
    assert!(layout.set_weights(&[0], &[3.0, 1.0]));
    let DockNode::Split { children, .. } = &layout.root else {
        panic!("root stays a split");
    };
    assert_eq!(weights(&children[0].node), [0.75, 0.25]);
    assert!(!layout.set_weights(&[], &[1.0]), "wrong count");
    assert!(
        !layout.set_weights(&[1], &[1.0]),
        "a container has no weights"
    );
    assert!(!layout.set_weights(&[7], &[1.0]), "no such child");
}
