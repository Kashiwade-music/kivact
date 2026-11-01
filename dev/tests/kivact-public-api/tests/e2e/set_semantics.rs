//! Role: Tests for set-related accessibility semantics.
//!
//! `level`, `set_size`, `pos_in_set` and `multiselectable` reach the host's
//! accessibility tree.

use crate::support::mount;
use kivact::prelude::*;
use kivact::primitive::NodeExt;

#[test]
fn level_set_size_position_and_multiselectable_reach_the_accessibility_tree() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                node()
                    .role(Role::List)
                    .name("Files")
                    .multiselectable(true)
                    .set_size(40_u32),
                node()
                    .role(Role::ListItem)
                    .name("Third")
                    .level(2_u32)
                    .set_size(40_u32)
                    .pos_in_set(3_u32),
                node().role(Role::ListItem).name("Plain"),
            )),
        )
    });
    let list = screen.get_by_role(Role::List, "Files");
    assert!(list.is_multiselectable());
    assert_eq!(list.set_size(), Some(40));
    assert_eq!(list.level(), None);
    assert_eq!(list.pos_in_set(), None);
    let item = screen.get_by_role(Role::ListItem, "Third");
    assert_eq!(item.level(), Some(2));
    assert_eq!(item.set_size(), Some(40));
    assert_eq!(item.pos_in_set(), Some(3));
    assert!(!item.is_multiselectable());
    let plain = screen.get_by_role(Role::ListItem, "Plain");
    assert_eq!(plain.level(), None);
    assert_eq!(plain.set_size(), None);
    assert_eq!(plain.pos_in_set(), None);
    assert!(!plain.is_multiselectable());
    assert!(screen.errors().is_empty());
}

#[test]
fn zero_is_not_published_for_a_level_or_a_position() {
    let (screen, ()) = mount(|| {
        (
            (),
            node()
                .role(Role::ListItem)
                .name("Item")
                .level(0_u32)
                .pos_in_set(0_u32),
        )
    });
    let item = screen.get_by_role(Role::ListItem, "Item");
    assert_eq!(item.level(), None);
    assert_eq!(item.pos_in_set(), None);
    assert!(screen.errors().is_empty());
}

#[test]
fn a_position_past_the_set_size_is_not_published_but_the_size_is() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                node()
                    .role(Role::ListItem)
                    .name("Past")
                    .set_size(3_u32)
                    .pos_in_set(4_u32),
                node()
                    .role(Role::ListItem)
                    .name("Last")
                    .set_size(3_u32)
                    .pos_in_set(3_u32),
            )),
        )
    });
    let past = screen.get_by_role(Role::ListItem, "Past");
    assert_eq!(past.set_size(), Some(3));
    assert_eq!(past.pos_in_set(), None);
    let last = screen.get_by_role(Role::ListItem, "Last");
    assert_eq!(last.set_size(), Some(3));
    assert_eq!(last.pos_in_set(), Some(3));
    assert!(screen.errors().is_empty());
}

#[test]
fn the_values_follow_a_signal() {
    let (screen, (level, size, position, multi)) = mount(|| {
        let level = signal(1_u32);
        let size = signal(5_u32);
        let position = signal(1_u32);
        let multi = signal(false);
        (
            (level, size, position, multi),
            node()
                .role(Role::TreeItem)
                .name("Node")
                .level(level)
                .set_size(size)
                .pos_in_set(position)
                .multiselectable(multi),
        )
    });
    let node = screen.get_by_role(Role::TreeItem, "Node");
    assert_eq!(
        (node.level(), node.set_size(), node.pos_in_set()),
        (Some(1), Some(5), Some(1))
    );
    assert!(!node.is_multiselectable());
    level.set(3);
    size.set(9);
    position.set(4);
    multi.set(true);
    screen.run_until_idle();
    assert_eq!(
        (node.level(), node.set_size(), node.pos_in_set()),
        (Some(3), Some(9), Some(4))
    );
    assert!(node.is_multiselectable());
    level.set(0);
    position.set(0);
    multi.set(false);
    screen.run_until_idle();
    assert_eq!(node.level(), None);
    assert_eq!(node.pos_in_set(), None);
    assert_eq!(node.set_size(), Some(9));
    assert!(!node.is_multiselectable());
    assert!(screen.errors().is_empty());
}
