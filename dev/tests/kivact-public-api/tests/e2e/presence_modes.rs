//! Role: Presence modes of a replaced branch: Wait, PopLayout and Sync.

use std::time::Duration;

use super::paint::{BLUE, RED, pixel};
use crate::support::mount;
use kivact::motion::{Easing, Transition};
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
            .background(rgb(RED))
            .exit(style().opacity(0.0))
            .transition(TransitionProp::All, Transition::tween(200, Easing::Linear)),
    )
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
            .fallback(|| {
                node()
                    .test_id("new")
                    .style(style().size(20, 30).background(rgb(BLUE)))
            })
            .presence(PresenceMode::Sync);
        (shown, content)
    });
    assert!(screen.query_by_test_id("old").is_some());
    shown.set(false);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert!(
        screen.query_by_test_id("new").is_some(),
        "Sync: the new arm enters at once"
    );
    // An exiting arm is not found by its test id, so the old arm is read from
    // the paint. The new arm covers x < 20; at x = 40 only the old arm is
    // there, half faded at 100 ms: red over the background, so r - g >= 100
    // on a white or a black background.
    let shot = screen.screenshot();
    let p = pixel(&shot, 40, 15);
    let (r, g) = ((p >> 16) & 255, (p >> 8) & 255);
    assert!(
        r >= g + 100,
        "Sync: the old arm is still drawn at x 40: {p:06x}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
