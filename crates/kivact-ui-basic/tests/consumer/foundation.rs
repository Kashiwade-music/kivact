//! Role: Foundation tests: theme, standard recipes, icons and primitives.

use super::{assert_clean, distance, pixel, pixel_in};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::RenderOptions;
use kivact::ui::{ButtonVariant, ControlSize, ListItem, MenuEntry};
use kivact::view::Orientation;
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .padding(16)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

#[test]
fn theme_switch_restyles_mounted_components_and_keeps_their_state() {
    let (screen, (dark, checked, name)) =
        kivact::test::render_with_handle(crate::options(), || {
            let dark = signal(false);
            let checked = signal(true);
            let name = signal(String::from("Ada"));
            (
                (dark, checked, name),
                kivact::view::provide_theme(
                    move || {
                        basic::theme(if dark.get() {
                            kivact::style::ThemeMode::Dark
                        } else {
                            kivact::style::ThemeMode::Light
                        })
                    },
                    move || {
                        page((
                            kivact::ui::button("Save").test_id("save"),
                            kivact::ui::checkbox("Sync", checked),
                            kivact::ui::text_input("Name", name),
                        ))
                    },
                ),
            )
        });
    assert!(distance(pixel(&screen, 4.0, 4.0), 0xffffff) <= 2);
    assert!(distance(pixel_in(&screen, "save", 4.0), 0x171717) <= 2);
    assert!(
        basic::theme(kivact::style::ThemeMode::Light)
            == basic::theme(kivact::style::ThemeMode::Light),
        "the kit's themes share one recipe table and icon set, so an equal theme compares equal"
    );
    dark.set(true);
    screen.run_until_idle();
    assert!(distance(pixel(&screen, 4.0, 4.0), 0x0a0a0a) <= 2);
    assert!(
        distance(pixel_in(&screen, "save", 4.0), 0xe5e5e5) <= 2,
        "the dark primary fills the mounted button"
    );
    assert!(checked.get_untracked());
    assert_eq!(name.get_untracked(), "Ada");
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "Sync").is_checked(),
        Some(true)
    );
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Name").value().as_deref(),
        Some("Ada")
    );
    assert_clean(&screen);
}

#[test]
fn button_variants_and_sizes_follow_the_lyra_sources() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let variant = |label: &'static str, variant: ButtonVariant| {
            kivact::ui::button(label).variant(variant).test_id(label)
        };
        (
            (),
            page((
                row((
                    variant("Primary", ButtonVariant::Primary),
                    variant("Secondary", ButtonVariant::Secondary),
                    variant("Outline", ButtonVariant::Outline),
                    variant("Ghost", ButtonVariant::Ghost),
                    variant("Danger", ButtonVariant::Danger),
                    variant("Link", ButtonVariant::Link),
                ))
                .style(style().gap(8)),
                row((
                    kivact::ui::button("Small")
                        .size(ControlSize::Small)
                        .test_id("small"),
                    kivact::ui::button("Medium").test_id("medium"),
                    kivact::ui::button("Large")
                        .size(ControlSize::Large)
                        .test_id("large"),
                    kivact::ui::button("Add")
                        .variant(kivact::ui::ButtonVariant::Outline)
                        .hide_label(true)
                        .icon(kivact::control::icon("plus"))
                        .test_id("add"),
                ))
                .style(style().gap(8).align_items(Align::Start)),
            )),
        )
    });
    let fill = |id: &str| pixel_in(&screen, id, 4.0);
    assert!(
        distance(fill("Primary"), 0x171717) <= 2,
        "{:06x}",
        fill("Primary")
    );
    assert!(distance(fill("Secondary"), 0xf5f5f5) <= 2);
    assert!(distance(fill("Outline"), 0xffffff) <= 2);
    assert!(distance(pixel_in(&screen, "Outline", 0.0), 0xe5e5e5) <= 4);
    assert!(distance(fill("Ghost"), 0xffffff) <= 2);
    assert!(distance(pixel_in(&screen, "Ghost", 0.0), 0xffffff) <= 2);
    // `bg-destructive/10` over the white page.
    assert!(
        distance(fill("Danger"), 0xfde6e7) <= 3,
        "{:06x}",
        fill("Danger")
    );
    assert!(distance(fill("Link"), 0xffffff) <= 2);
    let rect = |id: &str| screen.get_by_test_id(id).rect().expect("laid out");
    assert_eq!(rect("small").height, 28.0);
    assert_eq!(rect("medium").height, 32.0);
    assert_eq!(rect("large").height, 36.0);
    assert_eq!((rect("add").width, rect("add").height), (32.0, 32.0));
    let add = screen.get_by_role(Role::Button, "Add");
    assert!(screen.query_by_text("Add").is_none(), "the label is hidden");
    add.focus();
    assert!(add.is_focused());
    assert_clean(&screen);
}

