//! Role: Verify APG Alert semantics without unsolicited focus changes.
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions, render_with_handle};
use kivact::ui;
use kivact::view::{LiveRegion, Role};

#[test]
fn apg_alert_is_assertive_persistent_and_does_not_take_focus() {
    let (screen, visible) =
        render_with_handle(RenderOptions::default().mirror(MirrorMode::Off), || {
            let visible = signal(false);
            (
                visible,
                column((
                    ui::button("Continue"),
                    show(visible, || {
                        ui::alert("Session expiring").description("Save your work")
                    }),
                )),
            )
        });
    let control = screen.get_by_role(Role::Button, "Continue");
    control.focus();
    visible.set(true);
    screen.run_until_idle();
    let alert = screen.get_by_role(Role::Alert, "Session expiring");
    assert_eq!(alert.live_region(), LiveRegion::Assertive);
    assert!(control.is_focused());
    screen.run_until_idle();
    assert!(alert.exists());
    assert!(control.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
