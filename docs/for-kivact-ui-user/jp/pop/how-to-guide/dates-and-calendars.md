# 日付と期間を選ばせる

[English](../../../en/pop/how-to-guide/dates-and-calendars.md)

日付、日付の期間、カレンダー上の 1 日を、入力または選択で受け取るための手引きです。3 つのコンポーネントはどれも値をシグナルで受け取るので、日付はアプリの状態に残ります。

## 1 つの日付を尋ねる

`pop::date_input(label, value, today)` は、カレンダーのポップアップが付いたテキスト欄です。`value` は `Signal<CalendarDate>`（または同じ型のバインディング）で、`today` は呼び出し側が渡します。

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn due_date() -> impl IntoView {
    let today = CalendarDate::today();
    let due = signal(today.add_days(7));
    let valid = signal(true);
    column((
        pop::date_input("Due date", due, today).valid(valid),
        button("Save")
            .disabled(move || !valid.get())
            .on_press(move || println!("due {}", due.get())),
    ))
    .style(style().gap(12).padding(24))
}
# fn main() {}
```

ラベルはスクリーンリーダー向けにテキスト欄の名前になります。見える形で出したいときは、自分で描いてください（`field("Due date", ..)` で包むか、上に `text` を置きます）。カレンダーを開くボタンの名前は "Choose a date" です。

テキスト欄の動きは次のとおりです。

- 日付を `YYYY-MM-DD` で表示します。`YYYY/MM/DD` でも入力できます。年は 4 桁、月と日は 2 桁です。
- シグナルが変わるのは、テキストが実在する完全な日付になったときだけです。`2027-1` と入力している途中は、状態は直前の日付のままです。
- 日付ではないテキストは、欄を赤くして "Use YYYY-MM-DD" を表示します。`2027-02-30` のように実在しない日付なら "This date does not exist" です。日付の書きかけは、欄からフォーカスが外れるか Enter が押されるまでエラーになりません。
- `.valid(signal)` は、テキストが日付のあいだ `true`、そうでなければ `false` を書き込みます。保存を止めたいときに使ってください。コンポーネント自身は何も止めません。
- カレンダーで日を選ぶと、テキストとシグナルが書き換わり、ポップアップが閉じます。Escape か外側のクリックで、変更せずに閉じます。

データに依存するルール（過去の日付、週末、予約済みの日）はアプリの責任です。[チュートリアル](../getting-started/03-validate-and-submit.md) のように、シグナルを検査して入力欄のそばにエラーを出してください。ルールを検査するのは `valid` が `true` のときだけにします。シグナルは直前の正しい日付を保持しているためです。

### 日付を任意にする

日付入力は常に日付を持ちます。「日付なし」を表したいときは、そばにスイッチを置いて、任意の値を導出します。

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn deadline() -> impl IntoView {
    let today = CalendarDate::today();
    let enabled = signal(false);
    let date = signal(today);
    let deadline = memo(move || enabled.get().then(|| date.get()));
    column((
        checkbox("Set a deadline", enabled),
        show(enabled, move || pop::date_input("Deadline", date, today)),
        text(move || match deadline.get() {
            Some(day) => format!("Deadline: {day}"),
            None => "No deadline".to_owned(),
        }),
    ))
    .style(style().gap(12).padding(24))
}
# fn main() {}
```

## 期間を尋ねる

