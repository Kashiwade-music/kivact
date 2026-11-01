//! Role: Tests the standard command palette on real windows.
//!
//! Its roles and names, the keys that filter, move and run, the open state it
//! keeps, a command's icon and hint, the footer slot and the recipes it reads.

use crate::support::mount;
use kivact::interaction::{ListItemState, PopupState};
use kivact::prelude::*;
use kivact::primitive::PartState;
use kivact::style::{Style, Theme};
use kivact::test::RenderOptions;
use kivact::ui::{Command, Look, UiRecipes, command_palette};
use kivact::view::Modifiers;

fn commands() -> Vec<Command<&'static str>> {
    vec![
        Command::new("new", "New file")
            .group("File")
            .hint("Ctrl N")
            .icon("check"),
        Command::new("open", "Open file")
            .group("File")
            .keywords(["load"]),
        Command::new("theme", "Toggle theme").group("View"),
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
                command_palette(open, commands(), move |key| {
                    ran.update(|keys| keys.push(key))
                }),
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
            command_palette(open, commands(), move |key| {
                ran.update(|keys| keys.push(key))
            }),
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

#[test]
fn the_arrows_move_the_active_row_and_wrap_over_enabled_commands() {
    let (screen, ran) = mount(|| {
        let ran = signal(Vec::<&'static str>::new());
        (
            ran,
            command_palette(signal(true), commands(), move |key| {
                ran.update(|keys| keys.push(key))
            }),
        )
    });
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    search.press_key(Key::ArrowUp);
    assert!(
        screen
            .get_by_role(Role::Option, "Toggle theme")
            .is_selected(),
        "up from the first wraps to the last enabled one, skipping the disabled"
    );
    search.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "New file").is_selected());
    screen.get_by_role(Role::Option, "Open file").hover();
    assert!(screen.get_by_role(Role::Option, "Open file").is_selected());
    screen.get_by_role(Role::Option, "Open file").click();
    assert_eq!(ran.get_untracked(), vec!["open"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_shortcut_given_to_close_on_toggles_it_and_the_footer_slot_is_drawn() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            command_palette(open, commands(), |_| {})
                .close_on(|key| key.modifiers.ctrl && key.key == Key::Char('k'))
                .placeholder("Search the menu")
                .footer(|| text("Up and down move")),
        )
    });
    screen.get_by_text("Up and down move");
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    search.press_key_with(
        Key::Char('k'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_failing_run_is_reported_and_leaves_the_palette_open() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            command_palette(open, commands(), |_| Err(UiError::msg("not now"))),
        )
    });
    screen
        .get_by_role(Role::TextBox, "Search commands")
        .press_key(Key::Enter);
    assert!(open.get_untracked());
    assert_eq!(screen.errors().len(), 1);
}

#[test]
fn a_row_with_an_icon_starts_its_text_later_and_the_recipes_style_every_part() {
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<(bool, bool)>::new()));
    let log = seen.clone();
    let recipes = UiRecipes::default().command_palette(move |table| {
        let log = log.clone();
        table
            .row(move |prev| {
                move |theme: &Theme, look: &Look<ListItemState>| {
                    log.lock().unwrap().push((look.active, look.input.hovered));
                    prev(theme, look)
                }
            })
            .panel(|prev| {
                move |theme: &Theme, look: &Look<PopupState>| prev(theme, look).width(300)
            })
            .footer(|prev| {
                move |theme: &Theme, look: &Look<PartState>| -> Style { prev(theme, look) }
            })
    });
    let (screen, ()) = crate::support::mount_with(
        RenderOptions {
            theme: Theme::light().with(recipes),
            ..RenderOptions::from_env()
        },
        || ((), command_palette(signal(true), commands(), |_| {})),
    );
    let panel = screen
        .get_by_role(Role::Dialog, "Command palette")
        .rect()
        .expect("laid out");
    assert!(
        panel.width <= 300.0 + 1.0,
        "the panel recipe sets the width"
    );
    let icon_text = screen.get_by_text("New file").rect().expect("laid out").x;
    let plain_text = screen.get_by_text("Open file").rect().expect("laid out").x;
    assert!(
        icon_text > plain_text + 8.0,
        "the icon takes room before the label: {icon_text} {plain_text}"
    );
    assert!(
        seen.lock().unwrap().iter().any(|(active, _)| *active),
        "the row recipe sees the active row"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
