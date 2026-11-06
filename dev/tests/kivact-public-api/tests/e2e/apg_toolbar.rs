//! Role: Cover APG Toolbar axis, boundaries, disabled controls and direction.
//! Audit: https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/
//! Native roving supplies optional wrapping, Home/End and last-focus reentry.
//! Perpendicular arrows remain available to child controls. Disabled controls
//! are skipped unless the caller explicitly opts into disabled focusability.
use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::toolbar;
use kivact::view::{Orientation, ReadingDirection, provide_direction};

#[test]
fn vertical_toolbar_uses_its_axis_home_end_and_skips_disabled_controls() {
    let (screen, hits) = mount(|| {
        let hits = signal(0);
        (
            hits,
            toolbar("Tools").orientation(Orientation::Vertical).start((
                button("First").on_press(move || hits.update(|hits| *hits += 1)),
                button("Unavailable").disabled(true),
                button("Middle"),
                button("Last"),
            )),
        )
    });
    assert_eq!(
        screen.get_by_role(Role::Toolbar, "Tools").orientation(),
        Orientation::Vertical
    );
    screen.get_by_role(Role::Button, "First").focus();
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Button, "Middle").is_focused());
    screen.press_key(Key::End);
    assert!(screen.get_by_role(Role::Button, "Last").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Button, "First").is_focused());
    screen.press_key(Key::ArrowUp);
    assert!(screen.get_by_role(Role::Button, "Last").is_focused());
    screen.press_key(Key::Home);
    assert!(screen.get_by_role(Role::Button, "First").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::Button, "First").is_focused());
    assert_eq!(hits.get_untracked(), 0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn rtl_toolbar_navigation_follows_the_visual_direction() {
    let (screen, ()) = mount(|| {
        (
            (),
            provide_direction(ReadingDirection::Rtl, || {
                toolbar("RTL tools").start((button("First"), button("Second"), button("Third")))
            }),
        )
    });
    screen.get_by_role(Role::Button, "First").focus();
    screen.press_key(Key::ArrowLeft);
    assert!(screen.get_by_role(Role::Button, "Second").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::Button, "First").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
