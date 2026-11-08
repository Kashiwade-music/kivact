//! Role: Presence modes of a replaced branch: Wait, PopLayout and Sync.

use std::time::Duration;

use crate::support::mount;
use kivact::motion::{Easing, Repeat, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::view::{Node, PresenceMode};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The old arm leaves over 200 ms (its exit is a 200 ms tween of the opacity).
fn leaving_arm(test_id: &'static str) -> Node {
    node().test_id(test_id).style(
        style()
            .size(50, 30)
            .exit(style().opacity(0.0))
            .transition(TransitionProp::All, Transition::tween(200, Easing::Linear)),
    )
}

/// The old arm's exit is two alternating 100 ms iterations, so its run ends
/// on the value it started from, not on the exit target.
fn held_exit_arm(test_id: &'static str) -> Node {
    node().test_id(test_id).style(
        style().size(50, 30).exit(style().opacity(0.0)).transition(
            TransitionProp::All,
            Transition::tween(100, Easing::Linear)
                .repeat(Repeat::Times(2))
                .alternate(),
        ),
    )
}

#[test]
fn wait_mounts_after_an_exit_that_ends_held() {
    let (screen, shown) = mount(|| {
        let shown = signal(true);
        let content = show(shown, || held_exit_arm("old"))
            .fallback(|| node().test_id("new").style(style().size(50, 30)))
            .presence(PresenceMode::Wait);
        (shown, content)
    });
    assert!(screen.query_by_test_id("old").is_some());
    shown.set(false);
    screen.run_until_idle();
    screen.advance(ms(150));
    assert!(
        screen.query_by_test_id("new").is_none(),
        "Wait: the new arm waits for the exit's second iteration"
    );
    screen.advance(ms(250));
    assert!(
        screen.query_by_test_id("old").is_none(),
        "an exit that ends held still completes"
    );
    assert!(
        screen.query_by_test_id("new").is_some(),
        "the new arm mounts once the held exit completes"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn wait_mounts_after_exit() {
    let (screen, shown) = mount(|| {
        let shown = signal(true);
        let content = show(shown, || leaving_arm("old"))
            .fallback(|| node().test_id("new").style(style().size(50, 30)))
            .presence(PresenceMode::Wait);
        (shown, content)
    });
    assert!(screen.query_by_test_id("old").is_some());
    shown.set(false);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert!(
        screen.query_by_test_id("new").is_none(),
        "Wait: the new arm waits for the exit"
    );
    screen.advance(ms(150));
    assert!(screen.query_by_test_id("old").is_none());
    assert!(
        screen.query_by_test_id("new").is_some(),
        "the new arm mounts after the 200 ms exit"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pop_layout_reflows_at_once() {
    let (screen, shown) = mount(|| {
        let shown = signal(true);
        let panel = row((
            show(shown, || leaving_arm("leaving")).presence(PresenceMode::PopLayout),
            node().test_id("sibling").style(style().size(18, 30)),
        ))
        .style(style().gap(4).padding(10));
        (shown, panel)
    });
    let before = screen.get_by_test_id("sibling").rect().unwrap().x;
    shown.set(false);
    screen.run_until_idle();
    let after = screen.get_by_test_id("sibling").rect().unwrap().x;
    assert!(
        after < before - 40.0,
        "PopLayout: the sibling moves at once ({before} -> {after})"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn sync_keeps_both() {
    let (screen, shown) = mount(|| {
        let shown = signal(true);
        let content = show(shown, || leaving_arm("old"))
            .fallback(|| node().test_id("new").style(style().size(50, 30)))
            .presence(PresenceMode::Sync);
        (shown, content)
    });
    shown.set(false);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert!(
        screen.query_by_test_id("new").is_some(),
        "Sync: the new arm enters at once"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
