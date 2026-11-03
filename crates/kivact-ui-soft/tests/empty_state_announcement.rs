//! Role: Verify Empty State speaks only after initial render and while eligible.
use kivact::control::{NodeBuilder, column};
use kivact::primitive::NodeExt;
use kivact::signal;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, WindowMode, render_with_handle};
use kivact::view::{LiveRegion, Role, show};
use kivact_ui_soft::empty_state;

fn options() -> RenderOptions {
    RenderOptions {
        theme: kivact_ui_soft::theme(ThemeMode::Light),
        window: WindowMode::Headless,
        ..RenderOptions::default()
    }
}

#[test]
fn initial_empty_state_is_silent_but_late_mount_and_text_changes_announce() {
    let (screen, (initial, late)) = render_with_handle(options(), || {
        let initial = signal("Initial");
        let late = signal(false);
        (
            (initial, late),
            column((
                empty_state(move || initial.get()).description("Initial help"),
                show(late, || empty_state("No results").description("Try again")),
            )),
        )
    });
    assert!(
        screen
            .query_by_role(Role::Text, "Initial. Initial help")
            .is_none()
    );

    initial.set("Updated");
    late.set(true);
    screen.run_until_idle();
    let update = screen.get_by_role(Role::Text, "Updated. Initial help");
    let appeared = screen.get_by_role(Role::Text, "No results. Try again");
    assert!(update.exists() && appeared.exists());

    initial.set(" Updated ");
    screen.run_until_idle();
    // The normalized text was the same, so the announcer did not replace
    // the existing announcement with a new text node.
    assert!(update.exists());
    assert!(appeared.exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn hidden_empty_state_rearms_and_live_ancestors_handle_their_own_text() {
    let (screen, (hidden, live, title)) = render_with_handle(options(), || {
        let hidden = signal(true);
        let live = signal(LiveRegion::Polite);
        let title = signal("No results");
        (
            (hidden, live, title),
            column((
                column(empty_state(move || title.get()).description("Try again")).hidden(hidden),
                column(empty_state("Embedded").description("Own region")).live_region(live),
                column(empty_state("Status").description("Own role")).role(Role::Status),
            )),
        )
    });
    title.set("Still hidden");
    screen.run_until_idle();
    assert!(
        screen
            .query_by_role(Role::Text, "Still hidden. Try again")
            .is_none()
    );
    assert!(
        screen
            .query_by_role(Role::Text, "Embedded. Own region")
            .is_none()
    );
    assert!(
        screen
            .query_by_role(Role::Text, "Status. Own role")
            .is_none()
    );

    hidden.set(false);
    screen.run_until_idle();
    let first = screen.get_by_role(Role::Text, "Still hidden. Try again");
    hidden.set(true);
    screen.run_until_idle();
    assert!(!first.exists());
    hidden.set(false);
    live.set(LiveRegion::Off);
    screen.run_until_idle();
    assert!(
        screen
            .query_by_role(Role::Text, "Still hidden. Try again")
            .is_some()
    );
    assert!(
        screen
            .query_by_role(Role::Text, "Embedded. Own region")
            .is_some()
    );
    assert!(
        screen
            .query_by_role(Role::Text, "Status. Own role")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