#[test]
fn link_button_underlines_only_while_hovered() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                kivact::ui::button("Read more")
                    .variant(ButtonVariant::Link)
                    .test_id("link"),
            ),
        )
    });
    let link = screen.get_by_test_id("link");
    let rest = link.screenshot().hash();
    link.hover();
    assert_ne!(link.screenshot().hash(), rest, "hover draws the underline");
    assert_clean(&screen);
}

/// The color 3 px in and 13 px down inside a check box's 16 px box, clear of the check mark.
fn check_box_fill(screen: &kivact::test::Screen, name: &str) -> u32 {
    let rect = screen
        .get_by_role(Role::CheckBox, name)
        .rect()
        .expect("laid out");
    let top = ((rect.height - 16.0) / 2.0).round();
    pixel(screen, rect.x + 4.0, rect.y + top + 13.0)
}

#[test]
fn checkbox_and_switch_toggle_with_pointer_and_keyboard() {
    let (screen, (checked, on)) = kivact::test::render_with_handle(crate::options(), || {
        let checked = signal(false);
        let on = signal(false);
        (
            (checked, on),
            page((
                kivact::ui::checkbox("Accept terms", checked),
                kivact::ui::switch("Airplane mode", on).test_id("switch"),
                kivact::ui::checkbox("Partial", signal(false)).indeterminate(true),
                kivact::ui::checkbox("Locked", signal(false)).disabled(true),
            )),
        )
    });
    assert!(distance(check_box_fill(&screen, "Accept terms"), 0xffffff) <= 2);
    let check = screen.get_by_role(Role::CheckBox, "Accept terms");
    check.click();
    assert!(checked.get_untracked());
    assert!(distance(check_box_fill(&screen, "Accept terms"), 0x171717) <= 2);
    check.press_key(Key::Space);
    assert!(!checked.get_untracked());
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Partial")
            .is_indeterminate()
    );
    assert!(screen.get_by_role(Role::CheckBox, "Locked").is_disabled());
    let switch = screen.get_by_role(Role::Switch, "Airplane mode");
    let track = |screen: &kivact::test::Screen| pixel_in(screen, "switch", 6.0);
    assert!(distance(track(&screen), 0x171717) > 40);
    switch.click();
    assert!(on.get_untracked());
    assert!(
        distance(track(&screen), 0x171717) <= 2,
        "{:06x}",
        track(&screen)
    );
    switch.press_key(Key::Space);
    assert!(!on.get_untracked());
    assert_clean(&screen);
}

#[test]
fn text_input_edits_its_binding_and_rings_on_focus() {
    let (screen, (name, notes)) = kivact::test::render_with_handle(crate::options(), || {
        let name = signal(String::new());
        let notes = signal(String::new());
        (
            (name, notes),
            page((
                kivact::ui::text_input("Name", name)
                    .placeholder("Your name")
                    .test_id("name"),
                kivact::ui::text_input("Code", signal(String::new()))
                    .size(ControlSize::Small)
                    .test_id("code"),
                kivact::ui::text_area("Notes", notes).test_id("notes"),
            )),
        )
    });
    let rect = |id: &str| screen.get_by_test_id(id).rect().expect("laid out");
    assert_eq!(rect("name").height, 32.0);
    assert_eq!(rect("code").height, 28.0);
    // Three lines (the engine's least for a multiline input) of 16 px,
    // `py-2` and the 1 px border: 2 px over shadcn's `min-h-16`, whose
    // `field-sizing: content` starts at one line.
    assert_eq!(rect("notes").height, 66.0);

    assert!(distance(pixel_in(&screen, "name", 0.0), 0xe5e5e5) <= 4);
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.click();
    field.type_text("Ada");
    assert_eq!(name.get_untracked(), "Ada");
    assert!(
        distance(pixel_in(&screen, "name", 0.0), 0xa1a1a1) <= 4,
        "the focused border is the ring color: {:06x}",
        pixel_in(&screen, "name", 0.0)
    );
    let area = screen.get_by_role(Role::TextBox, "Notes");
    area.click();
    area.type_text("Line");
    assert_eq!(notes.get_untracked(), "Line");
    assert_clean(&screen);
}

