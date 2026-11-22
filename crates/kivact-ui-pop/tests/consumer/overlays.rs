//! Role: Notifications through the public API.

use kivact::prelude::*;
use kivact::view::LiveRegion;
use std::time::Duration;

#[test]
fn notification_has_live_semantics_and_expires_on_the_host_clock() {
    let (screen, toaster) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let toaster = kivact::interaction::use_toaster();
            toaster.max_visible(4);
            (toaster, kivact::ui::toast_region(toaster))
        },
    );
    toaster.push(
        kivact::interaction::Toast::new("Saved")
            .tone(kivact::interaction::Tone::Success)
            .body("Request updated"),
    );
    toaster.push(
        kivact::interaction::Toast::new("Failed")
            .tone(kivact::interaction::Tone::Danger)
            .body("Request rejected"),
    );
    toaster.push(
        kivact::interaction::Toast::new("Heads up")
            .tone(kivact::interaction::Tone::Warning)
            .body("Quota almost used"),
    );
    screen.run_until_idle();
    assert!(screen.accessibility_tree().contains("Saved"));
    let saved = screen.get_by_role(Role::Status, "Saved");
    let groups = screen.all_by_role(Role::Group, "");
    assert_eq!(groups[0].live_region(), LiveRegion::Polite);
    assert_eq!(groups[1].live_region(), LiveRegion::Assertive);
    assert!(saved.exists());
    screen.get_by_role(Role::Alert, "Failed");
    screen.get_by_role(Role::Alert, "Heads up");
    assert_eq!(screen.all_by_role(Role::Button, "Close").len(), 3);
    screen.advance(Duration::from_millis(4999));
    assert!(screen.query_by_text("Saved").is_some());
    screen.advance(Duration::from_millis(1));
    assert!(screen.query_by_text("Saved").is_none());
    assert!(screen.query_by_text("Failed").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
