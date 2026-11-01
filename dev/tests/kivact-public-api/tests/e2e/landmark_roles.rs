//! Role: Tests that row groups and navigation landmarks reach the tree.
//!
//! `Role::RowGroup` on the sections of a table and `Role::Navigation` on
//! a named set of links are published to assistive technology like the
//! other roles: found by role and name, with their rows and links inside.

use crate::support::mount;
use kivact::prelude::*;

#[test]
fn table_sections_are_row_groups_and_a_named_link_set_is_a_navigation() {
    let (screen, ()) = mount(|| {
        let cell = |label: &'static str| node().role(Role::Cell).child(text(label));
        let view = column((
            node().role(Role::Navigation).name("breadcrumb").child(row((
                node().role(Role::Link).name("Home").child(text("Home")),
                node().role(Role::Link).name("Docs").child(text("Docs")),
            ))),
            node().role(Role::Table).name("Invoices").child((
                node()
                    .role(Role::RowGroup)
                    .child(node().role(Role::Row).child(cell("Invoice"))),
                node().role(Role::RowGroup).child((
                    node().role(Role::Row).child(cell("INV001")),
                    node().role(Role::Row).child(cell("INV002")),
                )),
            )),
        ));
        ((), view)
    });
    let nav = screen.get_by_role(Role::Navigation, "breadcrumb");
    let links = screen.within(nav);
    links.get_by_role(Role::Link, "Home");
    links.get_by_role(Role::Link, "Docs");
    let groups = screen.all_by_role(Role::RowGroup, "");
    assert_eq!(groups.len(), 2);
    let rows = |group| screen.within(group).all_by_role(Role::Row, "").len();
    let mut groups = groups.into_iter();
    assert_eq!(groups.next().map(rows), Some(1));
    assert_eq!(groups.next().map(rows), Some(2));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
