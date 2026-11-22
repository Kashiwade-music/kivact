# フォームを作る: field、検証、ピッカー、ワンタイムコード

[English](../../../en/basic/how-to-guide/forms.md)

この手引きでは、キットを使ってフォームを作る方法を説明します。欄にラベルを付けて検証し、欄をまとめ、入力欄にアイコンやボタンを付け、標準の部品にない選択の部品（コンボボックス、日付ピッカー、カレンダー、ワンタイムコード）を使います。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

基本の部品（`text_input`、`text_area`、`checkbox`、`switch`、`radio_group`、`select`、`slider`）は標準コンポーネントで、テーマが Lyra の見た目を与えます。どの部品も、アプリが持つシグナルを編集します。フォームが「送信」されるのは、アプリのコードがそのシグナルを読むときです。たいていは、ボタンの `on_press` の中です。

## 欄にラベルを付けて検証する

`field(label, control)` は、見えるラベルとコントロールを組にします。`.hint(..)` はその下に説明を足し、`.error(..)` は値が `Some` のあいだメッセージを出します。エラーは `memo` でシグナルから計算します。

```rust,no_run
use kivact::prelude::*;

#[component]
fn email_field() -> impl Element {
    let email = signal(String::from("ada@"));
    let error = memo(move || {
        let email = email.get();
        (!email.contains("@example.com")).then(|| "Use an @example.com address.".to_string())
    });
    field("Email", text_input(email))
        .hint("We never share it.")
        .error(error)
        .style(style().width(320))
}
```

エラーが `Some` のあいだ、ラベルは赤くなり、コントロールは無効な状態として印が付き、メッセージはアラートとして読み上げられます。ラベルはコントロールのアクセシブルな名前になるので、両方に同じ語を書いてください（`field("Email", text_input(..))`）。ラベルを押すと、コントロールにフォーカスが移るか、コントロールが切り替わります。

