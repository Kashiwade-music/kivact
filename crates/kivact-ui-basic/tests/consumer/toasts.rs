//! Role: Toast tests: variants, actions, timing and the stack that opens on hover.

use super::assert_clean;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::Screen;
use kivact::ui::toaster;
use kivact::view::Key;
use kivact_ui_basic as basic;
use std::time::Duration;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(16)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

/// Mount a page with a toaster and a button that pushes `make()`.
fn toast_page(make: fn() -> kivact::ui::Toast) -> (Screen, ()) {
    kivact::test::render_with_handle(crate::options(), move || {
        (
            (),
            page((
                kivact::ui::button("Show").on_press(move || {
                    kivact::ui::toaster().push(make());
                }),
                kivact::ui::toast_region(kivact::ui::toaster()).hold_all_on_hover(true),
            )),
        )
    })
}

#[test]
fn toast_shows_title_description_and_leaves_after_five_seconds() {
    let (screen, ()) = toast_page(|| {
        kivact::ui::Toast::new("Event has been created")
            .body("Sunday, December 03, 2023 at 9:00 AM")
    });
    screen.get_by_role(Role::Button, "Show").click();
    let toast = screen.get_by_role(Role::Status, "Event has been created");
    screen.get_by_text("Sunday, December 03, 2023 at 9:00 AM");
    let rect = toast.rect().expect("laid out");
    assert_eq!(rect.width, 384.0);
    assert_eq!(
        rect.x + rect.width,
        800.0 - 16.0,
        "right-aligned with a 16 px margin"
    );
    assert_eq!(rect.y + rect.height, 600.0 - 16.0, "16 px above the bottom");
    assert_clean(&screen);
    screen.assert_layout();
    screen.advance(Duration::from_millis(4900));
    screen.get_by_role(Role::Status, "Event has been created");
    screen.advance(Duration::from_millis(200));
    assert!(
        screen
            .query_by_role(Role::Status, "Event has been created")
            .is_none(),
        "gone after the default 5 seconds"
    );
    assert_clean(&screen);
}

/// Mount `view` under an app whose toaster shows up to eight toasts.
fn toasting<V: IntoView>(view: impl FnOnce() -> V + 'static) -> (Screen, ()) {
    kivact::test::render_with_handle(crate::options(), move || {
        toaster().max_visible(8);
        (
            (),
            page((
                view(),
                kivact::ui::toast_region(kivact::ui::toaster()).hold_all_on_hover(true),
            )),
        )
    })
}

#[test]
fn toast_variants_are_announced_by_urgency_and_the_close_button_dismisses() {
    let (screen, ()) = toasting(|| {
        kivact::ui::button("Show").on_press(|| {
            kivact::ui::toaster().push(kivact::ui::Toast::new("Plain"));
            kivact::ui::toaster()
                .push(kivact::ui::Toast::new("Saved").tone(kivact::ui::Tone::Success));
            kivact::ui::toaster().push(kivact::ui::Toast::new("FYI").tone(kivact::ui::Tone::Info));
            kivact::ui::toaster()
                .push(kivact::ui::Toast::new("Careful").tone(kivact::ui::Tone::Warning));
            kivact::ui::toaster()
                .push(kivact::ui::Toast::new("Failed").tone(kivact::ui::Tone::Danger));
            kivact::ui::toaster().push(
                kivact::ui::Toast::new("Uploading")
                    .icon(|| basic::spinner().role(Role::Generic))
                    .duration(None),
            );
        })
    });
    screen.get_by_role(Role::Button, "Show").click();
    for title in ["Plain", "Saved", "FYI", "Uploading"] {
        screen.get_by_role(Role::Status, title);
    }
    for title in ["Careful", "Failed"] {
        screen.get_by_role(Role::Alert, title);
    }
    assert!(
        screen.query_by_role(Role::Status, "Loading").is_none(),
        "the spinner is no second status inside the live region"
    );
    assert_clean(&screen);
    screen.assert_layout();

    // The stack is closed until the pointer is over it: open it on the newest.
    screen.get_by_role(Role::Status, "Uploading").hover();
    let saved = screen.get_by_role(Role::Status, "Saved");
    screen
        .within(saved)
        .get_by_role(Role::Button, "Close")
        .click();
    assert!(screen.query_by_role(Role::Status, "Saved").is_none());
    screen.get_by_role(Role::Status, "Plain");

    // Escape inside a toast dismisses it.
    let fyi = screen.get_by_role(Role::Status, "FYI");
    screen
        .within(fyi)
        .get_by_role(Role::Button, "Close")
        .focus();
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Status, "FYI").is_none());

    // The ordinary ones leave after five seconds once the stack is closed
    // again; the loading one stays.
    screen.mouse_move(20.0, 20.0);
    screen.get_by_role(Role::Button, "Show").focus();
    screen.advance(Duration::from_secs(6));
    assert!(screen.query_by_role(Role::Status, "Plain").is_none());
    assert!(screen.query_by_role(Role::Alert, "Failed").is_none());
    screen.get_by_role(Role::Status, "Uploading");
    screen.advance(Duration::from_secs(60));
    screen.get_by_role(Role::Status, "Uploading");
    assert_clean(&screen);
}

