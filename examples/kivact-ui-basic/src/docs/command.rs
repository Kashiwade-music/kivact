//! Catalog page of Command: the standard command palette in a dialog.

use catalog_shell::{Example, Page, PreviewAlign};
use kivact::interaction::Command;
use kivact::prelude::*;
use kivact::ui::command_palette;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("command", "Command", "Components", include_str!("command.rs"))
        .description("A menu that searches as you type and chooses a command or an item. It shows the standard command_palette in a modal dialog. Typing narrows the items, the arrow keys move, and Enter or a click chooses. Items can have groups, icons and shortcuts.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .align(PreviewAlign::Start)
        .example(Example::new(
            "default",
            "Default",
            "Pass the list of Commands and a callback that receives the key of the chosen one. The shortcut that opens it is the application's job. Choosing an item closes it.",
            default,
        ))
        .example(Example::new(
            "simple",
            "Simple",
            "A list without groups. placeholder and empty_text change the texts.",
            simple,
        ))
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    let last = signal(String::from("Nothing chosen yet."));
    column((
        button("Open command menu")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        basic::muted(move || last.get()),
        command_palette(
            vec![
                Command::new("calendar", "Calendar")
                    .with_group("Suggestions")
                    .with_icon("calendar"),
                Command::new("emoji", "Search Emoji")
                    .with_group("Suggestions")
                    .with_icon("smile")
                    .with_keywords(["face"]),
                Command::new("profile", "Profile")
                    .with_group("Settings")
                    .with_icon("user")
                    .with_hint("Ctrl P"),
                Command::new("billing", "Billing")
                    .with_group("Settings")
                    .with_icon("credit-card")
                    .with_hint("Ctrl B"),
                Command::new("settings", "Settings")
                    .with_group("Settings")
                    .with_icon("settings")
                    .with_hint("Ctrl S"),
            ],
            open,
        )
        .on_activate(move |key: &str| last.set(format!("Ran {key}."))),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: simple
fn simple() -> impl IntoView {
    let open = signal(false);
    column((
        button("Choose a theme")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        command_palette(
            vec![
                Command::new("light", "Light"),
                Command::new("dark", "Dark"),
                Command::new("system", "System"),
            ],
            open,
        )
        .on_activate(|_: &str| {})
        .placeholder("Choose a theme...")
        .empty_text("No theme found."),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example
