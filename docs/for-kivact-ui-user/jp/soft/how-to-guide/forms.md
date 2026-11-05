# フォーム

[English](../../../en/soft/how-to-guide/forms.md)

文字列には Kivact 標準のテキスト入力を、日付には Soft の型付き日付入力を使います。各フィールドを個別のシグナルにバインドします。保存時は両方の値を読み取り、一つのレコードとして検証できます。

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn event_form(save: impl Fn(String, CalendarDate) + 'static) -> impl IntoView {
    let name = signal(String::new());
    let date = signal(None::<CalendarDate>);

    column((
        text_input("Event name", name),
        soft::date_input("Event date", date),
        button("Save event")
            .variant(kivact::ui::ButtonVariant::Primary)
            .disabled(move || name.get().trim().is_empty() || date.get().is_none())
            .on_press(move || {
                if let Some(date) = date.get() {
                    save(name.get().trim().to_owned(), date);
                }
            }),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
}
```

入力エラーは該当フィールドの近くに表示し、修正方法を伝えます。永続化の境界でも送信値を検証します。

`soft::input_group_with` で作るコントロールには、グループの既定サイズが適用されます。個々のコントロールの `.size(..)` が優先されます。`soft::date_range_input(..).presets(..)` に `soft::DateRangePreset` を渡すと、カレンダーからの選択に加えて、名前付きの日付範囲を選べます。

`soft::rich_text_editor` は Markdown 文字列にバインドします。編集モードでは、ツールバーで選択範囲に太字、斜体、取り消し線、インラインコードを適用でき、操作後はエディターにフォーカスが戻ります。プレビューに切り替えると、同じバインディングの内容を整形して表示します。

キー付きの選択には標準 API を使います。`select(label, options, selected).open_on_arrow(true)` は従来の矢印キーで開く動作を保ち、`combobox(label, options, selected)` は検索を加えます。複数選択には `Vec<K>` の binding と `.chips()` を使い、tokenizer には `.exclude_selected()` も加えます。非同期の `.search(..)`、`.debounce(..)`、`.filter(..)`、`.placeholder(..)`、`.empty_text(..)` は標準 builder のオプションです。タッチ時のドロワーには `.presentation(SelectPresentation::BottomSheet)`、ホストのタッチ対応に従うには `Adaptive` を使います。独自の選択内容には `selection_field(label, value_label).content(factory)` を使い、検索の binding は factory の外で作ります。Soft は select と combobox の recipe と token を設定します。
