//! Role: Accessibility contract tests.
//!
//! Accessibility: the window's one tree, replaced and updated
//! transactionally, and assistive technology's requests checked against
//! it.

use arvane::{ShellEvent, Window as _};
use kivact_types::protocol::{SemanticsNode, SemanticsTree, SemanticsUpdate, Transform};
use kivact_types::{A11yAction, NodeId, Point, Role};

use crate::support::{open, rect};

fn node(id: u64, role: Role, children: &[u64]) -> SemanticsNode {
    SemanticsNode {
        properties: None,
        id: NodeId::new(id),
        role,
        children: children.iter().copied().map(NodeId::new).collect(),
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
        root: NodeId::new(1),
        focus: Some(NodeId::new(3)),
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
fn a_replaced_tree_takes_updates_of_the_nodes_that_changed() {
    let mut window = open(100.0, 100.0);
    assert_eq!(window.accessibility().tree(), None);
    window.replace_accessibility(&tree()).expect("a valid tree");
    assert_eq!(window.accessibility().tree(), Some(tree()));
    // A label changes, the focus moves, a node is added under the group.
    let update = SemanticsUpdate {
        root: NodeId::new(1),
        focus: Some(NodeId::new(6)),
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
    assert_eq!(accepted.focus, Some(NodeId::new(6)));
    assert_eq!(
        ids(&accepted),
        [1, 2, 3, 4, 6, 5],
        "parents before children"
    );
    assert_eq!(accepted.nodes[3].label, "renamed");
    // Dropping the group from the root takes its subtree with it.
    window
        .update_accessibility(&SemanticsUpdate {
            root: NodeId::new(1),
            focus: None,
            nodes: vec![node(1, Role::Generic, &[5])],
        })
        .expect("a valid update");
    assert_eq!(ids(&window.accessibility().tree().expect("a tree")), [1, 5]);
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
    tree.nodes[0].children.push(NodeId::new(6));
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
                root: NodeId::new(1),
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
            root: NodeId::new(1),
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
