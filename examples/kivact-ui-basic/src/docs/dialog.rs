//! Role: Catalog page of Dialog: default, form, no close button and confirmation.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "dialog",
        name: "Dialog",
        description: "ページの上に重ねて表示するモーダルです。標準の kivact::ui::dialog に Lyra の見た目を当てたもので、フォーカスの閉じ込めと Escape での閉じる動作は標準の部品が行います。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact_ui_basic as basic;",
        source: include_str!("dialog.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "open の signal と題名を渡し、show に本文とフッターを返す関数を渡します。右上の閉じるボタンと Escape で閉じます。",
                default,
            ),
            Example::new(
                "form",
                "Edit profile",
                "本文に入力欄を置いた例です。保存すると値を反映して閉じます。",
                edit_profile,
            ),
            Example::new(
                "no-close-button",
                "No close button",
                "closable を false にすると右上の閉じるボタンを出しません。フッターのボタンで閉じます。",
                no_close_button,
            ),
            Example::new(
                "confirm",
                "Confirmation",
                "取り消せない操作の確認です。破壊的な操作には Danger のボタンを使います。",
                confirm,
            ),
        ],
    }
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
                        text_input("Name", name).style(style().width(pct(100))),
                    ))
                    .style(style().gap(8)),
                    column((
                        basic::label("Username"),
                        text_input("Username", username).style(style().width(pct(100))),
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
