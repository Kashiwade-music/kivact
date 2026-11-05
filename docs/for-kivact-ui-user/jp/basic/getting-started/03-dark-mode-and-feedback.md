# 3. ダークモードとフィードバック

[English](../../../en/basic/getting-started/03-dark-mode-and-feedback.md)

このステップで、画面に 3 つのことを教えます。ヘッダーのスイッチで、アプリの実行中にライトテーマとダークテーマを切り替えます。保存するとトーストを出します。「アカウントを削除」ボタンは、アラートダイアログで確認を求めます。最後に、実際のウィンドウで画面を操作するテストを書きます。

## `src/main.rs` を置き換える

`profile_card` と `notifications_card` はステップ 2 のものです。`settings` は `dark` signal を受け取るようになり、新しい `root` 関数がテーマを設定します。

```rust,test_harness
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact::style::{Direction, Theme, ThemeMode};
use kivact::interaction::{DialogMode, Toast, Tone, toaster};
use kivact::ui::{toast_region};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let text = Typography {
        family: Some("Inter, 'Noto Sans JP', sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

fn fonts() -> Vec<&'static [u8]> {
    Vec::new()
}

fn profile_card(
    name: Signal<String>,
    saved_name: Signal<String>,
    language: Signal<Option<&'static str>>,
) -> impl IntoView {
    let languages = vec![
        ListItem::new("en", "English"),
        ListItem::new("ja", "日本語"),
        ListItem::new("de", "Deutsch"),
    ];
    let changed = move || name.get() != saved_name.get();
    basic::card()
        .header(
            basic::card_header("プロフィール")
                .description("ほかの人にどう見えるかを決めます。")
                .action({
                    show(changed, || {
                        kivact::ui::badge("未保存の変更").variant(basic::BadgeVariant::Secondary)
                    })
                }),
        )
        .content({
            basic::field_group((
                field("表示名", text_input("表示名", name))
                    .hint("公開ページに表示されます。"),
                field("言語", select("言語", languages, language)),
            ))
        })
        .style(style().width(420))
}

fn notifications_card(
    digest: Signal<bool>,
    mentions: Signal<bool>,
    reminder: Signal<Option<&'static str>>,
) -> impl IntoView {
    let reminders = vec![
        ListItem::new("daily", "毎日"),
        ListItem::new("weekly", "毎週"),
        ListItem::new("never", "なし"),
    ];
    basic::card()
        .header(basic::card_header("通知").description("受け取りたいお知らせを選びます。"))
        .content({
            basic::field_group((
                field("週間ダイジェスト", switch("週間ダイジェスト", digest).hide_label(true))
                    .hint("毎週月曜日に、1 週間のまとめが届きます。")
                    .style(style().direction(Direction::Row).align_items(Align::Center).gap(12)),
                field("メンション", switch("メンション", mentions).hide_label(true))
                    .hint("自分宛てのメンションを知らせます。")
                    .style(style().direction(Direction::Row).align_items(Align::Center).gap(12)),
                basic::field_separator(),
                field(
                    "リマインダー",
                    kivact::ui::toggle_group("Options", reminders, reminder).variant(basic::ToggleVariant::Outline),
                ),
            ))
        })
        .style(style().width(420))
}

fn settings(dark: Signal<bool>) -> impl IntoView {
    let name = signal(String::from("Ada"));
    let saved_name = signal(String::from("Ada"));
    let language = signal(Some("ja"));
    let digest = signal(true);
    let mentions = signal(false);
    let reminder = signal(Some("weekly"));
    let confirm_delete = signal(false);
    column((
        row((
            column((
                basic::h3("設定"),
                basic::muted("プロフィールと通知を管理します。"),
            ))
            .style(style().grow(1).gap(8)),
            switch("ダークモード", dark),
        ))
        .style(style().align_items(Align::Center)),
        row((
            profile_card(name, saved_name, language),
            notifications_card(digest, mentions, reminder),
        ))
        .style(style().gap(16).align_items(Align::Start)),
        row((
            button("保存").icon(icon("check")).on_press(move || {
                saved_name.set(name.get());
                toaster().push(
                    Toast::new("設定を保存しました")
                        .tone(Tone::Success)
                        .body(format!("表示名: {}", name.get())),
                );
            }),
            basic::muted(move || format!("保存済みの名前: {}", saved_name.get())),
        ))
        .style(style().gap(12).align_items(Align::Center)),
        basic::separator(),
        basic::alert("危険な操作")
            .variant(basic::AlertVariant::Destructive)
            .icon(icon("triangle-alert"))
            .description("アカウントを削除すると、元に戻せません。")
            .action({
                button("アカウントを削除")
                    .variant(kivact::ui::ButtonVariant::Danger)
                    .on_press(move || confirm_delete.set(true))
            })
            .style(style().width(856)),
        dialog("アカウントを削除しますか？", confirm_delete)
            .mode(DialogMode::AlertModal)
            .closable(false)
            .description("すべての設定が削除されます。")
            .footer(move || {
                (
                    button("キャンセル")
                        .variant(kivact::ui::ButtonVariant::Outline)
                        .on_press(move || confirm_delete.set(false)),
                    button("削除")
                        .variant(kivact::ui::ButtonVariant::Danger)
                        .on_press(move || {
                            confirm_delete.set(false);
                            toaster().push(Toast::new("アカウントを削除しました"));
                        }),
                )
            }),
        toast_region(toaster()).hold_all_on_hover(true),
    ))
    .style(
        style()
            .size(pct(100), pct(100))
            .padding(24)
            .gap(16)
            .background(basic::ColorName::Background)
            .color(basic::ColorName::Foreground),
    )
}

fn root() -> impl IntoView {
    let dark = signal(false);
    kivact::style::provide_theme(
        move || theme(if dark.get() { ThemeMode::Dark } else { ThemeMode::Light }),
        move || settings(dark),
    )
}

fn main() -> kivact::Result<()> {
    let mut app = App::new("設定").theme(theme(ThemeMode::Light));
    for font in fonts() {
        app = app.font(font);
    }
    app.run(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, render_with};

    fn options() -> RenderOptions {
        RenderOptions::default().window_size(1000.0, 800.0).theme(theme(ThemeMode::Light))
    }

    #[test]
    fn saving_shows_a_toast_and_clears_the_badge() {
        let screen = render_with(options(), root);
        assert!(screen.query_by_text("未保存の変更").is_none());
        screen.get_by_role(Role::TextBox, "表示名").type_text("さん");
        screen.get_by_text("未保存の変更");
        screen.get_by_role(Role::Button, "保存").click();
        screen.get_by_role(Role::Status, "設定を保存しました");
        screen.get_by_text("保存済みの名前: Adaさん");
        assert!(screen.query_by_text("未保存の変更").is_none());
    }

    #[test]
    fn the_switch_changes_the_theme_and_keeps_the_input() {
        let screen = render_with(options(), root);
        let top_left = |screen: &kivact::test::Screen| {
            screen.screenshot().pixel(4, 4).expect("inside the frame")
        };
        assert_eq!(top_left(&screen) & 0xff, 0xff, "light page");
        screen.get_by_role(Role::Switch, "ダークモード").click();
        assert!(top_left(&screen) & 0xff < 0x20, "dark page");
        assert_eq!(
            screen.get_by_role(Role::TextBox, "表示名").value().as_deref(),
            Some("Ada")
        );
    }

    #[test]
    fn deleting_asks_first() {
        let screen = render_with(options(), root);
        screen.get_by_role(Role::Button, "アカウントを削除").click();
        screen.get_by_role(Role::AlertDialog, "アカウントを削除しますか？");
        screen.get_by_role(Role::Button, "キャンセル").click();
        assert!(screen.query_by_role(Role::AlertDialog, "アカウントを削除しますか？").is_none());
        assert!(screen.query_by_role(Role::Status, "アカウントを削除しました").is_none());

        screen.get_by_role(Role::Button, "アカウントを削除").click();
        screen.get_by_role(Role::Button, "削除").click();
        screen.get_by_role(Role::Status, "アカウントを削除しました");
    }
}
```

