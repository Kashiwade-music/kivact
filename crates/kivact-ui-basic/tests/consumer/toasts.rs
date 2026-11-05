//! Role: Toast tests: variants, actions, timing and the stack that opens on hover.

use crate::support::assert_clean;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::Screen;
use kivact_ui_basic as basic;
use std::time::Duration;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::ColorName::Background)
            .color(basic::ColorName::Foreground)
            .padding(16)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

/// Mount a page with a toaster and a button that pushes `make()`.
fn toast_page(make: fn() -> kivact::interaction::Toast) -> (Screen, ()) {
    kivact::test::render_with_handle(crate::support::options(), move || {
        (
            (),
            page((
                kivact::ui::button("Show").on_press(move || {
                    kivact::interaction::toaster().push(make());
                }),
                kivact::ui::toast_region(kivact::interaction::toaster()).hold_all_on_hover(true),
            )),
        )
    })
}

#[test]
fn toast_shows_title_description_and_leaves_after_five_seconds() {
    let (screen, ()) = toast_page(|| {
        kivact::interaction::Toast::new("Event has been created")
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
