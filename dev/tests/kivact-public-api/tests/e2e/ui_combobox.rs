//! Role: Verify the standard combobox family and the touch select sheet.
use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::{SelectPresentation, combobox, select};
use kivact::view::PointerKind;
use std::time::Duration;

#[test]
fn combobox_filters_options_and_selects_many_as_chips() {
    let (screen, (chosen, options)) = mount(|| {
        let chosen = signal(Vec::<&str>::new());
        let options = signal(vec![
            ListItem::new("red", "Red"),
            ListItem::new("green", "Green"),
            ListItem::new("blue", "Blue"),
        ]);
        (
            (chosen, options),
            combobox("Colors", options, chosen)
                .chips()
                .style(style().width(300)),
        )
    });
    let input = screen.get_by_role(Role::ComboBox, "Colors");
    input.focus();
    input.type_text("re");
    assert!(screen.query_by_role(Role::Option, "Blue").is_none());
    input.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), vec!["red"]);
    screen.get_by_role(Role::Button, "Remove Red");
    input.type_text("gr");
    input.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), vec!["red", "green"]);
    screen.get_by_role(Role::Button, "Remove Green");
    input.press_key(Key::Backspace);
    assert_eq!(chosen.get_untracked(), vec!["red"]);
    options.set(vec![ListItem::new("pink", "Pink")]);
    screen.run_until_idle();
    assert_eq!(chosen.get_untracked(), vec!["red"]);
    screen.get_by_role(Role::Button, "Remove Red").click();
    assert!(chosen.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn select_touch_opens_the_sheet() {
    let (screen, chosen) = mount(|| {
        let chosen = signal(Some("a"));
        (
            chosen,
            select(
                "Team",
                vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")],
                chosen,
            )
            .presentation(SelectPresentation::BottomSheet)
            .style(style().width(280)),
        )
    });
    let input = screen.get_by_role(Role::ComboBox, "Team");
    let rect = input.rect().unwrap();
    let contact = screen.contact_down(
        PointerKind::Touch,
        rect.x + rect.width / 2.0,
        rect.y + rect.height / 2.0,
    );
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    screen.contact_up(contact);
    screen.advance(Duration::from_millis(350));
    screen.get_by_role(Role::Dialog, "Team");
    screen.get_by_role(Role::Option, "Beta").click();
    assert_eq!(chosen.get_untracked(), Some("b"));
    screen.advance(Duration::from_millis(350));
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    input.focus();
    input.press_key(Key::ArrowDown);
    // Closed select arrows change the value; Alt+Down opens its listbox.
    input.press_key_with(
        Key::ArrowDown,
        kivact::view::Modifiers {
            alt: true,
            ..Default::default()
        },
    );
    screen.get_by_role(Role::ListBox, "Team");
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
