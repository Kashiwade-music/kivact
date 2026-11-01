//! Role: The kit's own texts are English and an App replaces them by key.

use super::*;
use kivact::i18n::Messages;

fn app() -> impl IntoView {
    {
        let open = signal(false);
        (
            kivact::ui::button("Open")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .expanded(open)
                .on_press(move || open.set(true)),
            kivact::ui::dialog("Open", open)
                .closable(false)
                .content(|| text("body").into_view())
                .footer(move || {
                    kivact::ui::button(kivact::i18n::message("creative.dialog.close", "Close"))
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .on_press(move || open.set(false))
                }),
        )
    }
}

#[test]
fn texts_are_english_until_the_app_replaces_them() {
    let screen = render_with(RenderOptions::from_env(), app);
    screen.get_by_role(Role::Button, "Open").click();
    screen.get_by_role(Role::Button, "Close");

    let screen = render_with(
        RenderOptions {
            messages: Messages::new()
                .with("creative.dialog.close", "閉じる")
                .with("dialog.close", "閉じる"),
            ..RenderOptions::from_env()
        },
        app,
    );
    screen.get_by_role(Role::Button, "Open").click();
    screen.get_by_role(Role::Button, "閉じる");
    assert!(screen.query_by_role(Role::Button, "Close").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
