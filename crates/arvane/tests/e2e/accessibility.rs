//! Role: Accessibility contract tests.
//!
//! Accessibility: the window's one tree, replaced and updated
//! transactionally, and assistive technology's requests checked against
//! it.

use arvane::{AccessibilityError, ShellEvent, Window as _};
use kivact_types::{
    A11yAction, NodeId, Point, Role, SemanticsNode, SemanticsProperties, SemanticsTree,
    SemanticsUpdate, Transform,
};

use crate::support::{open, rect};

fn node(id: u64, role: Role, children: &[u64]) -> SemanticsNode {
    SemanticsNode {
        properties: None,
        id: NodeId(id),
        role,
        children: children.iter().copied().map(NodeId).collect(),
        label: format!("node {id}"),
        description: String::new(),
        value: None,
        bounds: rect(0.0, 0.0, 10.0, 10.0),
        scroll: Point::default(),
        transform: Transform::IDENTITY,
        disabled: false,
        hidden: false,
        focusable: false,
        checked: None,
        selected: false,
        expanded: None,
        test_id: None,
        actions: Vec::new(),
    }
}

fn button(id: u64) -> SemanticsNode {
    SemanticsNode {
        properties: None,
        focusable: true,
        actions: vec![A11yAction::Activate],
        ..node(id, Role::Button, &[])
    }
}

/// Root 1 holds group 2 (buttons 3 and 4) and text 5.
fn tree() -> SemanticsTree {
    SemanticsTree {
        root: NodeId(1),
        focus: Some(NodeId(3)),
        nodes: vec![
            node(1, Role::Generic, &[2, 5]),
            node(2, Role::Group, &[3, 4]),
            button(3),
            button(4),
            node(5, Role::Text, &[]),
        ],
    }
}

fn ids(tree: &SemanticsTree) -> Vec<u64> {
    tree.nodes.iter().map(|n| n.id.0).collect()
}

#[test]
fn partial_checkbox_preserves_binary_value_across_tree_updates() {
    let mut window = open(100.0, 100.0);
    let mut partial = node(2, Role::CheckBox, &[]);
    partial.checked = Some(false);
    partial.properties = Some(Box::new(SemanticsProperties {
        indeterminate: true,
        ..Default::default()
    }));
    window
        .replace_accessibility(&SemanticsTree {
            root: NodeId(1),
            focus: None,
            nodes: vec![node(1, Role::Generic, &[2]), partial.clone()],
        })
        .expect("valid mixed checkbox");
    let accepted = window.accessibility().tree().expect("published tree");
    assert_eq!(accepted.nodes[1].checked, Some(false));
    assert!(
        accepted.nodes[1]
            .properties
            .as_ref()
            .expect("properties")
            .indeterminate
    );
    partial.checked = Some(true);
    partial.properties = None;
    window
        .update_accessibility(&SemanticsUpdate {
            root: NodeId(1),
            focus: None,
            nodes: vec![partial],
        })
        .expect("valid complete checkbox");
    let accepted = window.accessibility().tree().expect("updated tree");
    assert_eq!(accepted.nodes[1].checked, Some(true));
    assert!(accepted.nodes[1].properties.is_none());
}

#[test]
fn a_replaced_tree_takes_updates_of_the_nodes_that_changed() {
    let mut window = open(100.0, 100.0);
    assert_eq!(window.accessibility().tree(), None);
    window.replace_accessibility(&tree()).expect("a valid tree");
    assert_eq!(window.accessibility().tree(), Some(tree()));
    // A label changes, the focus moves, a node is added under the group.
    let update = SemanticsUpdate {
        root: NodeId(1),
        focus: Some(NodeId(6)),
        nodes: vec![
            SemanticsNode {
                properties: None,
                label: "renamed".into(),
                ..button(4)
            },
            node(2, Role::Group, &[3, 4, 6]),
            button(6),
        ],
    };
    window
        .update_accessibility(&update)
        .expect("a valid update");
    let accepted = window.accessibility().tree().expect("a tree");
    assert_eq!(accepted.focus, Some(NodeId(6)));
    assert_eq!(
        ids(&accepted),
        [1, 2, 3, 4, 6, 5],
        "parents before children"
    );
    assert_eq!(accepted.nodes[3].label, "renamed");
    // Dropping the group from the root takes its subtree with it.
    window
        .update_accessibility(&SemanticsUpdate {
            root: NodeId(1),
            focus: None,
            nodes: vec![node(1, Role::Generic, &[5])],
        })
        .expect("a valid update");
    assert_eq!(ids(&window.accessibility().tree().expect("a tree")), [1, 5]);
}