送信を止めるには、同じシグナルから `can_submit` の memo を作り、ボタンに使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn sign_up() -> impl Element {
    let email = signal(String::new());
    let accepted = signal(false);
    let created = signal(false);
    let error = memo(move || {
        let email = email.get();
        (!email.is_empty() && !email.contains('@')).then(|| "Enter an email address.".to_string())
    });
    let ready = memo(move || !email.get().is_empty() && error.get().is_none() && accepted.get());
    basic::field_group((
        field("Email", text_input(email)).error(error),
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

## 欄をまとめる

`basic::field_group((..))` は、欄をキットの間隔で積み重ねます。`basic::field_set(..)` は、凡例と説明を持つまとまりです。`basic::field_separator()` は欄のあいだに罫線を引きます。罫線にラベルを付けるには `.text("Or")` を渡します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn address() -> impl Element {
    let (name, city) = (signal(String::new()), signal(String::new()));
    basic::field_set(basic::field_group((
        field("Name", text_input(name)),
        basic::field_separator().text("Or"),
        field("City", text_input(city)).hint("Where you live."),
    )))
    .legend("Address")
    .description("Used to ship your order.")
    .style(style().width(320))
}
```

スイッチのように、それ自体がラベルを持つコントロールは、その文字と並べたほうが読みやすくなります。欄を行にし、コントロール自身のラベルは隠して（欄がそれを表示します）、子を揃えます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::FlexDirection;

#[component]
fn marketing() -> impl Element {
    let on = signal(true);
    field(
        "Marketing emails",
        switch("Marketing emails", on).hide_label(true),
    )
    .hint("Receive news and offers.")
    .style(
        style()
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .gap(12)
            .width(420),
    )
}
```

## 入力欄の中にアイコン・文字・ボタンを置く

`kivact::ui::input_group("Input group", (input,))` は、入力欄を共有の枠で囲み、その前後に付属の部品を足せるようにします。`.start(..)` と `.end(..)` はビューを受け取ります。付属の部品を押すと、入力欄にフォーカスが移ります。普通の文字には `kivact::ui::input_addon(..)` を、ボタンには `kivact::ui::button(..).variant(kivact::ui::ButtonVariant::Ghost).variant(basic::InputGroupButtonSize::Xs).style(style().margin_right(-4.8))` を使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn search_and_copy() -> impl Element {
    let query = signal(String::new());
    let url = signal(String::from("https://example.com"));
    column((
        kivact::ui::input_group(
            "Search",
            (kivact::ui::text_input(query)
                .label("Search")
                .variant(basic::InputGroupControl)
                .placeholder("Search..."),),
        )
        .start(icon("search"))
        .end(kivact::ui::input_addon("12 results"))
        .style(style().width(320)),
        kivact::ui::input_group(
            "Link",
            (kivact::ui::text_input(url)
                .label("Link")
                .variant(basic::InputGroupControl),),
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

- `kivact::ui::text_area(signal).label("..").variant(basic::InputGroupControl)` は、複数行のテキストで入力欄の代わりになります。`.block_start(..)` と `.block_end(..)` は、その上か下に付属の部品（文字数のカウンターなど）を置きます。
- `.invalid(true)` は、グループ全体を無効な状態として印を付けます。

## 長い一覧から 1 つか複数の選択肢を選ぶ

`kivact::ui::combobox(items, selected)` は、絞り込みの付いた一覧を持つテキストボックスです。項目は `ListItem` です。選択の型がモードを決めます。`signal(None)`（`Option<K>`）は 1 つを選び、`signal(Vec::new())` は複数を選びます。取り外せるチップを出すには `.chips()` を足します。

```rust,no_run
use kivact::prelude::*;

#[component]
fn frameworks() -> impl Element {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    let one = signal(None);
    let many = signal(vec!["next"]);
    column((
        kivact::ui::combobox(items.clone(), one)
            .placeholder("Select a framework")
            .name("Framework")
            .show_clear(true)
            .style(style().width(260)),
        kivact::ui::combobox(items, many)
            .chips()
            .placeholder("Add framework")
            .name("Frameworks")
            .style(style().width(360)),
    ))
    .style(style().gap(12))
}
```

- 名前は `.name(..)` で設定します。目に見えるラベルを出すには、コンボボックスを `field` で囲みます。
- `.empty_text("No frameworks found.")` は、絞り込みに一致する項目がないときの文言を置き換えます。`.disabled(true)` で無効にできます。
- 項目が少ないなら、標準の `select(items, signal).label("Language")` のほうが簡単です。`basic::native_select(items, signal)` は、ブラウザ標準の枠の見た目の select です。

## 日付または期間を選ぶ

`kivact::ui::date_input(date).attach(button)` は、自分のボタンからカレンダーを開きます。付けたボタンが、その文字、プレースホルダー、形式を持ちます。`date_range_input` も同じ形で期間を編集します。

```rust,no_run
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn dates() -> impl Element {
    let due = signal(Some(CalendarDate::new(2026, 10, 3)));
    let stay = signal(None::<DateRange>);
    column((
        kivact::ui::date_input(due)
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
        kivact::ui::date_range_input(stay)
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

`.calendar(|c| ..)` は、ポップアップのカレンダーを設定します。いつも表示しておくカレンダーには、`kivact::ui::calendar(signal).variable_rows(true)` を使います。1 日のシグナルでも期間のシグナルでも使え、どちらも同じ設定を受け取ります。

```rust,no_run
use kivact::i18n::{CalendarMonth, Weekday};
use kivact::prelude::*;

#[component]
fn weekdays_only() -> impl Element {
    kivact::ui::calendar(signal(None::<kivact::i18n::CalendarDate>))
        .variable_rows(true)
        .default_month(CalendarMonth::new(2026, 10))
        .week_start(Weekday::Monday)
        .disabled_dates(|d| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
        .caption(kivact::ui::CalendarCaption::Dropdown)
}
```

月と曜日の名前、日付の整形は、有効な `Locale` から来ます。カレンダーの操作の文言は `calendar.*` のメッセージを使います。トリガーの中身には `Locale::format_date` かアプリの整形関数を使えます。

## ワンタイムコードを求める

`basic::input_otp(code, length)` は、文字ごとに 1 つのセルを表示し、`String` を編集します。利用者は、コード全体を入力するか貼り付けられます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn verify() -> impl Element {
    let code = signal(String::new());
    let done = signal(String::new());
    column((
        basic::input_otp(code, 6usize)
            .name("Verification code")
            .groups(vec![3_usize, 3])
            .pattern(basic::OtpPattern::Digits)
            .on_complete(move |code: String| done.set(code)),
        basic::muted(move || format!("Entered: {}", done.get())),
    ))
    .style(style().gap(8))
}
```

`.groups(vec![3_usize, 3])` は、区切りでセルを分けます。`.pattern(..)` は使える文字を制限します（既定は `Digits`、`Alphanumeric` なら英字も入ります）。`.on_complete(..)` は最後のセルが埋まったときに実行されます。認証そのものはアプリが行います。

## 関連

- [オーバーレイ](overlays.md): フォームをダイアログやシートに入れる。
- [ナビゲーション](navigation.md): 欄のページのあいだのタブやウィザード。
- リファレンス: 入力グループ、ピッカー、`input_otp` は `cargo doc -p kivact-ui-basic --open`、`combobox`、`field`、`select`、`text_input` は `cargo doc -p kivact-ui --open`。

枠のない部品は `.variant(basic::InputGroupControl)` を使い、所属するグループのフォーカス、無効の状態、ブロックの間隔を観察します。グループのボタンは、見た目の `InputGroupButtonSize` の variant を持つ普通の `button` のビルダーです。`IconXs` と `IconSm` には `.hide_label(true)` を使ってください。
