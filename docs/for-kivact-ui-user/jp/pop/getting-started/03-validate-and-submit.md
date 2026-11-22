# ステップ 3: 検証して送信する

[English](../../../en/pop/getting-started/03-validate-and-submit.md)

このステップでは、日付欄を 2 つ足し、エラーが必要な欄の下にメッセージを出し、入力が正しいときに成功メッセージを表示します。最後に、ユーザーのようにフォームを埋めるテストを書きます。

## プログラム

`src/main.rs` を、テストを含む完全なプログラムに置き換えます。

```rust,test_harness
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::interaction::Tone;
use kivact_ui_pop as pop;

#[component]
fn sign_up(today: CalendarDate) -> impl Element {
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
        (attempted.get() && name_missing.get()).then(|| "Enter your name".to_owned())
    });
    let email_error = memo(move || {
        (attempted.get() && email_missing.get()).then(|| "Enter an email address".to_owned())
    });
    let birth_error = memo(move || {
        (birth_valid.get() && birth.get() >= today)
            .then(|| "Birth date must be in the past".to_owned())
    });
    let start_error = memo(move || {
        (start_valid.get() && start.get() < today)
            .then(|| "Choose today or a later date".to_owned())
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

    kivact::ui::card((
        text("New account").style(style().font_size(20).font_weight(600)),
        field("Name", text_input(name).placeholder("Your full name"))
            .hint("Shown on your invoices.")
            .error(name_error),
        field(
            "Email",
            text_input(email).label("Email").placeholder("you@example.com"),
        )
        .error(email_error),
        field(
            "Plan",
            radio_group(
                "Plan",
                vec![
                    ListItem::new("solo", "Solo"),
                    ListItem::new("team", "Team"),
                    ListItem::new("business", "Business"),
                ],
                plan,
            )
            .orientation(kivact::view::Orientation::Horizontal)
            .variant(kivact::ui::RadioGroupVariant::Segmented),
        ),
        { let value = kivact::state::binding(move || Some(birth.get()), move |day| { if let Some(day) = day { birth.set(day); } }); kivact::ui::date_input(value).label("Birth date").calendar(move |calendar| calendar.today(today).month_title(|date| format!("{}-{:02}",date.year(),date.month()))) }.valid(birth_valid),
        show(move || birth_error.get().is_some(), move || {
            text(move || birth_error.get().unwrap_or_default())
                .style(style().font_size(12).color(kivact::style::color::DANGER))
        }),
        { let value = kivact::state::binding(move || Some(start.get()), move |day| { if let Some(day) = day { start.set(day); } }); kivact::ui::date_input(value).label("Start date").calendar(move |calendar| calendar.today(today).month_title(|date| format!("{}-{:02}",date.year(),date.month()))) }.valid(start_valid),
        show(move || start_error.get().is_some(), move || {
            text(move || start_error.get().unwrap_or_default())
                .style(style().font_size(12).color(kivact::style::color::DANGER))
        }),
        checkbox("I accept the terms", terms),
        show(move || attempted.get() && !terms.get(), || {
            text("Accept the terms to continue")
                .style(style().font_size(12).color(kivact::style::color::DANGER))
        }),
        row((
            button("Cancel").variant(pop::ButtonTone::Soft),
            button("Create account")
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
            {let tone=kivact::state::IntoValue::into_value(Tone::Success); kivact::ui::alert("Account created").tone(tone).icon(kivact::view::icon(move || kivact::style::Icon::named(match tone.get() { kivact::interaction::Tone::Danger=>"alert-circle", kivact::interaction::Tone::Warning=>"alert-triangle",kivact::interaction::Tone::Success=>"circle-check",_=>"info-circle"}))).description(move || format!("Welcome, {}.", name.get()))}
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
        let options = RenderOptions::default().window_size(800., 1000.0).theme(pop::theme(ThemeMode::Light));
        render_with(options, || sign_up(CalendarDate::new(2026, 11, 1)))
    }

    #[test]
    fn a_complete_form_creates_the_account() {
        let screen = mount();
        screen.get_by_role(Role::TextBox, "Name").type_text("Haruka");
        screen
            .get_by_role(Role::TextBox, "Email")
            .type_text("haruka@example.com");
        screen
            .get_by_role(Role::CheckBox, "I accept the terms")
            .click();
        screen.get_by_role(Role::Button, "Create account").click();
        screen.get_by_text("Welcome, Haruka.");
        assert!(screen.get_by_role(Role::Button, "Create account").is_disabled());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn an_empty_form_shows_what_is_missing() {
        let screen = mount();
        screen.get_by_role(Role::Button, "Create account").click();
        screen.get_by_text("Enter your name");
        screen.get_by_text("Enter an email address");
        screen.get_by_text("Accept the terms to continue");
        assert!(screen.query_by_text("Account created").is_none());
    }

    #[test]
    fn a_start_date_in_the_past_is_refused() {
        let screen = mount();
        let start = screen.get_by_role(Role::TextBox, "Start date");
        start.press_key_with(
            Key::Char('a'),
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );
        start.type_text("2026-10-01");
        start.press_key(Key::Enter);
        screen.get_by_text("Choose today or a later date");
    }
}
```

