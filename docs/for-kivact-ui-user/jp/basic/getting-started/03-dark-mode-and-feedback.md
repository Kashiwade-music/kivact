# 3. ダークモードとフィードバック

[English](../../../en/basic/getting-started/03-dark-mode-and-feedback.md)

このステップで、画面は 3 つのことを覚えます。ヘッダーのスイッチで、アプリの実行中にライトとダークのテーマを切り替えられます。保存するとトーストが出ます。「Delete account」のボタンは、アラートのダイアログで確認を求めます。最後に、実際のウィンドウで画面を操作するテストを書きます。

## `src/main.rs` を置き換える

`profile_card` と `notifications_card` はステップ 2 のものです。`settings` は `dark` のシグナルを受け取るようになり、新しい `root` の関数がテーマを入れます。

```rust,test_harness
use kivact::interaction::{Toast, Tone, use_toaster};
use kivact::prelude::*;
use kivact::style::{FlexDirection, FontFamily, Theme, ThemeMode, font};
use kivact::ui::{DialogKind, toast_region};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    basic::theme(mode).set(&font::FAMILY_VAR, FontFamily::from("Inter, sans-serif"))
}

fn fonts() -> Vec<&'static [u8]> {
    Vec::new()
}

fn profile_card(
    name: Signal<String>,
    saved_name: Signal<String>,
    language: Signal<Option<&'static str>>,
) -> impl Element {
    let languages = vec![
        ListItem::new("en", "English"),
        ListItem::new("ja", "Japanese"),
        ListItem::new("de", "German"),
    ];
    let changed = move || name.get() != saved_name.get();
    kivact::ui::card({
        basic::field_group((
            field("Display name", text_input(name))
                .hint("Shown on your public page."),
            field("Language", select(languages, language)),
        ))
    })
    .header(
        row((
            column((
                text("Profile").style(style().font_size(14).line_height(20.0 / 14.0).font_weight(500)),
                text("Choose how others see you.").style(
                    style()
                        .line_height(1.625)
                        .color(basic::MUTED_FOREGROUND),
                ),
            ))
            .style(style().gap(4).flex_grow(1).min_width(0)),
            row({
                show(changed, || {
                    kivact::ui::badge("Unsaved changes").variant(basic::BadgeVariant::Secondary)
                })
            })
            .style(style().align_self(Align::Start).flex_shrink(0)),
        ))
        .style(style().align_items(Align::Start).gap(8)),
    )
    .style(style().width(420))
}

fn notifications_card(
    digest: Signal<bool>,
    mentions: Signal<bool>,
    reminder: Signal<Option<&'static str>>,
) -> impl Element {
    let reminders = vec![
        ListItem::new("daily", "Daily"),
        ListItem::new("weekly", "Weekly"),
        ListItem::new("never", "Never"),
    ];
    kivact::ui::card({
        basic::field_group((
            field(
                "Weekly digest",
                switch("Weekly digest", digest).hide_label(true),
            )
            .hint("A summary of your week, every Monday.")
            .style(
                style()
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .gap(12),
            ),
            field("Mentions", switch("Mentions", mentions).hide_label(true))
                .hint("Tell me when someone mentions me.")
                .style(
                    style()
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .gap(12),
                ),
            basic::field_separator(),
            field(
                "Reminders",
                kivact::ui::toggle_group("Options", reminders, reminder)
                    .variant(basic::ToggleVariant::Outline),
            ),
        ))
    })
    .header(
        column((
            text("Notifications").style(style().font_size(14).line_height(20.0 / 14.0).font_weight(500)),
            text("Choose what you want to hear about.").style(
                style()
                    .line_height(1.625)
                    .color(basic::MUTED_FOREGROUND),
            ),
        ))
        .style(style().gap(4).flex_grow(1).min_width(0)),
    )
    .style(style().width(420))
}

#[component]
fn settings(dark: Signal<bool>) -> impl Element {
    let name = signal(String::from("Ada"));
    let saved_name = signal(String::from("Ada"));
    let language = signal(Some("en"));
    let digest = signal(true);
    let mentions = signal(false);
    let reminder = signal(Some("weekly"));
    let confirm_delete = signal(false);
    column((
        row((
            column((
                kivact::ui::heading("Settings", 3),
                basic::muted("Manage your profile and notifications."),
            ))
            .style(style().flex_grow(1).gap(8)),
            switch("Dark mode", dark),
        ))
        .style(style().align_items(Align::Center)),
        row((
            profile_card(name, saved_name, language),
            notifications_card(digest, mentions, reminder),
        ))
        .style(style().gap(16).align_items(Align::Start)),
        row((
            button("Save").icon(icon("check")).on_press(move || {
                saved_name.set(name.get());
                use_toaster().push(
                    Toast::new("Settings saved")
                        .tone(Tone::Success)
                        .body(format!("Display name: {}", name.get())),
                );
            }),
            basic::muted(move || format!("Saved name: {}", saved_name.get())),
        ))
        .style(style().gap(12).align_items(Align::Center)),
        basic::separator(),
        kivact::ui::alert("Danger zone")
            .variant(basic::AlertTone::Destructive)
            .icon(icon("triangle-alert"))
            .description("Deleting your account cannot be undone.")
            .actions({
                button("Delete account")
                    .variant(kivact::ui::ButtonVariant::Danger)
                    .on_press(move || confirm_delete.set(true))
            })
            .style(style().width(856)),
        dialog("Delete your account?", confirm_delete)
            .mode(DialogKind::AlertModal)
            .closable(false)
            .description("All of your settings will be removed.")
            .footer(move || {
                (
                    button("Cancel")
                        .variant(kivact::ui::ButtonVariant::Outline)
                        .on_press(move || confirm_delete.set(false)),
                    button("Delete")
                        .variant(kivact::ui::ButtonVariant::Danger)
                        .on_press(move || {
                            confirm_delete.set(false);
                            use_toaster().push(Toast::new("Account deleted"));
                        }),
                )
            }),
        toast_region(use_toaster()).hold_all_on_hover(true),
    ))
    .style(
        style()
            .size(pct(100), pct(100))
            .padding(24)
            .gap(16)
            .background(basic::BACKGROUND)
            .color(basic::FOREGROUND),
    )
}

#[component]
fn root() -> impl IntoView {
    let dark = signal(false);
    kivact::style::provide_theme(
        move || {
            theme(if dark.get() {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            })
        },
        move || settings(dark),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(theme(ThemeMode::Light).with_fonts(fonts()))
        .run(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, render_with};

    fn options() -> RenderOptions {
        RenderOptions::with_theme(theme(ThemeMode::Light))
            .window_size(1000.0, 800.0)
    }

    #[test]
    fn saving_shows_a_toast_and_clears_the_badge() {
        let screen = render_with(options(), root);
        assert!(screen.query_by_text("Unsaved changes").is_none());
        screen
            .get_by_role(Role::TextBox, "Display name")
            .type_text(" Lovelace");
        screen.get_by_text("Unsaved changes");
        screen.get_by_role(Role::Button, "Save").click();
        screen.get_by_role(Role::Status, "Settings saved");
        screen.get_by_text("Saved name: Ada Lovelace");
        assert!(screen.query_by_text("Unsaved changes").is_none());
    }

    #[test]
    fn the_switch_changes_the_theme_and_keeps_the_input() {
        let screen = render_with(options(), root);
        let top_left = |screen: &kivact::test::Screen| {
            screen.screenshot().pixel(4, 4).expect("inside the frame")
        };
        assert_eq!(top_left(&screen) & 0xff, 0xff, "light page");
        screen.get_by_role(Role::Switch, "Dark mode").click();
        assert!(top_left(&screen) & 0xff < 0x20, "dark page");
        assert_eq!(
            screen
                .get_by_role(Role::TextBox, "Display name")
                .value()
                .as_deref(),
            Some("Ada")
        );
    }

    #[test]
    fn deleting_asks_first() {
        let screen = render_with(options(), root);
        screen.get_by_role(Role::Button, "Delete account").click();
        screen.get_by_role(Role::AlertDialog, "Delete your account?");
        screen.get_by_role(Role::Button, "Cancel").click();
        assert!(
            screen
                .query_by_role(Role::AlertDialog, "Delete your account?")
                .is_none()
        );
        assert!(
            screen
                .query_by_role(Role::Status, "Account deleted")
                .is_none()
        );

        screen.get_by_role(Role::Button, "Delete account").click();
        screen.get_by_role(Role::Button, "Delete").click();
        screen.get_by_role(Role::Status, "Account deleted");
    }
}
```

