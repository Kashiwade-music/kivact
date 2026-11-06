//! Role: Accessibility contract tests.
//!
//! Accessibility: the window's one tree, replaced and updated
//! transactionally, and assistive technology's requests checked against
//! it.

use arvane::ShellEvent;
use kivact_types::Transform;
use kivact_types::render::{SemanticsNode, SemanticsTree, SemanticsUpdate};
use kivact_types::{A11yAction, A11yRequest, DismissReason, Point, Role, SemanticsId};

use crate::support::{open, rect};

fn node(id: u64, role: Role, children: &[u64]) -> SemanticsNode {
    SemanticsNode {
        id: SemanticsId(id),
        children: children.iter().copied().map(SemanticsId).collect(),
        bounds: rect(0.0, 0.0, 10.0, 10.0),
        scroll: Point::default(),
        transform: Transform::IDENTITY,
        semantics: kivact_types::Semantics {
            role: Some(role),
            name: Some(format!("node {id}")),
            description: Some(String::new()),
            value_text: None,
            disabled: false,
            hidden: false,
            focusable: false,
            checked: None,
            selected: Some(false),
            expanded: None,
            test_id: None,
            actions: Some(Vec::new()),
            ..Default::default()
        },
    }
}

fn button(id: u64) -> SemanticsNode {
    {
        let base = node(id, Role::Button, &[]);
        SemanticsNode {
            semantics: kivact_types::Semantics {
                focusable: true,
                actions: Some(vec![A11yAction::Activate]),
                ..base.semantics
            },
            ..base
        }
    }
}

/// Root 1 holds group 2 (buttons 3 and 4) and text 5.
fn tree() -> SemanticsTree {
    SemanticsTree {
        root: SemanticsId(1),
        focus: Some(SemanticsId(3)),
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
    assert_eq!(window.probe().accessibility(), None);
    window
        .with(move |w| w.replace_accessibility(&tree()))
        .expect("a valid tree");
    assert_eq!(window.probe().accessibility(), Some(tree()));
    // A label changes, the focus moves, a node is added under the group.
    let update = SemanticsUpdate {
        root: SemanticsId(1),
        focus: Some(SemanticsId(6)),
        nodes: vec![
            {
                let base = button(4);
                SemanticsNode {
                    semantics: kivact_types::Semantics {
                        name: Some("renamed".into()),
                        ..base.semantics
                    },
                    ..base
                }
            },
            node(2, Role::Group, &[3, 4, 6]),
            button(6),
        ],
    };
    window
        .with(move |w| w.update_accessibility(&update))
        .expect("a valid update");
    let accepted = window.probe().accessibility().expect("a tree");
    assert_eq!(accepted.focus, Some(SemanticsId(6)));
    assert_eq!(
        ids(&accepted),
        [1, 2, 3, 4, 6, 5],
        "parents before children"
    );
    assert_eq!(accepted.nodes[3].semantics.name.as_deref(), Some("renamed"));
    // Dropping the group from the root takes its subtree with it.
    window
        .with(move |w| {
            w.update_accessibility(&SemanticsUpdate {
                root: SemanticsId(1),
                focus: None,
                nodes: vec![node(1, Role::Generic, &[5])],
            })
        })
        .expect("a valid update");
    assert_eq!(
        ids(&window.probe().accessibility().expect("a tree")),
        [1, 5]
    );
}

#[test]
fn requests_of_assistive_technology_are_checked_against_the_tree() {
    let mut window = open(100.0, 100.0);
    let event = |node: u64, request: A11yRequest| ShellEvent::Accessibility { node, request };
    assert_eq!(
        request(&mut window, 3, A11yRequest::Activate),
        None,
        "no tree yet"
    );
    let tree = action_tree();
    window
        .with({
            let tree = tree.clone();
            move |w| w.replace_accessibility(&tree)
        })
        .expect("a valid tree");
    assert_eq!(
        request(&mut window, 3, A11yRequest::Activate),
        Some(event(3, A11yRequest::Activate))
    );
    assert_eq!(
        request(&mut window, 3, A11yRequest::Focus),
        Some(event(3, A11yRequest::Focus))
    );
    // One default action: a toggle arrives as Activate.
    assert_eq!(
        request(&mut window, 6, A11yRequest::Toggle),
        Some(event(6, A11yRequest::Activate))
    );
    assert_eq!(
        request(&mut window, 6, A11yRequest::SetValue("on".to_string())),
        Some(event(6, A11yRequest::SetValue("on".to_string())))
    );
    assert_eq!(
        request(
            &mut window,
            6,
            A11yRequest::Dismiss(DismissReason::Accessibility)
        ),
        Some(event(6, A11yRequest::Dismiss(DismissReason::Accessibility)))
    );
    // Not declared, not focusable, absent.
    assert_eq!(request(&mut window, 3, A11yRequest::Increment), None);
    assert_eq!(request(&mut window, 5, A11yRequest::Focus), None);
    assert_eq!(request(&mut window, 6, A11yRequest::Focus), None);
    assert_eq!(request(&mut window, 42, A11yRequest::Activate), None);
    check_ancestor_requests(&mut window, &tree);
    // So does a disabled node itself.
    let original = tree.nodes[5].clone();
    window
        .with(move |w| {
            w.update_accessibility(&SemanticsUpdate {
                root: SemanticsId(1),
                focus: None,
                nodes: vec![{
                    let base = original;
                    SemanticsNode {
                        semantics: kivact_types::Semantics {
                            disabled: true,
                            ..base.semantics
                        },
                        ..base
                    }
                }],
            })
        })
        .expect("a valid update");
    assert_eq!(request(&mut window, 6, A11yRequest::Toggle), None);
}

fn request(
    window: &mut crate::support::Harness,
    node: u64,
    request: A11yRequest,
) -> Option<ShellEvent> {
    window.script().accessibility_action(node, request);
    window.events().into_iter().next()
}

fn action_tree() -> SemanticsTree {
    let mut tree = tree();
    tree.nodes.push({
        let base = node(6, Role::CheckBox, &[]);
        SemanticsNode {
            semantics: kivact_types::Semantics {
                actions: Some(vec![
                    A11yAction::Toggle,
                    A11yAction::SetValue,
                    A11yAction::Dismiss,
                ]),
                ..base.semantics
            },
            ..base
        }
    });
    tree.nodes[0].children.push(SemanticsId(6));
    tree
}

fn check_ancestor_requests(window: &mut crate::support::Harness, tree: &SemanticsTree) {
    // A disabled or hidden ancestor drops the request.
    for change in [
        {
            let base = node(2, Role::Group, &[3, 4]);
            SemanticsNode {
                semantics: kivact_types::Semantics {
                    disabled: true,
                    ..base.semantics
                },
                ..base
            }
        },
        {
            let base = node(2, Role::Group, &[3, 4]);
            SemanticsNode {
                semantics: kivact_types::Semantics {
                    hidden: true,
                    ..base.semantics
                },
                ..base
            }
        },
    ] {
        window
            .with({
                let tree = tree.clone();
                move |w| w.replace_accessibility(&tree)
            })
            .expect("a valid tree");
        window
            .with(move |w| {
                w.update_accessibility(&SemanticsUpdate {
                    root: SemanticsId(1),
                    focus: None,
                    nodes: vec![change],
                })
            })
            .expect("a valid update");
        assert_eq!(request(window, 3, A11yRequest::Activate), None);
        assert_eq!(request(window, 4, A11yRequest::Focus), None);
        assert!(request(window, 6, A11yRequest::Toggle).is_some());
    }
}