#[test]
fn an_unknown_icon_name_draws_nothing_and_is_reported_at_the_call() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                row((
                    kivact::control::icon("no-such-icon").test_id("unknown"),
                    node().style(style().size(16, 16)).test_id("blank"),
                    kivact::control::icon("check").test_id("check"),
                ))
                .style(style().gap(8)),
            ),
        )
    });
    let shot = |id: &str| screen.get_by_test_id(id).screenshot().hash();
    assert_eq!(
        shot("unknown"),
        shot("blank"),
        "an unknown name draws nothing"
    );
    assert_ne!(shot("check"), shot("blank"));
    assert!(basic::ICON_NAMES.contains(&"loader-circle"));
    assert!(basic::ICON_NAMES.contains(&"octagon-x"));
    assert!(basic::ICON_NAMES.windows(2).all(|pair| pair[0] < pair[1]));
    let errors = screen.errors();
    if cfg!(debug_assertions) {
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message().contains("no-such-icon"), "{errors:?}");
        let at = errors[0].location().expect("located");
        assert!(
            at.file().ends_with("foundation.rs"),
            "located at the call: {at}"
        );
    } else {
        assert!(errors.is_empty(), "{errors:?}");
    }
    screen.assert_layout();
}

#[test]
fn every_listed_icon_decodes() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                row(basic::ICON_NAMES
                    .iter()
                    .map(|name| kivact::control::icon(*name))
                    .collect::<Vec<_>>())
                .style(style().wrap(true).gap(4).width(760)),
            ),
        )
    });
    assert_clean(&screen);
}

#[test]
fn typography_and_primitives_lay_out_cleanly_with_their_semantics() {
    let (screen, ()) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            width: 800.0,
            height: 1400.0,
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            (
                (),
                page((
                    column((
                        basic::h1("Taxing Laughter"),
                        basic::h2("The King's Plan"),
                        basic::h3("The Joke Tax"),
                        basic::h4("People stopped telling jokes"),
                        basic::paragraph("The king thought long and hard."),
                        basic::lead("A modal dialog that interrupts the user."),
                        basic::large("Are you absolutely sure?"),
                        basic::small("Email address"),
                        basic::muted("Enter your email address."),
                        basic::blockquote("After all, everyone enjoys a good joke."),
                        basic::inline_code("cargo run"),
                    ))
                    .style(style().gap(12)),
                    column((
                        basic::bullet_list(vec![
                            "1st level of puns".to_owned(),
                            "2nd level of jokes".to_owned(),
                        ]),
                        basic::kbd_group((basic::kbd("Ctrl"), basic::kbd("K"))),
                        basic::separator(),
                        row((
                            text("Blog"),
                            basic::separator().orientation(Orientation::Vertical),
                            text("Docs"),
                        ))
                        .style(style().gap(8).height(20)),
                        basic::label("Accept terms"),
                        basic::spinner(),
                        basic::skeleton()
                            .style(style().size(200, 16))
                            .test_id("skeleton"),
                        node()
                            .style(style().width(160))
                            .child(basic::aspect_ratio(16.0 / 9.0, node()).test_id("ratio")),
                    ))
                    .style(style().gap(12).width(400)),
                )),
            )
        },
    );
    screen.assert_layout();
    for (name, level) in [
        ("Taxing Laughter", 1),
        ("The King's Plan", 2),
        ("The Joke Tax", 3),
        ("People stopped telling jokes", 4),
    ] {
        assert_eq!(screen.get_by_role(Role::Heading, name).level(), Some(level));
    }
    assert_eq!(screen.all_by_role(Role::Separator, "").len(), 2);
    screen.get_by_role(Role::Status, "Loading");
    screen.get_by_role(Role::List, "");
    screen.get_by_role(Role::ListItem, "1st level of puns");
    screen.get_by_role(Role::ListItem, "2nd level of jokes");
    let ratio = screen.get_by_test_id("ratio").rect().expect("laid out");
    assert_eq!((ratio.width, ratio.height), (160.0, 90.0));
    assert!(
        distance(pixel_in(&screen, "skeleton", 8.0), 0xf5f5f5) <= 2,
        "reduced motion keeps the skeleton opaque"
    );
    assert_clean(&screen);
}

