//! Role: Verify the tooltip pointer interaction required by the APG pattern.
use crate::support::mount;
use kivact::prelude::*;
use kivact::view::Placement;
use std::time::Duration;

#[test]
fn tooltip_remains_open_while_pointer_is_over_tooltip() {
    let (screen, ()) = mount(|| {
        let trigger = node()
            .role(Role::Button)
            .name("Hint trigger")
            .test_id("hint-trigger")
            .style(style().size(140, 36))
            .child(text("Hover for help"));
        (
            (),
            column((
                kivact::ui::tooltip("Helpful hint")
                    .placement(Placement::Below)
                    .offset(0)
                    .content(|| {
                        node()
                            .style(style().size(200, 60))
                            .child(text("Helpful hint"))
                    })
                    .attach(trigger),
                node().style(style().height(240)),
            )),
        )
    });
    screen.get_by_test_id("hint-trigger").hover();
    screen.advance(Duration::from_millis(500));
    let hint = screen.get_by_role(Role::Tooltip, "Helpful hint");
    hint.hover();
    screen.advance(Duration::from_millis(600));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Helpful hint")
            .is_some(),
        "the hint remains available while its own surface is hovered"
    );
    screen.mouse_leave();
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Helpful hint")
            .is_none(),
        "leaving both the trigger and hint dismisses it"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