`cargo run` を実行します。ステップ 2 と同じ 2 枚のカードが見え、右上に "Dark mode" のスイッチ、下に赤みを帯びた "Danger zone" の注意書きがあるはずです。次を試してください。

1. "Dark mode" を切り替えます。ページ、カード、コントロールが一度に暗くなります。入力した文字は残ります。何も作り直されず、変わるのは色だけだからです。
2. 名前を変えて Save を押します。右下に "Settings saved" のトーストが出て、数秒後に消えます。
3. "Delete account" を押します。Cancel と Delete のあるダイアログが出て、その後ろのページは操作できなくなります。ダイアログの外をクリックしても閉じません。アラートのダイアログは、はっきりした答えを求めるからです。Cancel と Escape では閉じます。Cancel を押し、もう一度 "Delete account" を押して Delete を押すと、ダイアログが閉じ、トーストに "Account deleted" と出ます。

## 仕組み

- **テーマの切り替え。** `provide_theme(theme, child)` は、その下のサブツリーにテーマを与えます。最初の引数は `dark` を読むクロージャです。そのため `dark` が変わるたびにテーマが作り直され、すべてのスタイルが色を解決し直します。部品は作り直されないので、状態は保たれます。テーマが変わりうるときは、固定の `Theme` の代わりに同じ種類のクロージャを渡してください。
- **トースト。** `toast_region(use_toaster())` は一度だけ配置されます。トーストが現れる場所です。どのハンドラーからも、`use_toaster().push(Toast::new(title))` を呼べます。`.tone(..)` はアイコンと緊急度を決め、`.body(..)` は 2 行目を足します。トーストは 5 秒後に自分で閉じ、ポインターが載っているあいだは止まり、閉じるボタンがあります。
- **アラートのダイアログ。** `dialog(title, open)` は標準のダイアログです。`.mode(DialogKind::AlertModal)` でキットのアラートのダイアログになります。`open` はただの `signal(bool)` です。ボタンがそれを `true` にし、`.footer(..)` のボタンが `false` にします。`.closable(false)` は、アラートのダイアログにはない閉じるボタンを取り除きます。
- **アラートとアラートのダイアログ。** `kivact::ui::alert(..)` はページ上の静的な注意書き（「Danger zone」の枠）です。ダイアログは、割り込んで出るポップアップです。

