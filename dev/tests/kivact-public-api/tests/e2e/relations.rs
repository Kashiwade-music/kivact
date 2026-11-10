//! Role: Tests for accessibility relations between elements.
//!
//! Relations between elements (`described_by`, `controls`, `labelled_by`) and
//! `Role::AlertDialog` reach the host's accessibility tree.

use crate::support::mount;
use kivact::prelude::*;

#[test]
fn relations_reach_the_accessibility_tree_in_call_order() {
    let (screen, ()) = mount(|| {
        let message = node_ref();
        let detail = node_ref();
        let heading = node_ref();
        let panel = node_ref();
        (
            (),
            column((
                node()
                    .role(Role::AlertDialog)
                    .name("Delete file")
                    .labelled_by(heading)
                    .described_by(message)
                    .described_by(detail)
                    .child((
                        text("Delete file").node_ref(heading),
                        text("This cannot be undone.").node_ref(message),
                        text("The file is removed for everyone.").node_ref(detail),
                    )),
                node()
                    .role(Role::Button)
                    .name("Details")
                    .focusable(true)
                    .controls(panel),
                node()
                    .role(Role::Group)
                    .name("Details panel")
                    .node_ref(panel),
            )),
        )
    });
    let dialog = screen.get_by_role(Role::AlertDialog, "Delete file");
    assert_eq!(dialog.role(), Some(Role::AlertDialog));
    let described: Vec<String> = dialog.described_by().iter().map(|e| e.text()).collect();
    assert_eq!(
        described,
        [
            "This cannot be undone.",
            "The file is removed for everyone."
        ]
    );
    let labels: Vec<String> = dialog.labelled_by().iter().map(|e| e.text()).collect();
    assert_eq!(labels, ["Delete file"]);
    assert!(dialog.controls().is_empty());
    let button = screen.get_by_role(Role::Button, "Details");
    let controlled: Vec<String> = button.controls().iter().map(|e| e.name()).collect();
    assert_eq!(controlled, ["Details panel"]);
    assert!(button.described_by().is_empty());
    assert!(screen.errors().is_empty());
}

#[test]
fn a_relation_disappears_with_its_target_and_returns_with_a_new_one() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        let hint = node_ref();
        (
            open,
            column((
                node().role(Role::Group).name("Field").described_by(hint),
                show(open, move || text("Use letters only.").node_ref(hint)),
            )),
        )
    });
    let field = screen.get_by_role(Role::Group, "Field");
    assert_eq!(field.described_by().len(), 1);
    open.set(false);
    screen.run_until_idle();
    assert!(field.described_by().is_empty());
    assert!(screen.errors().is_empty());
    open.set(true);
    screen.run_until_idle();
    let described = field.described_by();
    assert_eq!(described.len(), 1);
    assert_eq!(described[0].text(), "Use letters only.");
    assert!(screen.errors().is_empty());
}

#[test]
fn constant_semantics_publish_once_and_keep_no_publisher() {
    let (screen, (plain, named)) = mount(|| {
        let plain = signal(false);
        let named = signal(false);
        (
            (plain, named),
            column((
                show(plain, || node()),
                show(named, || node().role(Role::Button).name("Save")),
            )),
        )
    });
    let before_plain = screen.stats().live_reactive;
    plain.set(true);
    screen.run_until_idle();
    let plain_cost = screen.stats().live_reactive - before_plain;
    plain.set(false);
    screen.run_until_idle();
    let before_named = screen.stats().live_reactive;
    named.set(true);
    screen.run_until_idle();
    let named_cost = screen.stats().live_reactive - before_named;
    assert_eq!(
        named_cost, plain_cost,
        "constant semantics add no reactive node"
    );
    screen.get_by_role(Role::Button, "Save");
    assert!(screen.errors().is_empty());
}
