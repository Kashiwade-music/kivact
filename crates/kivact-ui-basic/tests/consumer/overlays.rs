//! Role: Overlay tests: accordion, the alert, sheet and popover looks, command menus.

use super::{assert_clean, distance, pixel};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::LayoutCheck;
use kivact::ui::{ButtonVariant, Side};
use kivact::ui::{Command, DialogKind, command_palette};
use kivact::view::{A11yAction, Key};
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

const SECTIONS: [(&str, &str, &str); 3] = [
    ("one", "Is it accessible?", "Yes. It adheres to ARIA."),
    ("two", "Is it styled?", "Yes. It comes with Lyra."),
    ("three", "Is it animated?", "Yes, when motion is on."),
];

/// An accordion: a disclosure per section over one list of open keys. In
/// single mode, opening a section closes the others.
fn accordion(open: Signal<Vec<&'static str>>, multiple: bool) -> Node {
    column(
        SECTIONS
            .map(|(key, title, body)| {
                let is_open = binding(
                    move || open.with(|keys| keys.contains(&key)),
                    move |on| {
                        open.update(|keys| {
                            keys.retain(|k| *k != key);
                            if on {
                                if !multiple {
                                    keys.clear();
                                }
                                keys.push(key);
                            }
                        })
                    },
                );
                kivact::ui::disclosure(title, is_open, move || text(body)).animated(true)
            })
            .into_iter()
            .collect::<Vec<_>>(),
    )
    .style(style().width(300))
}

#[test]
fn accordion_in_single_mode_keeps_one_section_open() {
    let (screen, open) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(vec!["one"]);
        (open, page(accordion(open, false)))
    });
    screen.get_by_text("Yes. It adheres to ARIA.");
    assert!(screen.query_by_text("Yes. It comes with Lyra.").is_none());
    let one = screen.get_by_role(Role::Button, "Is it accessible?");
    let two = screen.get_by_role(Role::Button, "Is it styled?");
    assert_eq!(one.is_expanded(), Some(true));
    assert_eq!(two.is_expanded(), Some(false));

    two.click();
    assert_eq!(
        open.get_untracked(),
        vec!["two"],
        "opening one closes the other"
    );
    screen.get_by_text("Yes. It comes with Lyra.");
    assert!(screen.query_by_text("Yes. It adheres to ARIA.").is_none());
    assert_eq!(one.is_expanded(), Some(false));
    assert_eq!(two.is_expanded(), Some(true));

    let three = screen.get_by_role(Role::Button, "Is it animated?");
    three.focus();
    three.press_key(Key::Enter);
    assert_eq!(open.get_untracked(), vec!["three"], "Enter opens it");
    three.press_key(Key::Space);
    assert!(open.get_untracked().is_empty(), "Space closes it again");
    assert!(screen.query_by_text("Yes, when motion is on.").is_none());

    open.set(vec!["one", "two"]);
    screen.run_until_idle();
    screen.get_by_text("Yes. It adheres to ARIA.");
    screen.get_by_text("Yes. It comes with Lyra.");
    assert_clean(&screen);
    screen.assert_layout_with(&layout_check());
}

#[test]
fn alert_dialog_ignores_outside_presses_and_reports_its_buttons() {
    let (screen, (open, confirmed)) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(false);
        let confirmed = signal(0);
        (
            (open, confirmed),
            page((
                kivact::ui::button("Delete").on_press(move || open.set(true)),
                kivact::ui::dialog("Are you absolutely sure?", open)
                    .kind(DialogKind::AlertModal)
                    .closable(false)
                    .description("This action cannot be undone.")
                    .footer(move || {
                        (
                            kivact::ui::button("Cancel")
                                .variant(ButtonVariant::Outline)
                                .on_press(move || open.set(false)),
                            kivact::ui::button("Continue")
                                .variant(ButtonVariant::Danger)
                                .on_press(move || {
                                    confirmed.update(|n| *n += 1);
                                    open.set(false);
                                }),
                        )
                    }),
            )),
        )
    });
    let opener = screen.get_by_role(Role::Button, "Delete");
    opener.click();
    let dialog = screen.get_by_role(Role::AlertDialog, "Are you absolutely sure?");
    screen.get_by_role(Role::Heading, "Are you absolutely sure?");
    assert_eq!(dialog.description(), "");
    let described: Vec<String> = dialog.described_by().iter().map(|e| e.text()).collect();
    assert_eq!(described, ["This action cannot be undone."]);
    let rect = dialog.rect().expect("laid out");
    assert_eq!(rect.width, 384.0);
    assert!(screen.get_by_role(Role::Button, "Cancel").is_focused());
    // The destructive action is drawn with the destructive tint, not primary.
    let action = screen
        .get_by_role(Role::Button, "Continue")
        .rect()
        .expect("laid out");
    assert!(distance(pixel(&screen, action.x + 4.0, action.y + 4.0), 0xfde6e7) <= 12);

    // A press on the scrim does not close an alert dialog.
    screen.click_at(5.0, 5.0);
    assert!(open.get_untracked());
    screen.get_by_role(Role::AlertDialog, "Are you absolutely sure?");

    screen.get_by_role(Role::Button, "Cancel").click();
    assert!(!open.get_untracked());
    assert_eq!(
        confirmed.get_untracked(),
        0,
        "cancel does not run the action"
    );
    assert!(opener.is_focused(), "focus returns to the opener");

    opener.click();
    screen.get_by_role(Role::Button, "Continue").click();
    assert_eq!(confirmed.get_untracked(), 1);
    assert!(!open.get_untracked());
    assert!(opener.is_focused());

    opener.click();
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked(), "Escape closes it");
    assert_eq!(confirmed.get_untracked(), 1);
    assert_clean(&screen);
}