#[test]
fn toast_action_runs_its_callback_and_dismisses_the_toast() {
    let (screen, undone) = kivact::test::render_with_handle(crate::options(), || {
        let undone = signal(0);
        (
            undone,
            page((
                kivact::ui::button("Show").on_press(move || {
                    kivact::ui::toaster().push(
                        kivact::ui::Toast::new("Event created")
                            .action("Undo", move || undone.update(|n| *n += 1)),
                    );
                }),
                kivact::ui::toast_region(kivact::ui::toaster()).hold_all_on_hover(true),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Show").click();
    screen.get_by_role(Role::Status, "Event created");
    screen.get_by_role(Role::Button, "Undo").click();
    assert_eq!(undone.get_untracked(), 1);
    assert!(
        screen
            .query_by_role(Role::Status, "Event created")
            .is_none()
    );
    assert_clean(&screen);
}

#[test]
fn toast_waits_while_hovered_and_honors_a_custom_duration() {
    let (screen, ()) = toasting(|| {
        kivact::ui::button("Show").on_press(|| {
            kivact::ui::toaster()
                .push(kivact::ui::Toast::new("Short").duration(Some(Duration::from_secs(1))));
            kivact::ui::toaster().push(kivact::ui::Toast::new("Sticky").duration(None));
        })
    });
    screen.get_by_role(Role::Button, "Show").click();
    // The stack opens on the newest toast and holds the time of both.
    screen.get_by_role(Role::Status, "Sticky").hover();
    let short = screen.get_by_role(Role::Status, "Short");
    short.hover();
    screen.advance(Duration::from_secs(10));
    screen.get_by_role(Role::Status, "Short");
    screen.mouse_move(700.0, 100.0);
    screen.advance(Duration::from_millis(900));
    screen.get_by_role(Role::Status, "Short");
    screen.advance(Duration::from_millis(200));
    assert!(screen.query_by_role(Role::Status, "Short").is_none());
    screen.advance(Duration::from_secs(120));
    screen.get_by_role(Role::Status, "Sticky");
}

#[test]
fn toasts_stack_behind_the_newest_and_open_into_a_list_while_hovered_or_focused() {
    let (screen, ()) = toasting(|| {
        kivact::ui::button("Show").on_press(|| {
            for title in ["One", "Two", "Three", "Four"] {
                kivact::ui::toaster().push(kivact::ui::Toast::new(title).body("Body"));
            }
        })
    });
    screen.get_by_role(Role::Button, "Show").click();
    screen.mouse_move(20.0, 20.0);
    let rect = |title: &str| {
        screen
            .get_by_role(Role::Status, title)
            .rect()
            .expect("laid out")
    };

    // Closed: every toast ends where the newest does, moved up by 14 px per
    // place behind it and shrunk by 5 % per place toward its bottom edge.
    let front = rect("Four");
    assert_eq!(front.width, 384.0);
    assert_eq!(front.y + front.height, 600.0 - 16.0);
    for (title, behind) in [("Three", 1.0), ("Two", 2.0)] {
        let back = rect(title);
        assert!(
            (back.width - 384.0 * (1.0 - 0.05 * behind)).abs() < 0.5,
            "{title}: {back:?}"
        );
        assert!(
            (back.y + back.height - (front.y + front.height - 14.0 * behind)).abs() < 0.5,
            "{title} sits {behind} gaps up: {back:?}"
        );
        assert!(
            back.y < front.y,
            "{title} peeks out above the newest: {back:?}"
        );
    }
    assert!(rect("Three").y > rect("Two").y);
    assert_clean(&screen);
    screen.assert_layout();

    // Hovered: a list of full-size cards with a 14 px gap, newest in place.
    screen.get_by_role(Role::Status, "Four").hover();
    assert_eq!(rect("Four"), front, "the newest stays where it was");
    for (older, newer) in [("Three", "Four"), ("Two", "Three"), ("One", "Two")] {
        let (older, newer) = (rect(older), rect(newer));
        assert_eq!(older.width, 384.0);
        assert_eq!(newer.y - (older.y + older.height), 14.0, "gap-3.5");
    }
    assert_clean(&screen);
    screen.assert_layout();

    // The time of every toast waits while the stack is open.
    screen.advance(Duration::from_secs(30));
    for title in ["One", "Two", "Three", "Four"] {
        screen.get_by_role(Role::Status, title);
    }

    // Closed again once the pointer is away; the five seconds then run out.
    screen.mouse_move(20.0, 20.0);
    assert!(rect("Three").width < 384.0, "closed again");
    screen.advance(Duration::from_secs(6));
    assert!(screen.query_by_role(Role::Status, "Four").is_none());
    assert_clean(&screen);
}

#[test]
fn the_focus_inside_a_toast_opens_the_stack() {
    let (screen, ()) = toasting(|| {
        kivact::ui::button("Show").on_press(|| {
            kivact::ui::toaster().push(kivact::ui::Toast::new("Older").duration(None));
            kivact::ui::toaster().push(kivact::ui::Toast::new("Newer").duration(None));
        })
    });
    screen.get_by_role(Role::Button, "Show").click();
    screen.mouse_move(20.0, 20.0);
    let width = |title: &str| {
        screen
            .get_by_role(Role::Status, title)
            .rect()
            .expect("laid out")
            .width
    };
    assert!(width("Older") < 384.0, "closed while nothing is in it");
    let newer = screen.get_by_role(Role::Status, "Newer");
    screen
        .within(newer)
        .get_by_role(Role::Button, "Close")
        .focus();
    assert_eq!(
        width("Older"),
        384.0,
        "open while the focus is in the stack"
    );
    screen.get_by_role(Role::Button, "Show").focus();
    assert!(width("Older") < 384.0, "closed when the focus leaves");
    assert_clean(&screen);
}
