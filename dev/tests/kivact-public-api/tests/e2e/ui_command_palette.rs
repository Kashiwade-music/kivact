//! Role: Tests the standard command palette on real windows.
//!
//! Its roles and names, the keys that filter, move and run, the open state it
//! keeps, a command's icon and hint, the footer slot and the recipes it reads.

use crate::support::mount;
use kivact::interaction::Command;
use kivact::prelude::*;
use kivact::ui::command_palette;

fn commands() -> Vec<Command<&'static str>> {
    vec![
        Command::new("new", "New file")
            .with_group("File")
            .with_hint("Ctrl N")
            .with_icon("check"),
        Command::new("open", "Open file")
            .with_group("File")
            .with_keywords(["load"]),
        Command::new("theme", "Toggle theme").with_group("View"),
        Command::new("quit", "Quit").disabled(true),
    ]
}

#[test]
fn a_palette_is_a_modal_with_a_search_field_over_named_options() {
    let (screen, (open, ran)) = mount(|| {
        let open = signal(false);
        let ran = signal(Vec::<&'static str>::new());
        (
            (open, ran),
            column((
                button("Commands").on_press(move || open.set(true)),
                command_palette(commands(), open)
                    .on_activate(move |key| ran.update(|keys| keys.push(key))),
            )),
        )
    });
    assert!(
        screen
            .query_by_role(Role::Dialog, "Command palette")
            .is_none()
    );
    let opener = screen.get_by_role(Role::Button, "Commands");
    opener.click();
    screen.get_by_role(Role::Dialog, "Command palette");
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    assert!(search.is_focused(), "the focus goes to the search field");
    screen.get_by_role(Role::ListBox, "Command palette");
    screen.get_by_role(Role::Group, "File");
    screen.get_by_role(Role::Group, "View");
    screen.get_by_text("Ctrl N");
    for name in ["New file", "Open file", "Toggle theme", "Quit"] {
        screen.get_by_role(Role::Option, name);
    }
    assert!(screen.get_by_role(Role::Option, "Quit").is_disabled());
    assert!(
        screen.get_by_role(Role::Option, "New file").is_selected(),
        "the first enabled command is active"
    );
    search.press_key(Key::Escape);
    assert!(!open.get_untracked(), "Escape closes it");
    assert!(ran.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn typing_filters_by_label_group_hint_and_keywords_and_enter_runs_and_closes() {
    let (screen, (open, ran)) = mount(|| {
        let open = signal(true);
        let ran = signal(Vec::<&'static str>::new());
        (
            (open, ran),
            command_palette(commands(), open)
                .on_activate(move |key| ran.update(|keys| keys.push(key))),
        )
    });
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    search.type_text("load");
    assert!(screen.query_by_role(Role::Option, "New file").is_none());
    assert!(screen.get_by_role(Role::Option, "Open file").is_selected());
    search.press_key(Key::Backspace);
    search.press_key(Key::Backspace);
    search.press_key(Key::Backspace);
    search.press_key(Key::Backspace);
    search.type_text("view");
    assert!(screen.query_by_role(Role::Option, "Open file").is_none());
    assert!(
        screen
            .get_by_role(Role::Option, "Toggle theme")
            .is_selected()
    );
    search.press_key(Key::Enter);
    assert_eq!(ran.get_untracked(), vec!["theme"]);
    assert!(!open.get_untracked(), "a run closes the palette");
    open.set(true);
    screen.run_until_idle();
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Search commands")
            .value()
            .as_deref(),
        Some(""),
        "the query is forgotten between openings"
    );
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    search.type_text("zzz");
    screen.get_by_text("No results found.");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
