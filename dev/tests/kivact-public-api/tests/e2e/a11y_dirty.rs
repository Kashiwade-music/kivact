//! Role: Sparse accessibility publication and real shell rejection recovery.
use crate::support::mount;
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::style::Position;

#[test]
fn rejected_numeric_snapshot_keeps_accepted_labels_until_full_retry() {
    let (screen, (number, label)) = mount(|| {
        let number = signal(1.0);
        let label = signal("before");
        (
            (number, label),
            column((
                node()
                    .role(Role::Slider)
                    .name("Range")
                    .test_id("range")
                    .value_min(0.0)
                    .value_max(10.0)
                    .value_now(number),
                text(move || label.get()).test_id("label"),
            )),
        )
    });
    let range = screen.get_by_test_id("range");
    let label_node = screen.get_by_test_id("label");
    let before = screen.stats();
    number.set(f64::NAN);
    label.set("after");
    screen.run_until_idle();
    assert_eq!(range.value_range(), (Some(0.0), Some(10.0), Some(1.0)));
    assert_eq!(
        label_node.name(),
        "before",
        "shell rejection is transactional"
    );
    assert!(screen.stats().a11y_published > before.a11y_published);
    number.set(2.0);
    screen.run_until_idle();
    assert_eq!(range.value_range(), (Some(0.0), Some(10.0), Some(2.0)));
    assert_eq!(
        label_node.name(),
        "after",
        "retry preserves all rejected dirt"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn fixed_geometry_reorder_and_removal_update_accepted_child_order() {
    let (screen, rows) = mount(|| {
        let rows = signal(vec![1u32, 2, 3]);
        (
            rows,
            node()
                .role(Role::Group)
                .name("Static parent")
                .style(style().size(200, 80))
                .child(for_each(
                    rows,
                    |id| *id,
                    |id, _| {
                        node()
                            .role(Role::Group)
                            .name(format!("child-{id}"))
                            .test_id(format!("child-{id}"))
                            .style(
                                style()
                                    .size(80, 30)
                                    .position(Position::Absolute)
                                    .left(0)
                                    .top(0),
                            )
                    },
                )),
        )
    });
    let original = screen.get_by_test_id("child-1").accessibility_bounds();
    rows.set(vec![3, 1, 2]);
    screen.run_until_idle();
    let tree = screen.accessibility_tree();
    assert!(
        tree.find("child-3").unwrap() < tree.find("child-1").unwrap(),
        "{tree}"
    );
    assert!(
        tree.find("child-1").unwrap() < tree.find("child-2").unwrap(),
        "{tree}"
    );
    assert_eq!(
        screen.get_by_test_id("child-1").accessibility_bounds(),
        original
    );
    rows.set(vec![3, 2]);
    screen.run_until_idle();
    let tree = screen.accessibility_tree();
    assert!(!tree.contains("child-1"), "{tree}");
    assert!(
        tree.find("child-3").unwrap() < tree.find("child-2").unwrap(),
        "{tree}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
