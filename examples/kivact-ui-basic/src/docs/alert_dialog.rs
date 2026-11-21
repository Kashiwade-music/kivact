//! Catalog page of Alert Dialog: confirmation, destructive, media and small sizes.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::ui::DialogKind;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("alert-dialog", "Alert Dialog", "Components", include_str!("alert_dialog.rs"))
        .description("A modal that asks the user to confirm or to answer. Make the standard kivact::ui::dialog a DialogKind::AlertModal and this theme draws it in the look of alert-dialog.tsx. Escape and the buttons close it; a press outside does not. Focus starts on the first button of the footer, and closing returns focus to the element that opened it.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::DialogKind;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass the open signal and the title, and set the mode to DialogKind::AlertModal. Put the cancel and the action buttons in the footer; pressing one sets open to false. closable(false) hides the close button.",
            default,
        ))
        .example(Example::new(
            "destructive",
            "Destructive",
            "Make the action button a ButtonVariant::Danger to show a destructive action in its color.",
            destructive,
        ))
        .example(Example::new(
            "with-media",
            "With media",
            "Pass a function that returns a 24 px icon to media; it is shown in a 40 px frame beside the title.",
            with_media,
        ))
        .example(Example::new(
            "small",
            "Small",
            "variant(AlertDialogSize::Sm) makes it 320 px wide and centered. Give the two buttons grow(1) and basis(0) to share the width between them.",
            small,
        ))
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    let answer = signal(String::from("Nothing chosen yet."));
    column((
        button("Show Dialog")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        basic::muted(move || answer.get()),
        dialog("Are you absolutely sure?", open)
            .mode(DialogKind::AlertModal)
            .closable(false)
            .description("This action cannot be undone. This will permanently delete your account.")
            .footer(move || {
                (
                    button("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Continue")
                        .variant(ButtonVariant::Primary)
                        .on_press(move || {
                            answer.set(String::from("You chose Continue."));
                            open.set(false);
                        }),
                )
            }),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: destructive
// xtask-allow(state): legacy: open, deleted; "Delete Chat" opens the alert and Cancel or Delete closes it; it opens again after a deletion, so it is true or false with either
fn destructive() -> impl IntoView {
    let open = signal(false);
    let deleted = signal(false);
    column((
        button("Delete Chat")
            .variant(ButtonVariant::Danger)
            .on_press(move || open.set(true)),
        basic::muted(move || {
            if deleted.get() {
                "Chat deleted."
            } else {
                "Chat kept."
            }
            .to_string()
        }),
        dialog("Delete chat?", open)
            .mode(DialogKind::AlertModal)
            .closable(false)
            .description("This will permanently delete this chat conversation.")
            .footer(move || {
                (
                    button("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Delete")
                        .variant(ButtonVariant::Danger)
                        .on_press(move || {
                            deleted.set(true);
                            open.set(false);
                        }),
                )
            }),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: with-media
fn with_media() -> impl IntoView {
    let open = signal(false);
    column((
        button("Share Chat")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Share this chat?", open)
            .mode(DialogKind::AlertModal)
            .closable(false)
            .description("They will be able to see everything in this conversation.")
            .media(|| icon("share").style(style().size(24, 24)))
            .footer(move || {
                (
                    button("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Share").on_press(move || open.set(false)),
                )
            }),
    ))
}
// end example

// example: small
fn small() -> impl IntoView {
    let open = signal(false);
    column((
        button("Show Dialog")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Allow accessory to connect?", open)
            .mode(DialogKind::AlertModal)
            .variant(basic::AlertDialogSize::Sm)
            .closable(false)
            .description("Do you want to allow the USB accessory to connect to this device?")
            .media(|| icon("shield").style(style().size(24, 24)))
            .footer(move || {
                let share = style().flex_grow(1).flex_basis(0);
                (
                    button("Don't allow")
                        .variant(ButtonVariant::Outline)
                        .style(share.clone())
                        .on_press(move || open.set(false)),
                    button("Allow")
                        .style(share)
                        .on_press(move || open.set(false)),
                )
            }),
    ))
}
// end example