#[test]
fn an_invalid_tree_or_update_leaves_the_accepted_tree_as_it_was() {
    let mut window = open(100.0, 100.0);
    window.replace_accessibility(&tree()).expect("a valid tree");
    let replaced = |nodes: Vec<SemanticsNode>, focus: Option<u64>| SemanticsTree {
        root: NodeId(1),
        focus: focus.map(NodeId),
        nodes,
    };
    let rejected_trees = [
        (
            replaced(
                vec![node(1, Role::Generic, &[2]), node(2, Role::Group, &[1])],
                None,
            ),
            AccessibilityError::InvalidTree(NodeId(1)),
        ),
        (
            replaced(
                vec![node(1, Role::Generic, &[]), node(2, Role::Group, &[])],
                None,
            ),
            AccessibilityError::UnreachableNode(NodeId(2)),
        ),
        (
            replaced(vec![node(1, Role::Generic, &[9])], None),
            AccessibilityError::MissingNode(NodeId(9)),
        ),
        (
            replaced(
                vec![node(1, Role::Generic, &[2]), node(2, Role::Text, &[])],
                Some(2),
            ),
            AccessibilityError::InvalidFocus(NodeId(2)),
        ),
        (
            replaced(
                vec![
                    node(1, Role::Generic, &[2]),
                    SemanticsNode {
                        properties: None,
                        bounds: rect(0.0, 0.0, -1.0, 4.0),
                        ..node(2, Role::Text, &[])
                    },
                ],
                None,
            ),
            AccessibilityError::InvalidBounds(NodeId(2)),
        ),
    ];
    for (bad, expected) in rejected_trees {
        assert_eq!(window.replace_accessibility(&bad), Err(expected.clone()));
        assert_eq!(window.accessibility().tree(), Some(tree()), "{expected:?}");
    }
    let update = |focus: Option<u64>, nodes: Vec<SemanticsNode>| SemanticsUpdate {
        root: NodeId(1),
        focus: focus.map(NodeId),
        nodes,
    };
    let rejected_updates = [
        // A cycle: the group under its own button.
        (
            update(
                Some(3),
                vec![SemanticsNode {
                    properties: None,
                    children: vec![NodeId(2)],
                    ..button(3)
                }],
            ),
            AccessibilityError::InvalidTree(NodeId(2)),
        ),
        // A node no parent lists.
        (
            update(Some(3), vec![button(7)]),
            AccessibilityError::UnreachableNode(NodeId(7)),
        ),
        // A child that does not exist.
        (
            update(Some(3), vec![node(2, Role::Group, &[3, 4, 8])]),
            AccessibilityError::MissingNode(NodeId(8)),
        ),
        // The focus on a disabled node, on a hidden one, and in a subtree
        // the update removes.
        (
            update(
                Some(4),
                vec![SemanticsNode {
                    properties: None,
                    disabled: true,
                    ..button(4)
                }],
            ),
            AccessibilityError::InvalidFocus(NodeId(4)),
        ),
        (
            update(
                Some(4),
                vec![SemanticsNode {
                    properties: None,
                    hidden: true,
                    ..node(2, Role::Group, &[3, 4])
                }],
            ),
            AccessibilityError::InvalidFocus(NodeId(4)),
        ),
        (
            update(Some(3), vec![node(1, Role::Generic, &[5])]),
            AccessibilityError::InvalidFocus(NodeId(3)),
        ),
        // A node listed twice.
        (
            update(
                Some(3),
                vec![SemanticsNode {
                    properties: Some(Box::new(kivact_types::SemanticsProperties {
                        value_now: Some(f64::NAN),
                        ..Default::default()
                    })),
                    ..button(4)
                }],
            ),
            AccessibilityError::InvalidProperties(NodeId(4)),
        ),
        (
            update(
                Some(3),
                vec![SemanticsNode {
                    properties: Some(Box::new(kivact_types::SemanticsProperties {
                        active_descendant: Some(NodeId(3)),
                        ..Default::default()
                    })),
                    ..button(4)
                }],
            ),
            AccessibilityError::InvalidProperties(NodeId(4)),
        ),
        // An active descendant that is no node at all.
        (
            update(
                Some(3),
                vec![SemanticsNode {
                    properties: Some(Box::new(kivact_types::SemanticsProperties {
                        active_descendant: Some(NodeId(99)),
                        ..Default::default()
                    })),
                    ..button(4)
                }],
            ),
            AccessibilityError::MissingNode(NodeId(99)),
        ),
        // A node outside the one it controls does not count.
        (
            update(
                Some(3),
                vec![SemanticsNode {
                    properties: Some(Box::new(kivact_types::SemanticsProperties {
                        active_descendant: Some(NodeId(5)),
                        controls: vec![NodeId(2)],
                        ..Default::default()
                    })),
                    ..button(4)
                }],
            ),
            AccessibilityError::InvalidProperties(NodeId(4)),
        ),
        (
            update(Some(3), vec![button(4), button(4)]),
            AccessibilityError::DuplicateNode(NodeId(4)),
        ),
    ];
    for (bad, expected) in rejected_updates {
        assert_eq!(window.update_accessibility(&bad), Err(expected.clone()));
        assert_eq!(window.accessibility().tree(), Some(tree()), "{expected:?}");
    }
}

