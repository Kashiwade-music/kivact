# ステップ 3: 検証して送信する

[English](../../../en/pop/getting-started/03-validate-and-submit.md)

このステップでは、日付欄を 2 つ足し、エラーが必要な欄の下にメッセージを出し、入力が正しいときに成功メッセージを表示します。最後に、ユーザーのようにフォームを埋めるテストを書きます。

## プログラム

`src/main.rs` を、テストを含む完全なプログラムに置き換えます。

```rust,test_harness
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::ui::Tone;
use kivact_ui_pop as pop;

fn sign_up(today: CalendarDate) -> impl IntoView {
    let name = signal(String::new());
    let email = signal(String::new());
    let plan = signal(Some("team"));
    let birth = signal(CalendarDate::new(2000, 1, 1));
    let start = signal(today);
    let birth_valid = signal(true);
    let start_valid = signal(true);
    let terms = signal(false);
    let attempted = signal(false);
    let created = signal(false);

    let name_missing = memo(move || name.with(|n| n.trim().is_empty()));
    let email_missing = memo(move || email.with(|e| !e.contains('@')));
    let name_error = memo(move || {
        (attempted.get() && name_missing.get()).then(|| "名前を入力してください".to_owned())
    });
    let email_error = memo(move || {
        (attempted.get() && email_missing.get())
            .then(|| "メールアドレスを入力してください".to_owned())
    });
    let birth_error = memo(move || {
        (birth_valid.get() && birth.get() >= today)
            .then(|| "生年月日は過去の日付にしてください".to_owned())
    });
    let start_error = memo(move || {
        (start_valid.get() && start.get() < today)
            .then(|| "今日以降の日付を選んでください".to_owned())
    });
    let ready = memo(move || {
        !name_missing.get()
            && !email_missing.get()
            && birth_valid.get()
            && birth_error.get().is_none()
            && start_valid.get()
            && start_error.get().is_none()
            && terms.get()
    });

    pop::panel((
        text("新規アカウント").style(style().font_size(20).weight(600)),
        field("名前", text_input("名前", name).placeholder("お名前を入力"))
            .hint("請求書に表示されます。")
            .error(name_error),
        field(
            "メールアドレス",
            text_input("メールアドレス", email).placeholder("you@example.com"),
        )
        .error(email_error),
        field(
            "プラン",
            radio_group(
                "プラン",
                vec![
                    ListItem::new("solo", "ソロ"),
                    ListItem::new("team", "チーム"),
                    ListItem::new("business", "ビジネス"),
                ],
                plan,
            )
            .orientation(Orientation::Horizontal)
            .variant(pop::Segmented),
        ),
        pop::date_input("生年月日", birth, today).valid(birth_valid),
        show(move || birth_error.get().is_some(), move || {
            text(move || birth_error.get().unwrap_or_default())
                .style(style().font_size(12).color(ColorToken::Danger))
        }),
        pop::date_input("開始日", start, today).valid(start_valid),
        show(move || start_error.get().is_some(), move || {
            text(move || start_error.get().unwrap_or_default())
                .style(style().font_size(12).color(ColorToken::Danger))
        }),
        checkbox("利用規約に同意します", terms),
        show(move || attempted.get() && !terms.get(), || {
            text("続けるには利用規約への同意が必要です")
                .style(style().font_size(12).color(ColorToken::Danger))
        }),
        row((
            button("キャンセル").variant(pop::ButtonTone::Soft),
            button("アカウントを作成")
                .variant(pop::ButtonTone::Accent)
                .disabled(created)
                .on_press(move || {
                    attempted.set(true);
                    if ready.get() {
                        created.set(true);
                    }
                }),
        ))
        .style(style().gap(8).justify_content(Justify::End)),
        show(created, move || {
            pop::alert("アカウントを作成しました")
                .tone(Tone::Success)
                .description(move || format!("ようこそ、{}さん。", name.get()))
        }),
    ))
    .style(style().width(420).margin(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("New account")
        .theme(pop::theme(ThemeMode::Light))
        .run(|| sign_up(CalendarDate::today()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, Screen, render_with};
    use kivact::view::Modifiers;

    fn mount() -> Screen {
        let options = RenderOptions {
            theme: pop::theme(ThemeMode::Light),
            height: 1000.0,
            ..RenderOptions::default()
        };
        render_with(options, || sign_up(CalendarDate::new(2026, 11, 1)))
    }

    #[test]
    fn a_complete_form_creates_the_account() {
        let screen = mount();
        screen.get_by_role(Role::TextBox, "名前").type_text("Haruka");
        screen
            .get_by_role(Role::TextBox, "メールアドレス")
            .type_text("haruka@example.com");
        screen
            .get_by_role(Role::CheckBox, "利用規約に同意します")
            .click();
        screen.get_by_role(Role::Button, "アカウントを作成").click();
        screen.get_by_text("ようこそ、Harukaさん。");
        assert!(screen.get_by_role(Role::Button, "アカウントを作成").is_disabled());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn an_empty_form_shows_what_is_missing() {
        let screen = mount();
        screen.get_by_role(Role::Button, "アカウントを作成").click();
        screen.get_by_text("名前を入力してください");
        screen.get_by_text("メールアドレスを入力してください");
        screen.get_by_text("続けるには利用規約への同意が必要です");
        assert!(screen.query_by_text("アカウントを作成しました").is_none());
    }

    #[test]
    fn a_start_date_in_the_past_is_refused() {
        let screen = mount();
        let start = screen.get_by_role(Role::TextBox, "開始日");
        start.press_key_with(
            Key::Char('a'),
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );
        start.type_text("2026-10-01");
        screen.get_by_text("今日以降の日付を選んでください");
    }
}
```

