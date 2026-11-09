//! Role: Tests that unmounting a long list gives its resources back (G-01, G-04).
//!
//! A `show` branch holds 3000 rows (a text and a button each). Mounting it
//! raises the live counts of the reactive graph, the owners, the frame loops
//! and the tree; closing it and settling must return all four to the values
//! they had before the list was drawn. Repeating the cycle checks that
//! nothing of the closed branches accumulates.

use crate::support::mount;
use kivact::prelude::*;

fn rows() -> impl IntoView {
    let ids = (0..3000u32).collect::<Vec<_>>();
    column(for_each(
        ids,
        |id| *id,
        |id| row((text(move || format!("row {}", id.get())), button("Edit"))),
    ))
}

#[test]
fn closing_three_thousand_rows_returns_the_live_counts_to_the_baseline() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        let view = column((show(open, rows), text("outside")));
        (open, view)
    });
    let before = screen.stats();
    for round in 0..3 {
        open.set(true);
        screen.run_until_idle();
        screen.get_by_text("row 0");
        open.set(false);
        screen.run_until_idle();
        let after = screen.stats();
        assert_eq!(
            after.live_reactive, before.live_reactive,
            "round {round}: live reactive nodes"
        );
        assert_eq!(
            after.live_owners, before.live_owners,
            "round {round}: live owners"
        );
        assert_eq!(
            after.frame_loops, before.frame_loops,
            "round {round}: live frame loops"
        );
        assert_eq!(
            after.tree_nodes, before.tree_nodes,
            "round {round}: tree nodes"
        );
    }
    screen.get_by_text("outside");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
