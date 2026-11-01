//! Role: Overlay state tests: command groups, toast close, popover options, scrim edges.

use super::{assert_clean, distance, pixel};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::ui::{Command, DialogKind, button, command_palette};
use kivact_ui_basic as basic;

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

fn entries() -> Vec<Command<&'static str>> {
    vec![
        Command::new("calendar", "Calendar")
            .group("Suggestions")
            .icon("calendar"),
        Command::new("search", "Search Emoji")
            .group("Suggestions")
            .icon("smile"),
        Command::new("profile", "Profile")
            .group("Settings")
            .icon("user")
            .hint("Ctrl P"),
        Command::new("help", "Help"),
    ]
}

#[test]
fn command_follows_changed_commands() {
    let (screen, items) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(entries());
        (
            items,
            page(command_palette(
                signal(true),
                items,
                |_key: &'static str| {},
            )),
        )
    });
    screen.get_by_text("Ctrl P");
    let text_x = |screen: &kivact::test::Screen, what: &str| {
        screen.get_by_text(what).rect().expect("laid out").x
    };
    let with_icon = text_x(&screen, "Calendar");
    items.update(|all| {
        all[2] = Command::new("profile", "Profile")
            .group("Settings")
            .icon("user")
            .hint("Ctrl K");
        all[0] = Command::new("calendar", "Calendar").group("Suggestions");
    });
    screen.run_until_idle();
    screen.get_by_text("Ctrl K");
    assert!(screen.query_by_text("Ctrl P").is_none());
    assert!(
        text_x(&screen, "Calendar") < with_icon - 16.0,
        "the row that lost its icon redraws without it"
    );
    assert_clean(&screen);
}

#[test]
fn the_toast_close_button_is_a_bare_icon_with_a_wide_hit_area() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page((
                button("Show").on_press(|| {
                    kivact::ui::toaster().push(kivact::ui::Toast::new("Saved"));
                }),
                kivact::ui::toast_region(kivact::ui::toaster()).hold_all_on_hover(true),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Show").click();
    let close = screen.get_by_role(Role::Button, "Close");
    let rect = close.rect().expect("laid out");
    close.hover();
    screen.run_until_idle();
    // A ghost button would fill its square with the muted color on hover.
    let corner = pixel(&screen, rect.x + 2.0, rect.y + 2.0);
    assert!(
        distance(corner, 0xffffff) <= 2,
        "no button chrome behind the icon: {corner:06x}"
    );
    close.click();
    assert!(screen.query_by_role(Role::Status, "Saved").is_none());
    assert_clean(&screen);
}

#[test]
fn a_popover_without_a_visible_title_can_be_sized_to_its_content() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                kivact::ui::dialog("Pick a thing", signal(true))
                    .kind(kivact::ui::DialogKind::Popover)
                    .hide_title(true)
                    .width(kivact::style::Length::Auto)
                    .content(|| text("Narrow").test_id("body"))
                    .attach(button("Open")),
            ),
        )
    });
    let dialog = screen.get_by_role(Role::Dialog, "Pick a thing");
    let width = dialog.rect().expect("laid out").width;
    assert!(width < 100.0, "w-auto, not w-72: {width}");
    let body = screen.get_by_test_id("body").rect().expect("laid out");
    let panel = dialog.rect().expect("laid out");
    assert!(
        (body.x - panel.x - 10.0).abs() <= 1.0,
        "p-2.5 around the body"
    );
    assert!(
        (body.y - panel.y - 10.0).abs() <= 1.0,
        "no heading above it"
    );
    assert_clean(&screen);
}

#[test]
fn a_modal_scrim_blurs_the_page_without_darkening_the_window_edges() {
    // The dark page: a frame is read back composited over white, so a blur
    // that read the transparent outside would lighten its edges here (a
    // window's surface shows them black).
    let theme = basic::theme(kivact::style::ThemeMode::Dark);
    let (screen, ()) = kivact::test::render_with_handle(
        kivact::test::RenderOptions {
            theme,
            ..crate::options()
        },
        || {
            let open = signal(false);
            (
                (),
                page((
                    button("Delete").on_press(move || open.set(true)),
                    kivact::ui::dialog("Are you absolutely sure?", open)
                        .kind(DialogKind::AlertModal)
                        .closable(false)
                        .footer(move || button("Cancel").on_press(move || open.set(false))),
                )),
            )
        },
    );
    screen.get_by_role(Role::Button, "Delete").click();
    screen.get_by_role(Role::AlertDialog, "Are you absolutely sure?");
    let shot = screen.screenshot();
    let (right, bottom) = (shot.width - 1, shot.height - 1);
    let (mid_x, mid_y) = (shot.width / 2, shot.height / 2);
    let at = |x: u32, y: u32| shot.pixel(x, y).expect("inside the frame");
    // The page under the black 10 % scrim, blurred: the same at the edges
    // and corners as a little inside them (the blur reads the window
    // mirrored at its edges, not the transparent outside).
    let inside = at(24, mid_y);
    assert!(
        distance(inside, 0x090909) <= 6,
        "the scrimmed page: {inside:06x}"
    );
    for (x, y) in [
        (0, 0),
        (right, 0),
        (0, bottom),
        (right, bottom),
        (0, mid_y),
        (right, mid_y),
        (mid_x, 0),
        (mid_x, bottom),
    ] {
        let edge = at(x, y);
        assert!(
            distance(edge, inside) <= 4,
            "({x}, {y}) is {edge:06x}, not the page inside ({inside:06x})"
        );
    }
    assert_clean(&screen);
}
