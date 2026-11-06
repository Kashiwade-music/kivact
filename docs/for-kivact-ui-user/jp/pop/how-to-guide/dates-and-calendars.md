# 日付と期間を選ぶ

[English](../../../en/pop/how-to-guide/dates-and-calendars.md)

Pop テーマを設定し、標準の `kivact::ui` のカレンダーと日付入力を使います。アプリが `Option<CalendarDate>` または `Option<DateRange>` の binding を持ち、`None` は空欄を表します。

## 日付を入力する

```rust,no_run
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;

fn dates() -> impl IntoView {
    let today = CalendarDate::today();
    let due = signal(Some(today));
    let stay = signal(Some(DateRange::new(today, today.add_days(2))));
    column((
        kivact::ui::date_input("Due date", due)
            .calendar(move |calendar| calendar.today(today)),
        kivact::ui::date_range_input("Stay", stay)
            .start_label("Check-in")
            .end_label("Check-out")
            .calendar(move |calendar| calendar.today(today).number_of_months(1usize)),
    )).style(style().gap(12))
}
```

テキスト入力は解析できた値を通知し、`.valid(binding)` は編集中の文字列の妥当性を通知します。不完全な文字列は編集中も表示されます。`.format(DateStyle)` と `.format_with(..)` で表示形式を指定します。`.min(..)`、`.max(..)`、`.week_start(..)`、`.calendar(..)` で制約と共通カレンダーを設定します。期間入力では `.start_label(..)` と `.end_label(..)` で各端点の名前を指定できます。

期間を選ぶカレンダーは、二つの端点を選んだときだけ呼び出し元へ確定します。それ以前にポップアップを閉じると元の期間を保ちます。期間の状態を持つ中核から「Pick a start date」と「Pick an end date」を通知します。`.presentation(DatePresentation::Desktop)` はデスクトップのポップアップ、`BottomSheet` はタッチ用のシート、`Adaptive` はプラットフォームに合わせた表示です。

## カレンダーを表示する

```rust,no_run
use kivact::i18n::{CalendarDate, DateRange, Weekday};
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn schedule() -> impl IntoView {
    let today = CalendarDate::today();
    let day = signal(Some(today));
    kivact::ui::calendar(day)
        .today(today)
        .week_start(Weekday::Monday)
        .year_navigation(true)
        .caption(kivact::ui::CalendarCaption::Dropdown)
        .month_title(|date| format!("{}-{:02}", date.year(), date.month()))
        .events(vec![kivact::ui::CalendarEvent {date: today, color: rgb(0x2b8bd8)}])
        .range(Some(DateRange::new(today.add_days(5), today.add_days(9))))
        .variant(pop::WeekendHue::new(|day| match day {
            Weekday::Saturday | Weekday::Sunday => Some(pop::Hue::Pink),
            _ => None,
        }))
        .on_activate(|date| println!("opened {date}"))
}
```

`.week_start(..)` は先頭の列を指定し、既定値はロケールに従います。`.caption(CalendarCaption::Dropdown)` は月の見出しを月と年のメニューに置き換えます。`.year_navigation(true)` は年を移動するボタンを追加し、Shift と PageUp または PageDown でも年を移動できます。`.month_title(..)` は文字列の見出しを変更します。

`.events(..)` は一日につき最大三つの点を描きます。`.range(Some(DateRange::new(first, last)))` は期間を強調します。`pop::WeekendHue` はすべての Pop の色相を保持し、曜日の callback から読むアプリのリアクティブ値も使えます。標準の `.weekend(..)` と `.marks(..)` は意味に応じた `Tone` を受け取ります。`.on_activate(..)` は、既に選択されている日でも呼ばれます。

## 言語

月と曜日の名前はロケールに従います。メッセージのキーは `date.open`、`date.range_open`、`date.start`、`date.end`、`calendar.previous`、`calendar.next`、`calendar.previous_year`、`calendar.next_year`、`calendar.range.pick_start`、`calendar.range.pick_end` などです。

```rust,no_run
use kivact::i18n::{CalendarDate, Locale, Messages};
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;
fn picker() -> impl IntoView { kivact::ui::date_input("Date", signal(Some(CalendarDate::today()))) }
fn main() -> kivact::Result<()> {
    App::new("Dates")
        .locale(Locale::new("ja-JP"))
        .messages(Messages::new().with("date.open", "日付を選択").with("date.range_open", "期間を選択"))
        .theme(pop::theme(ThemeMode::Light))
        .run(picker)
}
```

見た目は[テーマとトークン](theme-and-tokens.md)、アプリが持つ検証規則は[検証](../getting-started/03-validate-and-submit.md)を参照してください。
