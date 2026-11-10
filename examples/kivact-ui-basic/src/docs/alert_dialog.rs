//! Role: Catalog page of Alert Dialog: confirmation, destructive, media and small sizes.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "alert-dialog",
        name: "Alert Dialog",
        description: "ユーザーに確認や回答を求めるモーダルです。標準の kivact::ui::dialog を DialogMode::AlertModal にすると、このテーマでは alert-dialog.tsx の見た目で描きます。Escape とボタンで閉じ、外側の押下では閉じません。フォーカスはフッターの最初のボタンから始まり、閉じると開いた要素へ戻ります。",
        usage: "use kivact::prelude::*;\nuse kivact::interaction::DialogMode;\nuse kivact_ui_basic as basic;",
        source: include_str!("alert_dialog.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "open の signal と題名を渡し、mode を DialogMode::AlertModal にします。キャンセルと実行のボタンは footer に置き、押したときに open を false にします。閉じるボタンは closable(false) で出しません。",
                default,
            ),
            Example::new(
                "destructive",
                "Destructive",
                "実行のボタンを ButtonVariant::Danger にすると、破壊的な操作の色になります。",
                destructive,
            ),
            Example::new(
                "with-media",
                "With media",
                "media に 24 px のアイコンなどを返す関数を渡すと、題名の脇に 40 px の枠で表示します。",
                with_media,
            ),
            Example::new(
                "small",
                "Small",
                "variant(AlertDialogSize::Sm) にすると 320 px 幅の中央寄せになります。2 つのボタンに grow(1) と basis(0) を付けると幅を分け合います。",
                small,
            ),
        ],
    }
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
            .mode(DialogMode::AlertModal)
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
fn destructive() -> impl IntoView {
    // xtask-allow(state-fields): independent: the dialog can be opened again after the deletion; every open and closed state is valid
    let open = signal(false);
    // xtask-allow(state-fields): independent: a one-way record of the deletion, kept after the dialog closes
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
            .mode(DialogMode::AlertModal)
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
            .mode(DialogMode::AlertModal)
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
            .mode(DialogMode::AlertModal)
            .variant(basic::AlertDialogSize::Sm)
            .closable(false)
            .description("Do you want to allow the USB accessory to connect to this device?")
            .media(|| icon("shield").style(style().size(24, 24)))
            .footer(move || {
                let share = style().grow(1).basis(0);
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
