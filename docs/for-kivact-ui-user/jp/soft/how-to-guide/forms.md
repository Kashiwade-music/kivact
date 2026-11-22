# フォーム

[English](../../../en/soft/how-to-guide/forms.md)

文字列には Kivact 標準のテキスト入力を、日付には Soft の型付き日付入力を使います。各フィールドを個別のシグナルにバインドします。保存時は両方の値を読み取り、一つのレコードとして検証できます。

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_soft as soft;

#[component]
fn event_form(save: Callback<(String, CalendarDate)>) -> impl Element {
    let name = signal(String::new());
    let date = signal(None::<CalendarDate>);

    column((
        text_input(name).label("Event name"),
        kivact::ui::date_input(date).label("Event date"),
        button("Save event")
            .variant(kivact::ui::ButtonVariant::Primary)
            .disabled(move || name.get().trim().is_empty() || date.get().is_none())
            .on_press(move || {
                if let Some(date) = date.get() {
                    save.call((name.get().trim().to_owned(), date));
                }
            }),
    ))
    .style(style().gap(soft::SPACE_S3))
}
```

入力エラーは該当フィールドの近くに表示し、修正方法を伝えます。永続化の境界でも送信値を検証します。

`kivact::ui::input_group(label, children)` で作るコントロールには、グループの既定サイズが適用されます。個々のコントロールの `.size(..)` が優先されます。`kivact::ui::date_range_input(..).presets(..)` に `kivact::ui::DateRangePreset` を渡すと、カレンダーからの選択に加えて、名前付きの日付範囲を選べます。

`soft::rich_text_editor` は Markdown 文字列にシグナルを結び付けます。編集モードでは、ツールバーで選択範囲に太字、斜体、取り消し線、インラインコードを適用でき、操作後はエディターにフォーカスが戻ります。プレビューに切り替えると、同じシグナルの内容を整形して表示します。

キー付きの選択には標準 API を使います。`select(label, items, selected).open_on_arrow(true)` は従来の矢印キーで開く動作を保ち、`combobox(label, items, selected)` は検索を加えます。複数選択には `Vec<K>` のシグナルと `.chips()` を使い、tokenizer には `.exclude_selected()` も加えます。非同期の `.search(..)`、`.debounce(..)`、`.filter(..)`、`.placeholder(..)`、`.empty_text(..)` は標準 builder のオプションです。タッチ時のドロワーには `.presentation(SelectPresentation::BottomSheet)`、ホストのタッチ対応に従うには `Adaptive` を使います。独自の選択内容には `select(items, value).label(label).trigger(view)` を使い、検索のシグナルは trigger の view の外で作ります。Soft は select と combobox の recipe とテーマの変数を設定します。

連結した操作には `kivact::ui::button_group(label, children)`、キー付きのトグルには `toggle_group(label, items, selected)` を使います。どちらも方向、継承するコントロールのサイズ、標準の `.disabled(..)` に対応します。メンバーの区切り線とサイズは `ButtonGroupMember::from(button).solid(true).keep_size(true)` で指定します。`.variant(soft::ButtonGroupVariant(soft::Elevation::Low))` はグループに影を加えます。押したときのアイコンは各選択肢の `ListItem::with_pressed_icon(..)`、ラベルを隠す場合は `.icon_only(..)` を使います。複数選択には `Vec<K>`、単一選択には `Option<K>` のシグナルを使います。

標準の `calendar(selected)` は `Option<CalendarDate>` と `Option<DateRange>` のどちらも受け取ります。初期月、無効な日、上下限、両端を含む日数の制約、可変の行数、月数、月外の日、週の始まり、自動フォーカス、確定コールバックを同じカレンダーで指定できます。`date_input`、`date_range_input`、`date_time_input`、`time_input` は `kivact::ui`、日付と時刻の値は `kivact::i18n` の型を使います。`.calendar(|calendar| ...)` の設定と範囲の preset は引き続き使えます。標準の field の上下限には `Some(date)`、秒の精度には `.format(...)` または `.time_format(...)` と `TimeStyle { seconds: true }` を指定します。Soft はカレンダーと日付・時刻の recipe、wheel の寸法を設定します。日付の解析とカレンダー名は Locale、コンポーネントのメッセージは標準の calendar/date/time のカタログキーが管理します。

ネイティブのファイル選択とドロップには標準の `dropzone(label).icon(icon("upload")).pick_on_press()` を使います。`DropzoneVariant::Field` は小さい field の装飾、既定は大きいターゲットです。選ばれたファイルは `.on_files(...)` で呼び出し側の状態に保存し、名前を `.hint(...)` に表示します。通常のボタンでその状態を消し、フォーカスをターゲットへ戻します。複数ファイルの選択には `.multiple()`、picker のフィルターには `.filters(...)` を使います。ドロップも検証するには `.validate(...)` を指定し、拡張子やメタデータ・サイズの検証はアプリケーションのコールバックに書きます。picker のみの小さい field にする場合は `.accept_drop(false)` を指定します。ネイティブ picker は引き続き使えます。

隣接する操作には標準 `input_group(label, children)` を使用します。既定サイズを継承し、操作に明示したサイズを保持します。
