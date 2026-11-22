# 3. ダークモードとフィードバック

[English](../../../en/basic/getting-started/03-dark-mode-and-feedback.md)

このステップで、画面に 3 つのことを教えます。ヘッダーのスイッチで、アプリの実行中にライトテーマとダークテーマを切り替えます。保存するとトーストを出します。「アカウントを削除」ボタンは、アラートダイアログで確認を求めます。最後に、実際のウィンドウで画面を操作するテストを書きます。

## `src/main.rs` を置き換える

`profile_card` と `notifications_card` はステップ 2 のものです。`settings` は `dark` signal を受け取るようになり、新しい `root` 関数がテーマを設定します。

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

`cargo run` で実行します。ステップ 2 と同じ 2 枚のカードに加え、右上に「ダークモード」のスイッチ、下に赤みを帯びた「危険な操作」の枠が表示されます。次のことを試してください。

1. 「ダークモード」を切り替えます。ページ、カード、コントロールがすべて一度に暗くなります。入力した文字は残ります。作り直しは行われず、色だけが変わるからです。
2. 名前を変えて「保存」を押します。右下にトースト「設定を保存しました」が現れ、数秒後に消えます。
3. 「アカウントを削除」を押します。「キャンセル」と「削除」を持つダイアログが開き、背後のページは操作できなくなります。ダイアログの外側をクリックしても閉じません。アラートダイアログは、はっきりした返答を求めるからです。閉じるのは「キャンセル」か Escape キーです。「キャンセル」を押してから、もう一度「アカウントを削除」と「削除」を押すと、ダイアログが閉じ、トーストに「アカウントを削除しました」と表示されます。

## 仕組み

- **テーマの切り替え。** `provide_theme(theme, child)` は、その内側の部分木にテーマを与えます。第 1 引数は `dark` を読むクロージャなので、`dark` が変わるたびにテーマが作り直され、すべてのスタイルが色を解決し直します。コンポーネントは再マウントされないので、状態を保ちます。テーマが変わりうるときは、固定の `Theme` ではなく、このようなクロージャを渡します。
- **トースト。** `toast_region(use_toaster())` は一度だけマウントします。トーストが現れる場所です。以降は、どのハンドラからでも `use_toaster().push(Toast::new(title))` と呼べます。`.tone(..)` でアイコンと緊急度を選び、`.body(..)` で 2 行目を足します。トーストは 5 秒で自動的に閉じ、ポインターが乗っている間は止まり、閉じるボタンも持ちます。
- **アラートダイアログ。** `dialog(title, open)` は標準のダイアログで、`.mode(DialogMode::AlertModal)` にするとキットのアラートダイアログになります。`open` はふつうの `signal(bool)` です。ボタンが `true` にし、`.footer(..)` のボタンが `false` にします。`.closable(false)` は、アラートダイアログが持たない閉じるボタンを取り除きます。
- **alert とアラートダイアログの違い。** `kivact::ui::alert(..)` はページに置く静的な囲み（「危険な操作」の枠）です。ダイアログは、操作を中断させるポップアップです。

## テスト

上のブロックの末尾に `#[cfg(test)] mod tests` があります。このテストは `cargo test` で動きます（ステップ 1 で足した `test` feature を使います）。`kivact::test::render_with(options, root)` は、画面を実際のウィンドウにマウントし、仮想の時計で動かします。レイアウトとアクセシビリティツリーはアプリと同じです。要素は、ユーザーやスクリーンリーダーと同じ見方、つまりロールと名前（`get_by_role(Role::Button, "保存")`、`get_by_text(..)`）で探し、実際の入力（`click`、`type_text`）で操作します。テストは `main` を通らないので、テーマはオプションで渡す必要があります（`theme: theme(ThemeMode::Light)`）。

`cargo test` を実行します。3 つのテストが成功します。それぞれウィンドウを開くため、最初の 2 つは少し時間がかかります。

## 次へ

空のウィンドウから、状態とフィードバックとテストを備えたテーマ付きの画面まで、一通りの道のりを作りました。続いて、[how-to ガイド](../how-to-guide/README.md)で個別の作業の進め方を確認してください。たとえば[テーマとトークン](../how-to-guide/theme-and-tokens.md)や[オーバーレイ](../how-to-guide/overlays.md)があります。キットのすべてのコンポーネントを一度に見るには、リポジトリのクローンで `cargo run -p kivact-ui-basic-catalog` を実行します。ほかのキットは[キットの一覧](../../README.md)にあります。
