# フォームを作る: field、検証、ピッカー、ワンタイムコード

[English](../../../en/basic/how-to-guide/forms.md)

このガイドでは、キットでフォームを作る方法を説明します。field にラベルを付けて検証し、field をまとめ、入力欄にアイコンやボタンを足し、標準セットにないピッカー（コンボボックス、日付ピッカー、カレンダー、ワンタイムコード）を使います。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

ふつうのコントロール（`text_input`、`text_area`、`checkbox`、`switch`、`radio_group`、`select`、`slider`）は標準コンポーネントで、テーマが Lyra の見た目を与えます。どのコントロールも、アプリの持つ signal を編集します。フォームが「送信」されるのは、アプリのコードがそれらの signal を読むとき、たいていはボタンの `on_press` の中です。

## field にラベルを付けて検証する

`field(label, control)` は、見えるラベルとコントロールを組にします。`.hint(..)` は下に補足の文を足し、`.error(..)` は値が `Some` の間メッセージを表示します。エラーは、`memo` で signal から計算します。

```rust,no_run
use kivact::prelude::*;

fn email_field() -> impl IntoView {
    let email = signal(String::from("ada@"));
    let error = memo(move || {
        let email = email.get();
        (!email.contains("@example.com")).then(|| "Use an @example.com address.".to_string())
    });
    field("Email", text_input("Email", email))
        .hint("We never share it.")
        .error(error)
        .style(style().width(320))
}
```

エラーが `Some` の間は、ラベルが赤くなり、コントロールが invalid として扱われ、メッセージがアラートとして読み上げられます。ラベルはコントロールの accessible name になるので、両方に同じ語を書きます（`field("Email", text_input("Email", ..))`）。ラベルを押すと、コントロールにフォーカスが移るか、切り替わります。

送信を止めるには、同じ signal から `can_submit` の memo を導き、ボタンに使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn sign_up() -> impl IntoView {
    let email = signal(String::new());
    let accepted = signal(false);
    let created = signal(false);
    let error = memo(move || {
        let email = email.get();
        (!email.is_empty() && !email.contains('@')).then(|| "Enter an email address.".to_string())
    });
    let ready = memo(move || !email.get().is_empty() && error.get().is_none() && accepted.get());
    basic::field_group((
        field("Email", text_input("Email", email)).error(error),
        checkbox("I accept the terms", accepted),
        button("Create account")
            .disabled(move || !ready.get())
            .on_press(move || created.set(true)),
        basic::muted(move || {
            if created.get() {
                "Account created."
            } else {
                ""
            }
        }),
    ))
    .style(style().width(320))
}
```

## field をセットにまとめる

`basic::field_group((..))` は、キットの余白で field を縦に並べます。`basic::field_set(..)` は凡例と説明つきのグループで、`basic::field_separator()` は field の間に線を引きます。`.text("Or")` を付けると、線にラベルが付きます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn address() -> impl IntoView {
    let (name, city) = (signal(String::new()), signal(String::new()));
    basic::field_set(basic::field_group((
        field("Name", text_input("Name", name)),
        basic::field_separator().text("Or"),
        field("City", text_input("City", city)).hint("Where you live."),
    )))
    .legend("Address")
    .description("Used to ship your order.")
    .style(style().width(320))
}
```

スイッチのように自分のラベルを持つコントロールは、文字の隣にあるほうが読みやすくなります。field を行にし、コントロール自身のラベルは隠し（field が表示します）、子の位置を揃えます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::Direction;

fn marketing() -> impl IntoView {
    let on = signal(true);
    field(
        "Marketing emails",
        switch("Marketing emails", on).hide_label(true),
    )
    .hint("Receive news and offers.")
    .style(
        style()
            .direction(Direction::Row)
            .align_items(Align::Center)
            .gap(12)
            .width(420),
    )
}
```

## 入力欄の中にアイコン、文字、ボタンを置く

`kivact::ui::input_group("Input group", (input,))` は、入力欄を共通の枠で包み、前後に付け足しを置けるようにします。`.start(..)` と `.end(..)` は view を取り、付け足しを押すと入力欄にフォーカスが移ります。文字には `kivact::ui::input_addon(..)`、ボタンには `kivact::ui::button(..).variant(kivact::ui::ButtonVariant::Ghost).variant(basic::InputGroupButtonSize::Xs).style(style().margin_right(-4.8))` を使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn search_and_copy() -> impl IntoView {
    let query = signal(String::new());
    let url = signal(String::from("https://example.com"));
    column((
        kivact::ui::input_group(
            "Search",
            (kivact::ui::text_input("Search", query)
                .variant(basic::InputGroupControl)
                .placeholder("Search..."),),
        )
        .start(icon("search"))
        .end(kivact::ui::input_addon("12 results"))
        .style(style().width(320)),
        kivact::ui::input_group(
            "Link",
            (kivact::ui::text_input("Link", url).variant(basic::InputGroupControl),),
        )
        .end(
            kivact::ui::button("Copy")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .variant(basic::InputGroupButtonSize::Xs)
                .style(style().margin_right(-4.8))
                .icon(icon("copy"))
                .on_press(|| {}),
        )
        .style(style().width(320)),
    ))
    .style(style().gap(12))
}
```

- `kivact::ui::text_area(label, signal).variant(basic::InputGroupControl)` は、入力欄の代わりに複数行の文字を受け取ります。`.block_start(..)` と `.block_end(..)` は、その上下に付け足し（文字数のカウンターなど）を置きます。
- `.invalid(true)` は、グループ全体を invalid にします。