`cargo run` で実行します。ステップ 2 と同じ 2 枚のカードに加え、右上に「ダークモード」のスイッチ、下に赤みを帯びた「危険な操作」の枠が表示されます。次のことを試してください。

1. 「ダークモード」を切り替えます。ページ、カード、コントロールがすべて一度に暗くなります。入力した文字は残ります。作り直しは行われず、色だけが変わるからです。
2. 名前を変えて「保存」を押します。右下にトースト「設定を保存しました」が現れ、数秒後に消えます。
3. 「アカウントを削除」を押します。「キャンセル」と「削除」を持つダイアログが開き、背後のページは操作できなくなります。ダイアログの外側をクリックしても閉じません。アラートダイアログは、はっきりした返答を求めるからです。閉じるのは「キャンセル」か Escape キーです。「キャンセル」を押してから、もう一度「アカウントを削除」と「削除」を押すと、ダイアログが閉じ、トーストに「アカウントを削除しました」と表示されます。

## 仕組み

- **テーマの切り替え。** `provide_theme(theme, child)` は、その内側の部分木にテーマを与えます。第 1 引数は `dark` を読むクロージャなので、`dark` が変わるたびにテーマが作り直され、すべてのスタイルが色を解決し直します。コンポーネントは再マウントされないので、状態を保ちます。テーマが変わりうるときは、固定の `Theme` ではなく、このようなクロージャを渡します。
- **トースト。** `toast_region(toaster())` は一度だけマウントします。トーストが現れる場所です。以降は、どのハンドラからでも `toaster().push(Toast::new(title))` と呼べます。`.tone(..)` でアイコンと緊急度を選び、`.body(..)` で 2 行目を足します。トーストは 5 秒で自動的に閉じ、ポインターが乗っている間は止まり、閉じるボタンも持ちます。
- **アラートダイアログ。** `dialog(title, open)` は標準のダイアログで、`.mode(DialogMode::AlertModal)` にするとキットのアラートダイアログになります。`open` はふつうの `signal(bool)` です。ボタンが `true` にし、`.footer(..)` のボタンが `false` にします。`.closable(false)` は、アラートダイアログが持たない閉じるボタンを取り除きます。
- **alert とアラートダイアログの違い。** `basic::alert(..)` はページに置く静的な囲み（「危険な操作」の枠）です。ダイアログは、操作を中断させるポップアップです。