#[test]
fn requests_of_assistive_technology_are_checked_against_the_tree() {
    let mut window = open(100.0, 100.0);
    let event = |node: u64, action: A11yAction, value: Option<&str>| ShellEvent::Accessibility {
        node,
        action,
        value: value.map(str::to_owned),
    };
    assert_eq!(
        window.accessibility_action(3, A11yAction::Activate, None),
        None,
        "no tree yet"
    );
    let mut tree = tree();
    tree.nodes.push(SemanticsNode {
        properties: None,
        actions: vec![
            A11yAction::Toggle,
            A11yAction::SetValue,
            A11yAction::Dismiss,
        ],
        ..node(6, Role::CheckBox, &[])
    });
    tree.nodes[0].children.push(NodeId(6));
    window.replace_accessibility(&tree).expect("a valid tree");
    let request = |node: u64, action: A11yAction, value: Option<&str>| {
        window.accessibility_action(node, action, value)
    };
    assert_eq!(
        request(3, A11yAction::Activate, None),
        Some(event(3, A11yAction::Activate, None))
    );
    assert_eq!(
        request(3, A11yAction::Focus, None),
        Some(event(3, A11yAction::Focus, None))
    );
    // One default action: a toggle arrives as Activate.
    assert_eq!(
        request(6, A11yAction::Toggle, None),
        Some(event(6, A11yAction::Activate, None))
    );
    assert_eq!(
        request(6, A11yAction::SetValue, Some("on")),
        Some(event(6, A11yAction::SetValue, Some("on")))
    );
    assert_eq!(
        request(6, A11yAction::Dismiss, None),
        Some(event(6, A11yAction::Dismiss, None))
    );
    // Not declared, not focusable, absent.
    assert_eq!(request(3, A11yAction::Increment, None), None);
    assert_eq!(request(5, A11yAction::Focus, None), None);
    assert_eq!(request(6, A11yAction::Focus, None), None);
    assert_eq!(request(42, A11yAction::Activate, None), None);
    // A disabled or hidden ancestor drops the request.
    for change in [
        SemanticsNode {
            properties: None,
            disabled: true,
            ..node(2, Role::Group, &[3, 4])
        },
        SemanticsNode {
            properties: None,
            hidden: true,
            ..node(2, Role::Group, &[3, 4])
        },
    ] {
        window.replace_accessibility(&tree).expect("a valid tree");
        window
            .update_accessibility(&SemanticsUpdate {
                root: NodeId(1),
                focus: None,
                nodes: vec![change],
            })
            .expect("a valid update");
        assert_eq!(
            window.accessibility_action(3, A11yAction::Activate, None),
            None
        );
        assert_eq!(
            window.accessibility_action(4, A11yAction::Focus, None),
            None
        );
        assert!(
            window
                .accessibility_action(6, A11yAction::Toggle, None)
                .is_some()
        );
    }
    // So does a disabled node itself.
    window
        .update_accessibility(&SemanticsUpdate {
            root: NodeId(1),
            focus: None,
            nodes: vec![SemanticsNode {
                properties: None,
                disabled: true,
                ..tree.nodes[5].clone()
            }],
        })
        .expect("a valid update");
    assert_eq!(
        window.accessibility_action(6, A11yAction::Toggle, None),
        None
    );
}

