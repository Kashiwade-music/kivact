//! Role: The kit's own texts are English and an App replaces them by key.

use super::*;
use kivact::i18n::{Messages, message};
use kivact::ui::ListItem;
use kivact::view::provide_messages;

fn dialog_app() -> impl IntoView {
    let open = signal(true);
    kivact::ui::dialog("Title", open)
        .content(|| kivact::ui::text_input("Memo", kivact::signal(String::new())))
}

#[test]
fn dialog_texts_are_english_until_the_app_replaces_them() {
    let screen = render_with(RenderOptions::from_env(), dialog_app);
    screen.get_by_role(Role::Button, "Close");

    let screen = render_with(
        RenderOptions {
            messages: Messages::new().with("dialog.close", "閉じる"),
            ..RenderOptions::from_env()
        },
        dialog_app,
    );
    screen.get_by_role(Role::Button, "閉じる");
    assert!(screen.query_by_role(Role::Button, "Close").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_subtree_and_a_runtime_switch_reach_the_kit() {
    let screen = render_with(RenderOptions::from_env(), || {
        let messages = signal(Messages::new());
        let selected = signal(vec![1_u64, 2]);
        column((
            provide_messages(messages, move || {
                kivact::ui::data_table(
                    vec![(1_u64, "a"), (2, "b")],
                    |row: &(u64, &str)| row.0,
                    vec![
                        kivact::ui::DataColumn::new("Name", |row: &(u64, &str)| row.1.to_owned())
                            .width(120.0),
                    ],
                )
                .selection(selected)
            }),
            button_that_switches(messages),
        ))
    });
    screen.get_by_text("2 of 2 row(s) selected.");
    screen.get_by_role(Role::Button, "ja").click();
    screen.get_by_text("2 / 2 件選択中");
    screen.get_by_role(Role::Grid, "データテーブル");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn options_built_in_a_handler_follow_a_runtime_locale_switch() {
    let screen = render_with(RenderOptions::from_env(), || {
        let messages = signal(Messages::new());
        let selected = signal(Some("a".to_owned()));
        let options = signal(Vec::<ListItem<String>>::new());
        column((
            provide_messages(messages, move || {
                // The messages are read in setup; the options that hold them
                // are built later, in a handler.
                let alpha = message("option.alpha", "Alpha");
                let beta = message("option.beta", "Beta");
                column((
                    kivact::ui::select("Academy", options, selected),
                    kivact::ui::button("load").on_press(move || {
                        options.set(vec![
                            ListItem::new("a".to_owned(), alpha),
                            ListItem::new("b".to_owned(), beta),
                        ]);
                    }),
                ))
            }),
            kivact::ui::button("ja").on_press(move || {
                messages.set(
                    Messages::new()
                        .with("option.alpha", "アルファ")
                        .with("option.beta", "ベータ"),
                );
            }),
        ))
    });
    screen.get_by_role(Role::Button, "load").click();
    let trigger = screen.get_by_role(Role::ComboBox, "Academy");
    assert_eq!(trigger.value().as_deref(), Some("Alpha"));
    trigger.click();
    screen.get_by_role(Role::Option, "Beta");
    screen.press_key(Key::Escape);
    screen.get_by_role(Role::Button, "ja").click();
    screen.run_until_idle();
    assert_eq!(trigger.value().as_deref(), Some("アルファ"));
    trigger.click();
    screen.get_by_role(Role::Option, "ベータ");
    assert!(screen.query_by_role(Role::Option, "Beta").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn button_that_switches(messages: Signal<Messages>) -> impl IntoView {
    kivact::ui::button("ja").on_press(move || {
        messages.set(
            Messages::new()
                .with("data_table.selected", "{selected} / {total} 件選択中")
                .with("data_table.label", "データテーブル"),
        );
    })
}