#[test]
fn sheet_attaches_to_each_side_and_closes_on_escape() {
    for (side, name) in [
        (Side::Right, "Right"),
        (Side::Left, "Left"),
        (Side::Top, "Top"),
        (Side::Bottom, "Bottom"),
    ] {
        let (screen, open) = kivact::test::render_with_handle(crate::options(), move || {
            let open = signal(false);
            (
                open,
                page((
                    kivact::ui::button("Open").on_press(move || open.set(true)),
                    kivact::ui::dialog(name, open)
                        .kind(DialogKind::Drawer(side))
                        .description("Make changes here.")
                        .content(|| text("The body"))
                        .footer(|| kivact::ui::button("Save changes")),
                )),
            )
        });
        let opener = screen.get_by_role(Role::Button, "Open");
        opener.click();
        let dialog = screen.get_by_role(Role::Dialog, name);
        let rect = panel(&dialog);
        match side {
            Side::Right => {
                assert_eq!(rect.x + rect.width, 800.0, "{name}");
                assert_eq!((rect.y, rect.height), (0.0, 600.0), "{name}");
                assert_eq!(rect.width, 384.0, "{name}: sm:max-w-sm");
            }
            Side::Left => {
                assert_eq!((rect.x, rect.y, rect.height), (0.0, 0.0, 600.0), "{name}");
                assert_eq!(rect.width, 384.0, "{name}");
            }
            Side::Top => {
                assert_eq!((rect.x, rect.y, rect.width), (0.0, 0.0, 800.0), "{name}");
                assert!(rect.height < 600.0);
            }
            Side::Bottom => {
                assert_eq!((rect.x, rect.width), (0.0, 800.0), "{name}");
                assert_eq!(rect.y + rect.height, 600.0, "{name}");
            }
        }
        screen.get_by_text("The body");
        screen.get_by_role(Role::Button, "Save changes");
        screen.get_by_role(Role::Button, "Close");
        assert_clean(&screen);
        screen.assert_layout_with(&layout_check());
        screen.press_key(Key::Escape);
        assert!(!open.get_untracked(), "{name}: Escape closes it");
        assert!(opener.is_focused(), "{name}: focus returns to the opener");
    }
}

#[test]
fn popover_opens_from_its_trigger_and_dismisses_with_focus_restored() {
    let (screen, open) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(false);
        (
            open,
            page(
                kivact::ui::dialog("Dimensions", open)
                    .kind(DialogKind::Popover)
                    .description("Set the dimensions for the layer.")
                    .content(|| kivact::ui::text_input("Width", signal(String::from("100%"))))
                    .attach(kivact::ui::button("Open popover").variant(ButtonVariant::Outline)),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Open popover");
    assert_eq!(trigger.is_expanded(), Some(false));
    trigger.click();
    assert!(open.get_untracked());
    assert_eq!(trigger.is_expanded(), Some(true));
    let panel = screen.get_by_role(Role::Dialog, "Dimensions");
    let rect = panel.rect().expect("laid out");
    assert_eq!(rect.width, 288.0);
    let anchor = trigger.rect().expect("laid out");
    assert!(
        rect.y >= anchor.y + anchor.height,
        "opens below the trigger"
    );
    screen.get_by_text("Set the dimensions for the layer.");
    assert!(
        screen.get_by_role(Role::TextBox, "Width").is_focused(),
        "the focus enters the panel"
    );
    assert_clean(&screen);
    screen.assert_layout_with(&layout_check());

    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(
        trigger.is_focused(),
        "Escape returns the focus to the trigger"
    );

    trigger.click();
    assert!(open.get_untracked());
    trigger.click();
    assert!(!open.get_untracked(), "the trigger toggles it");

    trigger.click();
    screen.click_at(700.0, 500.0);
    assert!(!open.get_untracked(), "a press outside closes it");
    assert!(
        !trigger.is_focused(),
        "an outside press keeps its destination"
    );

    trigger.click();
    screen
        .get_by_role(Role::Dialog, "Dimensions")
        .a11y(A11yAction::Dismiss);
    assert!(!open.get_untracked(), "assistive technology can dismiss it");
    assert_clean(&screen);
}

#[test]
fn hover_card_opens_after_its_delay_and_stays_while_hovered() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                basic::hover_card(|| text("The React Framework"))
                    .attach(kivact::ui::button("@nextjs").variant(ButtonVariant::Link)),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "@nextjs");
    trigger.hover();
    screen.advance(Duration::from_millis(500));
    assert!(
        screen.query_by_text("The React Framework").is_none(),
        "not before 600 ms"
    );
    screen.advance(Duration::from_millis(150));
    screen.get_by_text("The React Framework");
    let card = screen.get_by_role(Role::Group, "Preview");
    let rect = card.rect().expect("laid out");
    assert_eq!(rect.width, 256.0);
    assert_clean(&screen);
    screen.assert_layout_with(&layout_check());

    // Moving onto the card keeps it open past the close delay.
    screen.mouse_move(rect.x + 10.0, rect.y + 10.0);
    screen.advance(Duration::from_millis(1000));
    screen.get_by_text("The React Framework");

    // Leaving both closes it after 300 ms.
    screen.mouse_move(700.0, 500.0);
    screen.advance(Duration::from_millis(200));
    screen.get_by_text("The React Framework");
    screen.advance(Duration::from_millis(150));
    assert!(screen.query_by_text("The React Framework").is_none());

    // Escape closes it at once and keeps it closed until the pointer leaves.
    trigger.hover();
    screen.advance(Duration::from_millis(700));
    screen.get_by_text("The React Framework");
    screen.press_key(Key::Escape);
    assert!(screen.query_by_text("The React Framework").is_none());
    screen.advance(Duration::from_millis(1000));
    assert!(screen.query_by_text("The React Framework").is_none());
    assert_clean(&screen);
}