/// `node` with relations to other nodes.
fn related(
    mut node: SemanticsNode,
    described_by: &[u64],
    controls: &[u64],
    labelled_by: &[u64],
) -> SemanticsNode {
    let ids = |ids: &[u64]| ids.iter().copied().map(NodeId).collect();
    node.properties = Some(Box::new(SemanticsProperties {
        described_by: ids(described_by),
        controls: ids(controls),
        labelled_by: ids(labelled_by),
        ..Default::default()
    }));
    node
}

#[test]
fn a_relation_to_a_node_the_tree_lacks_is_rejected() {
    let mut window = open(100.0, 100.0);
    for (described_by, controls, labelled_by) in [
        (&[9][..], &[][..], &[][..]),
        (&[][..], &[9][..], &[][..]),
        (&[][..], &[][..], &[9][..]),
    ] {
        let mut bad = tree();
        bad.nodes[2] = related(button(3), described_by, controls, labelled_by);
        assert_eq!(
            window.replace_accessibility(&bad),
            Err(AccessibilityError::MissingNode(NodeId(9)))
        );
    }
    assert_eq!(window.accessibility().tree(), None);
    window.replace_accessibility(&tree()).expect("a valid tree");
    // An update naming a node that is not in the tree.
    let bad = SemanticsUpdate {
        root: NodeId(1),
        focus: Some(NodeId(3)),
        nodes: vec![related(button(3), &[9], &[], &[])],
    };
    assert_eq!(
        window.update_accessibility(&bad),
        Err(AccessibilityError::MissingNode(NodeId(9)))
    );
    // An update that removes the node a relation of an unchanged node names.
    let mut linked = tree();
    linked.nodes[2] = related(button(3), &[5], &[], &[]);
    window.replace_accessibility(&linked).expect("a valid tree");
    let removes_target = SemanticsUpdate {
        root: NodeId(1),
        focus: Some(NodeId(3)),
        nodes: vec![node(1, Role::Generic, &[2])],
    };
    assert_eq!(
        window.update_accessibility(&removes_target),
        Err(AccessibilityError::MissingNode(NodeId(5)))
    );
    assert_eq!(window.accessibility().tree(), Some(linked));
}

#[test]
fn an_active_descendant_may_live_inside_a_node_the_node_controls() {
    // A text box (4) beside the list (2) it controls; the active option (3)
    // is in the list, not under the text box (the ARIA combobox pattern).
    let related = |active: u64| SemanticsNode {
        properties: Some(Box::new(SemanticsProperties {
            active_descendant: Some(NodeId(active)),
            controls: vec![NodeId(2)],
            ..Default::default()
        })),
        ..button(4)
    };
    let mut bad = tree();
    bad.nodes[3] = related(5);
    let mut window = open(100.0, 100.0);
    assert_eq!(
        window.replace_accessibility(&bad),
        Err(AccessibilityError::InvalidProperties(NodeId(4)))
    );
    // Node 4 is inside group 2 here; give the text box its own place.
    let tree = |active: u64| SemanticsTree {
        root: NodeId(1),
        focus: Some(NodeId(4)),
        nodes: vec![
            node(1, Role::Generic, &[2, 4, 5]),
            node(2, Role::ListBox, &[3]),
            SemanticsNode {
                role: Role::Option,
                actions: Vec::new(),
                ..button(3)
            },
            related(active),
            node(5, Role::Text, &[]),
        ],
    };
    window
        .replace_accessibility(&tree(3))
        .expect("an option of the controlled list");
    let accepted = window.accessibility().tree().expect("a tree");
    let field = accepted.nodes.iter().find(|n| n.id == NodeId(4));
    assert_eq!(
        field
            .and_then(|n| n.properties.as_ref())
            .and_then(|p| p.active_descendant),
        Some(NodeId(3))
    );
    // The controlled list itself counts, and an update checks the same rule.
    window
        .update_accessibility(&SemanticsUpdate {
            root: NodeId(1),
            focus: Some(NodeId(4)),
            nodes: vec![related(2)],
        })
        .expect("the controlled node itself");
    assert_eq!(
        window.update_accessibility(&SemanticsUpdate {
            root: NodeId(1),
            focus: Some(NodeId(4)),
            nodes: vec![related(5)],
        }),
        Err(AccessibilityError::InvalidProperties(NodeId(4)))
    );
}

/// `text`'s runs, changed by `f` (new runs where it changed them).
fn edit_runs(
    text: &mut kivact_types::SemanticsText,
    f: impl FnOnce(&mut Vec<kivact_types::SemanticsTextRun>),
) {
    let mut runs: Vec<_> = text.runs.iter().map(|r| (**r).clone()).collect();
    f(&mut runs);
    text.runs = runs.into_iter().map(std::sync::Arc::new).collect();
}

