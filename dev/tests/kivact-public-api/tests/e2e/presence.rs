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