fn entries() -> Vec<Command<&'static str>> {
    vec![
        Command::new("calendar", "Calendar")
            .group("Suggestions")
            .icon("calendar"),
        Command::new("search", "Search Emoji")
            .group("Suggestions")
            .icon("smile")
            .keywords(["face"]),
        Command::new("profile", "Profile")
            .group("Settings")
            .icon("user")
            .hint("Ctrl P"),
        Command::new("settings", "Settings")
            .group("Settings")
            .icon("settings"),
        Command::new("help", "Help"),
    ]
}

#[test]
fn command_filters_groups_and_chooses_with_the_keyboard() {
    let (screen, (chosen, open)) = kivact::test::render_with_handle(crate::options(), || {
        let chosen = signal(Vec::<&'static str>::new());
        let open = signal(true);
        (
            (chosen, open),
            page(command_palette(
                open,
                entries(),
                move |key: &'static str| chosen.update(|keys| keys.push(key)),
            )),
        )
    });
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    screen.get_by_role(Role::Group, "Suggestions");
    screen.get_by_role(Role::Group, "Settings");
    screen.get_by_text("Ctrl P");
    for name in ["Calendar", "Search Emoji", "Profile", "Settings", "Help"] {
        screen.get_by_role(Role::Option, name);
    }
    assert!(screen.get_by_role(Role::Option, "Calendar").is_selected());
    assert_clean(&screen);
    screen.assert_layout_with(&layout_check());

    search.click();
    search.press_key(Key::ArrowDown);
    assert!(
        screen
            .get_by_role(Role::Option, "Search Emoji")
            .is_selected()
    );
    search.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), vec!["search"]);
    assert!(!open.get_untracked(), "choosing closes the modal palette");
    assert!(
        screen
            .query_by_role(Role::Dialog, "Command palette")
            .is_none()
    );
    open.set(true);
    screen.run_until_idle();
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    // The group heading is searched too: both options of "Settings" show,
    // the first of them active.
    search.type_text("sett");
    assert!(screen.query_by_role(Role::Option, "Calendar").is_none());
    assert!(
        screen.query_by_role(Role::Group, "Suggestions").is_none(),
        "an empty group has no heading"
    );
    screen.get_by_role(Role::Option, "Settings");
    assert!(screen.get_by_role(Role::Option, "Profile").is_selected());
    search.press_key(Key::ArrowDown);
    search.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), vec!["search", "settings"]);

    assert!(!open.get_untracked());
    assert!(
        screen
            .query_by_role(Role::Dialog, "Command palette")
            .is_none()
    );
    open.set(true);
    screen.run_until_idle();
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    // Keywords are searched too after reopening with an empty query.
    search.type_text("face");
    screen.get_by_role(Role::Option, "Search Emoji");
    assert!(screen.query_by_role(Role::Option, "Help").is_none());

    search.type_text("zzz");
    screen.get_by_text("No results found.");
    assert!(screen.query_by_role(Role::Option, "Search Emoji").is_none());
    assert_clean(&screen);
}

/// The box of the panel inside a dialog's overlay: the overlay node itself has
/// no size when its panel is placed at the window's edge.
fn panel(dialog: &kivact::test::Element<'_>) -> kivact::draw::Rect {
    dialog
        .children()
        .first()
        .and_then(|panel| panel.rect())
        .expect("the dialog has a laid out panel")
}

/// The layout check with the vertical centering of button labels relaxed: the
/// bundled test font's line box sits about 2 px off the centre of a 12 px
/// label, which the real catalog fonts do not.
fn layout_check() -> LayoutCheck {
    let mut check = LayoutCheck::default();
    check.center_tolerance = Some(3.0);
    check
}