`cargo run` で起動し、空のフォームで「アカウントを作成」を押してください。名前欄とメール欄の下、そしてチェックボックスの近くに赤いメッセージが出ます。フォームを埋めてもう一度押すと「アカウントを作成しました」というメッセージが現れ、ボタンが無効になります。日付欄に `2000-02-30` と入力すると、欄が赤くなり、その日付は存在しないと表示されます。

`cargo test` を実行します。3 つのテストが実際のウィンドウを開き、擬似入力で操作して、画面に出ているものを確かめます。

## 仕組み

**検証はアプリが持ちます。** それぞれの `memo` が、シグナルから値を導き出します。`memo` は、読んだシグナルが変わったときだけ再計算されます。`.error(name_error)` は `Option<String>` を受け取り、問題があるあいだ `Some` になります。すると欄はコントロールの下にメッセージを出し、コントロールを無効な状態として印を付け、スクリーンリーダー向けにメッセージを関連付けます。名前とメールのエラーは `attempted` も読むので、ユーザーが一度ボタンを押すまで静かなままです。

**`kivact::ui::date_input(label, optional_value)`** は、カレンダーのポップアップが付いた編集可能な日付欄です。テキストは `YYYY-MM-DD` の形式で、シグナル（`birth`）は、完全で実在する日付を Enter またはフォーカス移動で確定したときだけ変わります。入力途中の文字が状態に届くことはありません。`.calendar(|calendar| calendar.today(today))` は呼び出し側が指定します。そのためコンポーネントの動きが予測しやすく、テストもしやすくなります。プログラムは `CalendarDate::today()` を渡し、テストは固定の日付を渡します。

**`.valid(signal)`** は、テキストが実在する日付のあいだ `true`、そうでなければ `false` を書き込みます。コンポーネントは自前の形式エラー（"Use YYYY-MM-DD" や "This date does not exist"）を表示します。「過去であること」のような業務ルールはアプリの責任です。`birth_error` と `start_error` は、テキストが有効なときだけ値を検査するので、2 つのメッセージが重なることはありません。

**`show(condition, || view)`** は、条件が真のあいだだけビューを配置します。`Tone::Success` は `kivact::ui::alert` を緑にします。アラートなどの表示コンポーネントは、意味を標準の `kivact::interaction::Tone` で受け取ります。

**テスト**は、アプリと同じテーマを `RenderOptions` で渡して同じ見た目で描き、ロールとアクセシブルな名前、つまりスクリーンリーダーが読み上げる言葉で要素を探します。日付欄のテキストを置き換えるには、Ctrl+A で全選択してから入力します。

## できあがったもの

Pop の見た目を持ち、型付きの状態、欄ごとのエラー、本物の日付入力、そして合格するテストを備えたフォームができました。さらに進むには:

- [Pop のテーマとトークンを設定する](../how-to-guide/theme-and-tokens.md): ダークモードや色の変更。
- [日付と期間を選ばせる](../how-to-guide/dates-and-calendars.md): 期間、単体のカレンダー、イベントの点。

次へ: [Pop 作業別ガイド](../how-to-guide/README.md)
