//! Role: Tests the structure of the standard dialog's frame.
//!
//! `kivact::ui::dialog` on real windows: the description sits under the
//! title in the header, the content's first control takes the initial
//! focus, and the header's close button is the last tab stop, drawn at the
//! top right. A hidden title leaves the heading out but keeps the name, a
//! popover opens on its placement and alignment, and a media view heads the
//! header, which its recipes know of.

use crate::support::mount;
use kivact::prelude::*;
use kivact::ui;

#[test]
fn the_content_takes_the_initial_focus_and_the_close_button_comes_last() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        let view = column((
            ui::button("Open").on_press(move || open.set(true)),
            ui::dialog("Settings", open)
                .description("Preferences of the account")
                .content(|| ui::text_input("Name", signal(String::new())))
                .footer(|| ui::button("Save")),
        ));
        (open, view)
    });
    screen.get_by_role(Role::Button, "Open").click();
    assert!(open.get_untracked());
    let name = screen.get_by_role(Role::TextBox, "Name");
    assert!(name.is_focused(), "the content's first control, not Close");
    let rect = |e: kivact::test::Element<'_>| e.rect().expect("laid out");
    let title = rect(screen.get_by_role(Role::Heading, "Settings"));
    let description = rect(screen.get_by_text("Preferences of the account"));
    assert!(description.y >= title.y + title.height, "under the title");
    assert!(
        (description.x - title.x).abs() < 0.5,
        "aligned with the title"
    );
    let close = rect(screen.get_by_role(Role::Button, "Close"));
    let panel = rect(screen.get_by_role(Role::Dialog, "Settings"));
    assert!(
        close.x + close.width > panel.x + panel.width * 0.75,
        "at the right"
    );
    assert!(close.y < title.y + title.height, "at the top");
    screen.press_key(Key::Tab);
    assert!(screen.get_by_role(Role::Button, "Save").is_focused());
    screen.press_key(Key::Tab);
    assert!(
        screen.get_by_role(Role::Button, "Close").is_focused(),
        "last"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_hidden_title_keeps_the_name_and_a_popover_opens_where_it_is_placed() {
    use kivact::style::Align;
    use kivact::ui::DialogKind;
    use kivact::view::{OverlayAlign, Placement};
    let (screen, hidden) = mount(|| {
        let hidden = signal(true);
        let view = column(
            ui::dialog("Pick a date", signal(true))
                .kind(DialogKind::Popover)
                .hide_title(hidden)
                .placement(Placement::Above)
                .align(OverlayAlign::End)
                .width(200)
                .content(|| text("Calendar"))
                .attach(ui::button("Choose a date")),
        )
        .style(style().padding(200).align_items(Align::Center));
        (hidden, view)
    });
    let rect = |e: kivact::test::Element<'_>| e.rect().expect("laid out");
    screen.get_by_role(Role::Dialog, "Pick a date");
    assert!(
        screen.query_by_role(Role::Heading, "Pick a date").is_none(),
        "the heading is left out"
    );
    let trigger = rect(screen.get_by_role(Role::Button, "Choose a date"));
    let panel = rect(screen.get_by_role(Role::Dialog, "Pick a date"));
    let body = rect(screen.get_by_text("Calendar"));
    assert!(
        panel.y + panel.height <= trigger.y + 0.5,
        "above its trigger: {panel:?} {trigger:?}"
    );
    assert!(
        (panel.x + panel.width - trigger.x - trigger.width).abs() < 0.5,
        "its end lined up with the trigger's: {panel:?} {trigger:?}"
    );
    assert!(body.y < panel.y + 24.0, "the body starts at the top");
    hidden.set(false);
    screen.run_until_idle();
    let heading = rect(screen.get_by_role(Role::Heading, "Pick a date"));
    let body = rect(screen.get_by_text("Calendar"));
    assert!(
        heading.y + heading.height <= body.y + 0.5,
        "the title is back"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_media_view_heads_the_header_and_its_recipes_know_of_it() {
    use crate::support::mount_with;
    use kivact::style::Direction;
    use kivact::test::RenderOptions;
    use kivact::ui::{DialogState, Look, UiRecipes};
    let dialog = || {
        ui::dialog("Allow accessory?", signal(true))
            .description("A USB accessory wants to connect.")
            .media(|| node().test_id("media").style(style().size(40, 40)))
    };
    let (screen, ()) = mount(move || ((), dialog()));
    let rect = |screen: &kivact::test::Screen, id: &str| {
        screen.get_by_test_id(id).rect().expect("laid out")
    };
    let title = |screen: &kivact::test::Screen| {
        screen
            .get_by_role(Role::Heading, "Allow accessory?")
            .rect()
            .expect("laid out")
    };
    let (media, heading) = (rect(&screen, "media"), title(&screen));
    assert!(
        media.y + media.height <= heading.y + 0.5,
        "the default header puts the media above the title"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    drop(screen);

    let theme = Theme::light();
    let recipes = UiRecipes::clone(&theme.get::<UiRecipes>()).dialog(|dialog| {
        dialog.header(|prev| {
            move |theme: &Theme, look: &Look<DialogState>| {
                let base = prev(theme, look);
                if look.media {
                    base.direction(Direction::Row)
                } else {
                    base
                }
            }
        })
    });
    let options = RenderOptions {
        theme: theme.with(recipes),
        ..RenderOptions::from_env()
    };
    let (screen, ()) = mount_with(options, move || ((), dialog()));
    let (media, heading) = (rect(&screen, "media"), title(&screen));
    assert!(
        media.x + media.width <= heading.x + 0.5 && (media.y - heading.y).abs() < 20.0,
        "a recipe that reads `media` puts it beside the title: {media:?} {heading:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
