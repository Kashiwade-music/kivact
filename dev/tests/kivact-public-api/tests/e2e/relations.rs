//! Role: Tests for accessibility relations between elements.
//!
//! Relations between elements (`described_by`, `controls`, `labelled_by`) and
//! `Role::AlertDialog` reach the host's accessibility tree.

use crate::support::mount;
use kivact::prelude::*;
use kivact::primitive::NodeExt;

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
fn a_reference_that_was_never_bound_is_left_out_without_an_error() {
    let (screen, ()) = mount(|| {
        let unbound = node_ref();
        let message = node_ref();
        (
            (),
            column((
                node()
                    .role(Role::Group)
                    .name("Field")
                    .described_by(unbound)
                    .described_by(message)
                    .controls(unbound)
                    .labelled_by(unbound),
                text("Hint").node_ref(message),
            )),
        )
    });
    let field = screen.get_by_role(Role::Group, "Field");
    let described: Vec<String> = field.described_by().iter().map(|e| e.text()).collect();
    assert_eq!(described, ["Hint"]);
    assert!(field.controls().is_empty());
    assert!(field.labelled_by().is_empty());
    assert!(screen.errors().is_empty());
}

#[test]
fn a_relation_to_a_hidden_node_is_kept_and_to_a_node_without_a_role_is_left_out() {
    let (screen, hidden) = mount(|| {
        let hidden = signal(true);
        let note = node_ref();
        let plain = node_ref();
        (
            hidden,
            column((
                node()
                    .role(Role::Group)
                    .name("Field")
                    .described_by(note)
                    .described_by(plain),
                text("Shown only to assistive technology")
                    .hidden(hidden)
                    .node_ref(note),
                node().node_ref(plain),
            )),
        )
    });
    let field = screen.get_by_role(Role::Group, "Field");
    assert_eq!(field.described_by().len(), 1);
    hidden.set(false);
    screen.run_until_idle();
    assert_eq!(field.described_by().len(), 1);
    assert!(screen.errors().is_empty());
}

/// Reverse dependencies survive a target being temporarily unexposed.
#[test]
fn relation_sources_follow_target_exposure_without_being_written() {
    let (screen, exposed) = mount(|| {
        let exposed = signal(true);
        let target = node_ref();
        (
            exposed,
            column((
                node().role(Role::Group).name("Source").described_by(target),
                node()
                    .node_ref(target)
                    .role(Role::Image)
                    .name(move || if exposed.get() { "Target" } else { "" }),
            )),
        )
    });
    let source = screen.get_by_role(Role::Group, "Source");
    assert_eq!(source.described_by().len(), 1);
    exposed.set(false);
    screen.run_until_idle();
    assert!(source.described_by().is_empty());
    exposed.set(true);
    screen.run_until_idle();
    assert_eq!(source.described_by().len(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