#[test]
fn editable_text_snapshots_validate_complete_and_partial_updates_transactionally() {
    use kivact_types::{SemanticsText, SemanticsTextRun, TextCaret};
    use std::sync::Arc;
    let mut window = open(100., 100.);
    let mut editor = node(2, Role::TextBox, &[]);
    editor.properties = Some(Box::new(SemanticsProperties {
        text: Some(SemanticsText {
            runs: Arc::from([Arc::new(SemanticsTextRun {
                id: NodeId(10),
                value: "a🦀".into(),
                bounds: rect(0., 0., 30., 15.),
                carets: vec![TextCaret {
                    offset: 2,
                    rect: rect(30., 0., 1., 15.),
                }],
            })]),
            start: 0,
            selection: (2, 0),
            caret: rect(0., 0., 1., 15.),
        }),
        ..Default::default()
    }));
    let original = SemanticsTree {
        root: NodeId(1),
        focus: None,
        nodes: vec![
            node(1, Role::Generic, &[2, 3]),
            editor.clone(),
            node(3, Role::Text, &[]),
        ],
    };
    window.replace_accessibility(&original).unwrap();
    let mut cases = Vec::new();
    for case in 0..10 {
        let mut bad = editor.clone();
        let props = bad.properties.as_mut().unwrap();
        let text = props.text.as_mut().unwrap();
        let expected = match case {
            0 => {
                edit_runs(text, |runs| runs[0].id = NodeId(3));
                AccessibilityError::DuplicateNode(NodeId(3))
            }
            1 => {
                edit_runs(text, |runs| runs.push(runs[0].clone()));
                AccessibilityError::DuplicateNode(NodeId(10))
            }
            2 => {
                // The run itself, given again, is the same run.
                text.runs = Arc::from([text.runs[0].clone(), text.runs[0].clone()]);
                AccessibilityError::DuplicateNode(NodeId(10))
            }
            3 => {
                text.selection = (3, 0);
                AccessibilityError::InvalidProperties(NodeId(2))
            }
            4 => {
                text.caret.x = f32::NAN;
                AccessibilityError::InvalidBounds(NodeId(2))
            }
            5 => {
                edit_runs(text, |runs| runs[0].bounds.width = -1.);
                AccessibilityError::InvalidBounds(NodeId(10))
            }
            6 => {
                edit_runs(text, |runs| runs[0].carets[0].rect.y = f32::INFINITY);
                AccessibilityError::InvalidBounds(NodeId(10))
            }
            7 => {
                edit_runs(text, |runs| runs[0].carets[0].offset = 3);
                AccessibilityError::InvalidProperties(NodeId(2))
            }
            8 => {
                edit_runs(text, Vec::clear);
                AccessibilityError::InvalidProperties(NodeId(2))
            }
            _ => {
                props.protected = true;
                AccessibilityError::InvalidProperties(NodeId(2))
            }
        };
        cases.push((bad, expected));
    }
    for (bad, expected) in cases {
        let mut complete = original.clone();
        complete.nodes[1] = bad.clone();
        assert_eq!(
            window.replace_accessibility(&complete),
            Err(expected.clone())
        );
        assert_eq!(window.accessibility().tree(), Some(original.clone()));
        assert_eq!(
            window.update_accessibility(&SemanticsUpdate {
                root: NodeId(1),
                focus: None,
                nodes: vec![bad]
            }),
            Err(expected)
        );
        assert_eq!(window.accessibility().tree(), Some(original.clone()));
    }
    editor
        .properties
        .as_mut()
        .unwrap()
        .text
        .as_mut()
        .unwrap()
        .selection = (0, 2);
    window
        .update_accessibility(&SemanticsUpdate {
            root: NodeId(1),
            focus: None,
            nodes: vec![editor.clone()],
        })
        .unwrap();
    assert_eq!(
        window.accessibility().tree().unwrap().nodes[1]
            .properties
            .as_ref()
            .unwrap()
            .text
            .as_ref()
            .unwrap()
            .selection,
        (0, 2)
    );
    editor.properties.as_mut().unwrap().text = None;
    window
        .update_accessibility(&SemanticsUpdate {
            root: NodeId(1),
            focus: None,
            nodes: vec![editor],
        })
        .unwrap();
    assert!(
        window.accessibility().tree().unwrap().nodes[1]
            .properties
            .as_ref()
            .unwrap()
            .text
            .is_none()
    );
}
