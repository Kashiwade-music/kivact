//! Role: The kit's own texts are English and an App replaces them by key.

use super::*;
use kivact::i18n::Messages;

fn app() -> impl IntoView {
    let open = signal(true);
    column((
        kivact::ui::dialog("Title", open).content(|| text("body")),
        g::pagination(signal(2_usize), 5),
    ))
}

#[test]
fn texts_are_english_until_the_app_replaces_them() {
    let screen = render_with(RenderOptions::from_env(), app);
    screen.get_by_role(Role::Button, "Close");
    screen.get_by_role(Role::Button, "Previous");
    screen.get_by_role(Role::Group, "Pages");

    let screen = render_with(
        RenderOptions {
            messages: Messages::new()
                .with("dialog.close", "ダイアログを閉じる")
                .with("game.pagination.previous", "前へ")
                .with("game.pagination.page", "ページ"),
            ..RenderOptions::from_env()
        },
        app,
    );
    screen.get_by_role(Role::Button, "ダイアログを閉じる");
    screen.get_by_role(Role::Button, "前へ");
    screen.get_by_role(Role::Group, "ページ");
    // A key the App did not set keeps the kit's text.
    screen.get_by_role(Role::Button, "Next");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn switch_paint_uses_the_apps_localized_on_and_off_messages() {
    let app = || {
        column((
            kivact::ui::switch("Enabled", signal(true)),
            kivact::ui::switch("Disabled", signal(false)),
        ))
        .style(style().padding(20).gap(8))
    };
    let english = render_with(options(false), app);
    let initial = english.screenshot().hash();
    assert_eq!(
        english.get_by_role(Role::Switch, "Enabled").is_checked(),
        Some(true)
    );
    assert_eq!(
        english.get_by_role(Role::Switch, "Disabled").is_checked(),
        Some(false)
    );
    drop(english);
    let translated = render_with(
        RenderOptions {
            messages: Messages::new()
                .with("switch.on", "入")
                .with("switch.off", "切"),
            ..options(false)
        },
        app,
    );
    assert_ne!(initial, translated.screenshot().hash());
    assert!(translated.errors().is_empty(), "{:?}", translated.errors());
}