`cargo run` で起動し、空のフォームで「アカウントを作成」を押してください。名前欄とメール欄の下、そしてチェックボックスの近くに赤いメッセージが出ます。フォームを埋めてもう一度押すと「アカウントを作成しました」というメッセージが現れ、ボタンが無効になります。日付欄に `2000-02-30` と入力すると、欄が赤くなり、その日付は存在しないと表示されます。

`cargo test` を実行します。3 つのテストが実際のウィンドウを開き、擬似入力で操作して、画面に出ているものを確かめます。

## 仕組み

**検証はアプリが持ちます。** それぞれの `memo` が、シグナルから値を導き出します。`memo` は、読んだシグナルが変わったときだけ再計算されます。`.error(name_error)` は `Option<String>` を受け取り、問題があるあいだ `Some` になります。すると欄はコントロールの下にメッセージを出し、コントロールを無効な状態として印を付け、スクリーンリーダー向けにメッセージを関連付けます。名前とメールのエラーは `attempted` も読むので、ユーザーが一度ボタンを押すまで静かなままです。

**`pop::date_input(label, value, today)`** は、カレンダーのポップアップが付いた編集可能な日付欄です。テキストは `YYYY-MM-DD` の形式で、シグナル（`birth`）は、テキストが完全で実在する日付になったときだけ変わります。入力途中の文字が状態に届くことはありません。`today` は呼び出し側が渡します。そのためコンポーネントの動きが予測しやすく、テストもしやすくなります。プログラムは `CalendarDate::today()` を渡し、テストは固定の日付を渡します。

**`.valid(signal)`** は、テキストが実在する日付のあいだ `true`、そうでなければ `false` を書き込みます。コンポーネントは自前の形式エラー（"Use YYYY-MM-DD" や "This date does not exist"）を表示します。「過去であること」のような業務ルールはアプリの責任です。`birth_error` と `start_error` は、テキストが有効なときだけ値を検査するので、2 つのメッセージが重なることはありません。

**`show(condition, || view)`** は、条件が真のあいだだけビューを配置します。`Tone::Success` は `pop::alert` を緑にします。アラートなどの表示コンポーネントは、意味を標準の `kivact::ui::Tone` で受け取ります。

**テスト**は、アプリと同じテーマを `RenderOptions` で渡して同じ見た目で描き、ロールとアクセシブルな名前、つまりスクリーンリーダーが読み上げる言葉で要素を探します。日付欄のテキストを置き換えるには、Ctrl+A で全選択してから入力します。

## できあがったもの

Pop の見た目を持ち、型付きの状態、欄ごとのエラー、本物の日付入力、そして合格するテストを備えたフォームができました。さらに進むには:

- [Pop のテーマとトークンを設定する](../how-to-guide/theme-and-tokens.md): ダークモードや色の変更。
- [日付と期間を選ばせる](../how-to-guide/dates-and-calendars.md): 期間、単体のカレンダー、イベントの点。

次へ: [Pop 作業別ガイド](../how-to-guide/README.md)
