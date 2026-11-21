//! Catalog page of Dialog: default, form, no close button and confirmation.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("dialog", "Dialog", "Components", include_str!("dialog.rs"))
        .description("A modal shown over the page. It is the standard kivact::ui::dialog in the Lyra look; the standard parts hold the focus and close it on Escape.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass the open signal and the title, and functions that make the content(..) and the footer(..). It closes with the close button at the top right or with Escape.",
            default,
        ))
        .example(Example::new(
            "form",
            "Edit profile",
            "An example with an input in the content. Saving applies the value and closes it.",
            edit_profile,
        ))
        .example(Example::new(
            "no-close-button",
            "No close button",
            "closable(false) hides the close button at the top right. Close it with a footer button.",
            no_close_button,
        ))
        .example(Example::new(
            "confirm",
            "Confirmation",
            "A confirmation for an action that cannot be undone. Use a Danger button for a destructive action.",
            confirm,
        ))
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    column((
        button("Open Dialog")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Are you absolutely sure?", open)
            .description("This will update the workspace settings for everyone.")
            .content(|| basic::muted("You can change this later from the settings page."))
            .footer(move || {
                row((
                    button("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Continue").on_press(move || open.set(false)),
                ))
                .style(style().gap(8))
            }),
    ))
}
// end example

// example: form
fn edit_profile() -> impl IntoView {
    let open = signal(false);
    let name = signal(String::from("Pedro Duarte"));
    let username = signal(String::from("@peduarte"));
    column((
        button("Edit Profile")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        basic::muted(move || format!("{} {}", name.get(), username.get())),
        dialog("Edit profile", open)
            .description("Make changes to your profile here. Click save when you're done.")
            .content(move || {
                column((
                    column((
                        basic::label("Name"),
                        text_input(name)
                            .label("Name")
                            .style(style().width(pct(100))),
                    ))
                    .style(style().gap(8)),
                    column((
                        basic::label("Username"),
                        text_input(username)
                            .label("Username")
                            .style(style().width(pct(100))),
                    ))
                    .style(style().gap(8)),
                ))
                .style(style().gap(16))
            })
            .footer(move || button("Save changes").on_press(move || open.set(false))),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: no-close-button
fn no_close_button() -> impl IntoView {
    let open = signal(false);
    column((
        button("No Close Button")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Terms of service", open)
            .closable(false)
            .description("Please read the terms before you continue.")
            .content(|| basic::muted("By continuing you agree to the terms of service."))
            .footer(move || {
                button("I understand")
                    .variant(ButtonVariant::Secondary)
                    .on_press(move || open.set(false))
            }),
    ))
}
// end example

// example: confirm
// xtask-allow(state): independent: open is the dialog binding, set by the Delete account button and cleared by Cancel, Delete or the dialog's own close; deleted only turns true on Delete, and the dialog may be reopened afterwards
fn confirm() -> impl IntoView {
    let open = signal(false);
    let deleted = signal(false);
    column((
        button("Delete account")
            .variant(ButtonVariant::Danger)
            .on_press(move || open.set(true)),
        basic::muted(move || {
            if deleted.get() {
                "Account deleted.".to_string()
            } else {
                "Account active.".to_string()
            }
        }),
        dialog("Delete account?", open)
            .description("This action cannot be undone.")
            .content(|| basic::muted("All of your data will be permanently removed."))
            .footer(move || {
                row((
                    button("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Delete")
                        .variant(ButtonVariant::Danger)
                        .on_press(move || {
                            deleted.set(true);
                            open.set(false);
                        }),
                ))
                .style(style().gap(8))
            }),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example
