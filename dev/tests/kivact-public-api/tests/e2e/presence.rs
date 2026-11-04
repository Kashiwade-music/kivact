//! Role: Verify reactive node presence through entrance and removal.

use std::time::Duration;

use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::view::Presence;

use crate::support::mount;

#[test]
fn presence_reports_entering_present_leaving() {
    let (screen, (shown, presence)) = mount(|| {
        let shown = signal(true);
        let target = node_ref();
        let presence = target.presence();
        let panel = show(shown, move || {
            node().node_ref(target).style(
                style()
                    .size(40, 30)
                    .enter(style().opacity(0.0))
                    .exit(style().opacity(0.0))
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(200, Easing::Linear),
                    ),
            )
        });
        ((shown, presence), panel)
    });
    assert_eq!(presence.get(), Presence::Entering);
    screen.advance(Duration::from_millis(250));
    assert_eq!(presence.get(), Presence::Present);
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(presence.get(), Presence::Leaving);
    screen.advance(Duration::from_millis(250));
    assert_eq!(presence.get(), Presence::Leaving);
    shown.set(true);
    screen.run_until_idle();
    assert_eq!(presence.get(), Presence::Entering);
    screen.advance(Duration::from_millis(250));
    assert_eq!(presence.get(), Presence::Present);
}

#[test]
fn exit_size_collapses_the_node_before_it_is_removed() {
    use kivact::view::Orientation;

    let (screen, (shown, presence)) = mount(|| {
        let shown = signal(true);
        let target = node_ref();
        let presence = target.presence();
        let panel = column((
            show(shown, move || {
                node().node_ref(target).test_id("collapsing").style(
                    style()
                        .size(80, 100)
                        .enter_size(Orientation::Vertical)
                        .exit_size(Orientation::Vertical)
                        .transition(TransitionProp::Size, Transition::tween(200, Easing::Linear)),
                )
            }),
            node().test_id("after").style(style().size(20, 20)),
        ))
        .test_id("size-parent");
        ((shown, presence), panel)
    });
    assert_eq!(presence.get(), Presence::Entering);
    screen.advance(Duration::from_millis(250));
    assert_eq!(presence.get(), Presence::Present);
    let panel = screen.get_by_test_id("collapsing");
    let original = panel.rect().expect("laid out").height;
    let parent_height = screen
        .get_by_test_id("size-parent")
        .rect()
        .expect("laid out")
        .height;
    let before = screen.get_by_test_id("after").rect().expect("laid out").y;
    shown.set(false);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    let height = panel.rect().expect("retained during exit").height;
    assert!(
        height > 0.0 && height < original,
        "intermediate height: {height}"
    );
    let during = screen.get_by_test_id("after").rect().expect("laid out").y;
    assert!(during < before && during > before - original);
    assert!(
        screen
            .get_by_test_id("size-parent")
            .rect()
            .expect("laid out")
            .height
            < parent_height
    );
    screen.advance(Duration::from_millis(150));
    assert!(panel.rect().is_none(), "removed when size motion finishes");
}