## 長い一覧から 1 つまたは複数を選ぶ

`kivact::ui::combobox("Framework", items, selected)` は、絞り込みつきの一覧を持つ入力欄です。項目は `ListItem` です。選択の型がモードを決めます。`signal(None)`（`Option<K>`）は 1 つ、`signal(Vec::new())` は複数を選びます。`.chips()` で外せるチップを表示します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn frameworks() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    let one = signal(None);
    let many = signal(vec!["next"]);
    column((
        kivact::ui::combobox("Framework", items.clone(), one)
            .placeholder("Select a framework")
            .name("Framework")
            .show_clear(true)
            .style(style().width(260)),
        kivact::ui::combobox("Frameworks", items, many)
            .chips()
            .placeholder("Add framework")
            .name("Frameworks")
            .style(style().width(360)),
    ))
    .style(style().gap(12))
}
```

- `.name(..)` を付けてください。コンボボックス自身はラベルを持たないので、これが accessible name になります。ラベルを表示したいときは `field` で包みます。
- `.empty_text("No frameworks found.")` は、絞り込みで何もなかったときの文言を置き換え、`.disabled(true)` は無効にします。
- 短い一覧なら、標準の `select("Language", items, signal)` のほうが簡単です。`basic::native_select(items, signal)` は、ブラウザーの素の select のように枠だけで描く select です。

## 日付や期間を選ぶ

`kivact::ui::date_input(label, date).attach(button)` は、自分のボタンからカレンダーを開きます。表示文字、空のときの案内、書式はボタンで指定します。期間は `date_range_input` で同じように編集します。

```rust,no_run
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn dates() -> impl IntoView {
    let due = signal(Some(CalendarDate::new(2026, 10, 3)));
    let stay = signal(None::<DateRange>);
    column((
        kivact::ui::date_input("Due date", due)
            .presentation(kivact::ui::DatePresentation::Desktop)
            .variant(basic::DatePickerVariant::Flush)
            .calendar(|c| c.variable_rows(true))
            .attach(
                button(move || {
                    due.get().map_or_else(
                        || "Pick a due date".to_owned(),
                        |d| format!("{:04}/{:02}/{:02}", d.year(), d.month(), d.day()),
                    )
                })
                .name("Due date")
                .icon(icon("calendar"))
                .variant(kivact::ui::ButtonVariant::Outline)
                .style(style().width(280)),
            ),
        kivact::ui::date_range_input("Stay", stay)
            .presentation(kivact::ui::DatePresentation::Desktop)
            .variant(basic::DatePickerVariant::Flush)
            .calendar(|c| c.number_of_months(2).variable_rows(true))
            .attach(
                button("Choose stay")
                    .name("Stay")
                    .variant(kivact::ui::ButtonVariant::Outline)
                    .style(style().width(300)),
            ),
    ))
    .style(style().gap(12))
}
```

`.calendar(|c| ..)` は、ポップアップの中のカレンダーを設定します。常に見えているカレンダーには、`kivact::ui::calendar(signal).variable_rows(true)` か `kivact::ui::calendar(signal).variable_rows(true)` を使います。どちらも同じオプションを取ります。

```rust,no_run
use kivact::i18n::{CalendarMonth, Weekday};
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn weekdays_only() -> impl IntoView {
    kivact::ui::calendar(signal(None::<kivact::i18n::CalendarDate>))
        .variable_rows(true)
        .default_month(CalendarMonth::new(2026, 10))
        .week_start(Weekday::Monday)
        .disabled_dates(|d| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
        .caption(kivact::ui::CalendarCaption::Dropdown)
}
```

月・曜日の名前と日付書式は現在の `Locale` に従います。操作ラベルは `calendar.*` のメッセージです。ボタン内では `Locale::format_date` またはアプリの書式関数を使えます。

## ワンタイムコードを求める

`basic::input_otp(label, code, length)` は、1 文字につき 1 つのセルを表示し、`String` を編集します。ユーザーは 1 文字ずつ入力することも、コード全体を貼り付けることもできます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn verify() -> impl IntoView {
    let code = signal(String::new());
    let done = signal(String::new());
    column((
        basic::input_otp("Verification code", code, 6)
            .groups(&[3, 3])
            .pattern(basic::OtpPattern::Digits)
            .on_complete(move |code: String| done.set(code)),
        basic::muted(move || format!("Entered: {}", done.get())),
    ))
    .style(style().gap(8))
}
```

`.groups(&[3, 3])` はセルを区切りで分け、`.pattern(..)` は使える文字を制限し（`Digits` が既定で、`Alphanumeric` は英字も許します）、`.on_complete(..)` は最後のセルが埋まったときに実行されます。コードの検証はアプリの仕事です。

## 関連項目

- [オーバーレイ](overlays.md): フォームをダイアログやシートに入れる。
- [ナビゲーション](navigation.md): ページの間のタブとウィザード。
- rustdoc: 入力グループ、コンボボックス、ピッカー、`input_otp` は `cargo doc -p kivact-ui-basic --open`、`field`、`select`、`text_input` は `cargo doc -p kivact-ui --open`。

枠のない入力は `.variant(basic::InputGroupControl)` を使い、実際のグループからフォーカス・不正状態・上下の余白を読みます。付属ボタンは通常の `button` に `InputGroupButtonSize` の variant を指定します。`IconXs` / `IconSm` は `.hide_label(true)` も指定します。リアクティブな寸法は `InputGroupButtonLook::default().size(size)` で、文字の表示は標準の `.hide_label(..)` で制御します。