## テスト

上のブロックは `#[cfg(test)] mod tests` で終わっています。これらのテストは `cargo test` で動きます（ステップ 1 で追加した `test` の機能です）。`kivact::test::render_with(options, root)` は、仮想の時計の上で、実際のウィンドウに画面を配置します。アプリと同じ配置とアクセシビリティのツリーを持ちます。要素は、利用者やスクリーンリーダーと同じように、ロールと名前で探します（`get_by_role(Role::Button, "Save")`、`get_by_text(..)`）。操作は実際の入力（`click`、`type_text`）で行います。テストは `main` を通らないので、テーマはオプションで渡す必要があります（`theme: theme(ThemeMode::Light)`）。

`cargo test` を実行します。3 つのテストが通るはずです。最初の 2 つは、ウィンドウを開くので少し時間がかかります。

## 次へ

白紙のウィンドウから、状態・フィードバック・テストを備えたテーマ付きの画面まで、一通りの道のりを作りました。個別の作業は [how-to ガイド](../how-to-guide/README.md)で解決できます。たとえば[テーマと変数](../how-to-guide/theme-and-tokens.md)や[オーバーレイ](../how-to-guide/overlays.md)があります。キットのすべてのコンポーネントを一度に見るには、リポジトリのクローンで `cargo run -p kivact-ui-basic-catalog` を実行します。ほかのキットは[キットの概要](../../README.md)にあります。
