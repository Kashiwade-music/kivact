//! Role: Tests for localization and component text handling.
//!
//! The locale and the texts of components (`kivact::i18n`): a component's
//! default text, the App's replacement, a switch at runtime and a subtree's
//! own.

use kivact::i18n::{Locale, Messages, locale, message, message_with};
use kivact::prelude::*;
use kivact::test::{RenderOptions, render, render_with};
use kivact::view::{provide_locale, provide_messages};

/// A part that shows one of its texts the way a component library does.
fn close_label() -> impl IntoView {
    text(message("dialog.close", "Close")).test_id("close")
}

fn locale_tag() -> impl IntoView {
    let locale = locale();
    text(move || locale.get().tag().to_owned()).test_id("locale")
}

#[test]
fn the_apps_messages_replace_the_default() {
    let screen = render_with(
        RenderOptions {
            messages: Messages::new().with("dialog.close", "閉じる"),
            ..RenderOptions::from_env()
        },
        || {
            column((
                close_label(),
                text(message("dialog.cancel", "Cancel")).test_id("cancel"),
                // The label a screen reader reads follows too.
                node()
                    .role(Role::Button)
                    .name(message("dialog.close", "Close"))
                    .focusable(true),
            ))
        },
    );
    screen.get_by_text("閉じる");
    screen.get_by_text("Cancel");
    screen.get_by_role(Role::Button, "閉じる");
    assert!(screen.query_by_text("Close").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn switching_the_locale_and_the_messages_at_runtime_updates_what_reads_them() {
    let screen = render(|| {
        let tag = signal(Locale::new("en-US"));
        let messages = signal(Messages::new());
        column((
            provide_locale(tag, move || {
                provide_messages(messages, || column((locale_tag(), close_label())))
            }),
            node()
                .role(Role::Button)
                .name("to-ja")
                .focusable(true)
                .style(style().size(80, 20))
                .on_press(move || {
                    tag.set(Locale::new("ja-JP"));
                    messages.set(Messages::new().with("dialog.close", "閉じる"));
                }),
        ))
    });
    screen.get_by_text("en-US");
    screen.get_by_text("Close");
    let before = screen.stats();

    screen.get_by_role(Role::Button, "to-ja").click();
    screen.get_by_text("ja-JP");
    screen.get_by_text("閉じる");
    assert!(screen.query_by_text("Close").is_none());
    assert_eq!(
        screen.stats().creates,
        before.creates,
        "the switch remounts nothing"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_subtree_has_its_own_locale_and_messages_and_siblings_keep_the_outer() {
    let screen = render_with(
        RenderOptions {
            locale: Some(Locale::new("en-US")),
            messages: Messages::new()
                .with("dialog.close", "Close it")
                .with("dialog.cancel", "Cancel it"),
            ..RenderOptions::from_env()
        },
        || {
            column((
                provide_locale(Locale::new("fr-FR"), || {
                    provide_messages(Messages::new().with("dialog.close", "Fermer"), || {
                        column((
                            text({
                                let locale = locale();
                                move || format!("inner {}", locale.get().tag())
                            }),
                            close_label(),
                            // Not set inside: the App's text shows through.
                            text(message("dialog.cancel", "Cancel")),
                        ))
                    })
                }),
                text({
                    let locale = locale();
                    move || format!("outer {}", locale.get().tag())
                }),
                text(message("dialog.close", "Close")),
            ))
        },
    );
    screen.get_by_text("inner fr-FR");
    screen.get_by_text("Fermer");
    screen.get_by_text("Cancel it");
    screen.get_by_text("outer en-US");
    screen.get_by_text("Close it");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_message_with_values_follows_both_the_values_and_the_messages() {
    let screen = render(|| {
        let count = signal(1usize);
        let wording = signal(Messages::new());
        column((
            provide_messages(wording, move || {
                text(message_with(
                    "table.selected",
                    "{count} selected",
                    move || [("count", count.get().to_string())],
                ))
            }),
            // A name the wording does not give stays as written.
            text(message_with("x.unknown", "{who} and {count}", || {
                [("count", "2".to_owned())]
            })),
            node()
                .role(Role::Button)
                .name("more")
                .focusable(true)
                .style(style().size(60, 20))
                .on_press(move || count.update(|n| *n += 1)),
            node()
                .role(Role::Button)
                .name("ja")
                .focusable(true)
                .style(style().size(60, 20))
                .on_press(move || {
                    wording.set(Messages::new().with("table.selected", "{count} 件選択中"));
                }),
        ))
    });
    screen.get_by_text("1 selected");
    screen.get_by_text("{who} and 2");
    screen.get_by_role(Role::Button, "more").click();
    screen.get_by_text("2 selected");
    screen.get_by_role(Role::Button, "ja").click();
    screen.get_by_text("2 件選択中");
    screen.get_by_role(Role::Button, "more").click();
    screen.get_by_text("3 件選択中");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