## テスト

上のブロックの末尾に `#[cfg(test)] mod tests` があります。このテストは `cargo test` で動きます（ステップ 1 で足した `test` feature を使います）。`kivact::test::render_with(options, root)` は、画面を実際のウィンドウにマウントし、仮想の時計で動かします。レイアウトとアクセシビリティツリーはアプリと同じです。要素は、ユーザーやスクリーンリーダーと同じ見方、つまりロールと名前（`get_by_role(Role::Button, "保存")`、`get_by_text(..)`）で探し、実際の入力（`click`、`type_text`）で操作します。テストは `main` を通らないので、テーマはオプションで渡す必要があります（`theme: theme(ThemeMode::Light)`）。

`cargo test` を実行します。3 つのテストが成功します。それぞれウィンドウを開くため、最初の 2 つは少し時間がかかります。

## 次へ

空のウィンドウから、状態とフィードバックとテストを備えたテーマ付きの画面まで、一通りの道のりを作りました。続いて、[how-to ガイド](../how-to-guide/README.md)で個別の作業の進め方を確認してください。たとえば[テーマとトークン](../how-to-guide/theme-and-tokens.md)や[オーバーレイ](../how-to-guide/overlays.md)があります。キットのすべてのコンポーネントを一度に見るには、リポジトリのクローンで `cargo run -p kivact-ui-basic-catalog` を実行します。ほかのキットは[キットの一覧](../../README.md)にあります。