`pop::date_range_input((start_label, end_label), (start, end), today)` は、1 つのカレンダーを共有する 2 つの欄を表示します。

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn stay() -> impl IntoView {
    let today = CalendarDate::today();
    let from = signal(today);
    let to = signal(today.add_days(2));
    let valid = signal(true);
    column((
        pop::date_range_input(("Check-in", "Check-out"), (from, to), today).valid(valid),
        text(move || format!("{} nights", to.get().to_epoch_days() - from.get().to_epoch_days())),
    ))
    .style(style().gap(12).padding(24))
}
# fn main() {}
```

各欄は上のルールに従います。カレンダーでは、最初に選んだ日が開始日と終了日の両方になり、終了日の選択を待ちます。2 回目の選択は終了日になり、開始日より前なら 2 つを入れ替えて、ポップアップを閉じます。期間は色付きで表示されます。開始日が終了日より後ろになるように入力すると、2 つの欄が赤くなり、"The end date is before the start date" が表示され、`valid` は `false` になります。ポップアップが開いているあいだは "Pick a start date" と "Pick an end date" が丁寧に読み上げられ、Close ボタンを押すと開始日の欄にフォーカスが戻ります。

## カレンダーだけを表示する

`pop::calendar(selected, today)` は、テキスト欄のない月間グリッドです。スケジュールや予約の画面に使います。多くの用途は次のビルダーでまかなえます。

```rust,no_run
use kivact::i18n::{CalendarDate, Weekday};
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn schedule() -> impl IntoView {
    let today = CalendarDate::today();
    let day = signal(today);
    let events = vec![
        pop::CalendarEvent::new(today, rgb(0x2b8bd8)),
        pop::CalendarEvent::new(today.add_days(3), rgb(0xef4458)),
    ];
    let trip = Some((today.add_days(5), today.add_days(9)));
    column((
        pop::calendar(day, today)
            .week_starts_on(Weekday::Monday)
            .year_navigation(true)
            .events(events)
            .range(trip)
            .weekend(|weekday| match weekday {
                Weekday::Saturday | Weekday::Sunday => Some(pop::Hue::Pink),
                _ => None,
            })
            .on_activate(|date| println!("opened {date}")),
        text(move || format!("Selected: {}", day.get())),
    ))
    .style(style().gap(12).padding(24))
}
# fn main() {}
```

- `.week_starts_on(Weekday)` は最初の列を決めます。指定がなければ、アプリのロケールでの週の始まりになります。
- `.year_navigation(true)` は、月のボタンの横に前年・翌年ボタンを足します。キーボードでは Shift を押しながら PageUp / PageDown で同じ操作ができます。`.year_month_menus(true)` はタイトルを年と月のセレクトに置き換えます。年は前後 50 年分です。
- `.events(..)` は 1 日につき最大 3 つの色付きの点を描きます。カレンダーは描くだけで、イベントの中身はアプリのモデルに残ります。点を更新したいときは `Vec<pop::CalendarEvent>` のシグナルを渡します。
- `.range(Some((first, last)))` は `first` から `last` まで（両端を含む）を色付けします。
- `.weekend(..)` は曜日ごとに色を付けます。既定ではすべての日が同じ色です。
- `.on_activate(..)` は、日をクリックするか、その日で Enter を押したときに実行されます。すでに選択済みの日でも実行されます。
- `.month_title(|first_of_month| String)` はタイトルの文字を置き換えます。

真偽値や一覧を受け取るビルダーは、どれもシグナルも受け取れます。

## 言語と文言

曜日の見出し、月のボタン、"Choose a date" ボタン、日付のエラーは、差し替えるまで英語です。アプリのロケールとメッセージを設定します。キーは `pop.` で始まります。

```rust,no_run
use kivact::i18n::{CalendarDate, Locale, Messages};
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn picker() -> impl IntoView {
    let today = CalendarDate::today();
    pop::date_input("Date", signal(today), today)
}

fn main() -> kivact::Result<()> {
    let messages = Messages::new()
        .with("pop.date.open", "日付を選択")
        .with("pop.date.error.format", "YYYY-MM-DD の形式で入力してください")
        .with("pop.date.error.invalid", "存在しない日付です")
        .with("pop.date.error.order", "終了日が開始日より前です")
        .with("pop.calendar.title", "{year}.{month}");
    App::new("Dates")
        .locale(Locale::new("ja-JP"))
        .messages(messages)
        .theme(pop::theme(ThemeMode::Light))
        .run(picker)
}
```

上のキーのほかに、カレンダーは `pop.calendar.sun` から `pop.calendar.sat`（曜日の見出し）、`pop.calendar.previous`、`.next`、`.previous_year`、`.next_year`、`.year`、`.month` を読み、期間のポップアップは `pop.date.range.pick_start`、`pop.date.range.pick_end`、`pop.date.range.close` を読みます。週の始まりもロケールで決まります。

## 関連

- [Pop のテーマとトークンを設定する](theme-and-tokens.md): カレンダーとイベントの色について。
- チュートリアルの [検証して送信する](../getting-started/03-validate-and-submit.md): 日付とフィールドのエラーを組み合わせています。
- `kivact::i18n::CalendarDate`（`new`、`checked`、`today`、`add_days`、`add_months`、`weekday`）は `cargo doc -p kivact --open`、Pop の API の残りは `cargo doc -p kivact-ui-pop --open` で確認できます。