#[test]
fn label_focuses_its_target_and_direction_reaches_the_subtree() {
    let (screen, (seen, outside)) = kivact::test::render_with_handle(crate::options(), || {
        let field = node_ref();
        let seen = signal(None);
        let outside = signal(None);
        outside.set(Some(kivact::view::current_direction()));
        (
            (seen, outside),
            page((
                basic::label("Email").target(field).test_id("email-label"),
                kivact::ui::text_input("Email", signal(String::new())).node_ref(field),
                kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, move || {
                    seen.set(Some(kivact::view::current_direction()));
                    text("مرحبا")
                }),
            )),
        )
    });
    screen.get_by_test_id("email-label").click();
    assert!(screen.get_by_role(Role::TextBox, "Email").is_focused());
    assert_eq!(
        seen.get_untracked(),
        Some(kivact::view::ReadingDirection::Rtl)
    );
    assert_eq!(
        outside.get_untracked(),
        Some(kivact::view::ReadingDirection::Ltr)
    );
    assert_clean(&screen);
}

#[test]
fn select_menu_tabs_and_dialog_use_the_lyra_recipes() {
    let (screen, (fruit, chosen, tab, open)) =
        kivact::test::render_with_handle(crate::options(), || {
            let fruit = signal(None::<u32>);
            let chosen = signal(0u32);
            let tab = signal("account");
            let open = signal(false);
            (
                (fruit, chosen, tab, open),
                page((
                    kivact::ui::select(
                        "Fruit",
                        vec![ListItem::new(1, "Apple"), ListItem::new(2, "Banana")],
                        fruit,
                    )
                    .placeholder("Select a fruit")
                    .test_id("fruit"),
                    kivact::ui::menu(
                        "Open",
                        vec![
                            MenuEntry::item(1, "Profile").shortcut("⇧⌘P"),
                            MenuEntry::item(2, "Status bar").checked(true),
                            MenuEntry::separator(),
                            MenuEntry::item(3, "Log out"),
                        ],
                        move |key: u32| chosen.set(key),
                    ),
                    kivact::ui::tabs(
                        "Tabs",
                        vec![
                            ListItem::new("account", "Account"),
                            ListItem::new("password", "Password"),
                        ],
                        tab,
                    )
                    .test_id("tabs")
                    .content(|key| text(format!("{key} panel"))),
                    kivact::ui::button("Delete").on_press(move || open.set(true)),
                    kivact::ui::dialog("Are you absolutely sure?", open)
                        .description("This action cannot be undone.")
                        .content(|| text("Body"))
                        .footer(move || {
                            kivact::ui::button("Cancel")
                                .variant(ButtonVariant::Outline)
                                .on_press(move || open.set(false))
                        }),
                )),
            )
        });
    assert_eq!(
        screen.get_by_test_id("fruit").rect().map(|r| r.height),
        Some(32.0)
    );
    screen.get_by_role(Role::ComboBox, "Fruit").click();
    screen.get_by_role(Role::Option, "Banana").click();
    assert_eq!(fruit.get_untracked(), Some(2));

    screen.get_by_role(Role::Button, "Open").click();
    screen.get_by_text("⇧⌘P");
    screen.get_by_role(Role::MenuItem, "Log out").click();
    assert_eq!(chosen.get_untracked(), 3);

    assert_eq!(
        screen.get_by_test_id("tabs").rect().map(|r| r.height),
        Some(32.0)
    );
    screen.get_by_role(Role::Tab, "Password").click();
    assert_eq!(tab.get_untracked(), "password");
    screen.get_by_text("password panel");

    screen.get_by_role(Role::Button, "Delete").click();
    let dialog = screen
        .get_by_role(Role::Dialog, "Are you absolutely sure?")
        .rect()
        .expect("dialog laid out");
    assert_eq!(dialog.width, 384.0);
    let close = screen
        .get_by_role(Role::Button, "Close")
        .rect()
        .expect("close laid out");
    assert_eq!((close.width, close.height), (28.0, 28.0));
    assert_eq!(close.x + close.width, dialog.x + dialog.width - 8.0);
    assert_eq!(close.y, dialog.y + 8.0);
    screen.get_by_role(Role::Button, "Close").click();
    assert!(!open.get_untracked());
    assert_clean(&screen);
}
