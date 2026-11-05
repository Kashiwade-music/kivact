# データ表示

[English](../../../en/soft/how-to-guide/data-display.md)

一つのレコードの属性名と値を並べる場合は、メタデータリストを使います。セクションには見出しを付け、各値にはデータに合うビューを指定します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn build_details() -> impl IntoView {
    column((
        kivact::ui::heading("Build details", 2),
        soft::metadata_list(vec![
            soft::metadata_item("Status", soft::text("Passed")),
            soft::metadata_item("Revision", soft::code("ae72f04")),
        ]),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
}
```

複数のレコードを列で比較する場合や並べ替えが必要な場合は、標準データテーブルを使います。数値順や日時順を表示文字列に左右されないよう、元の値をテーブルでも利用できる形にします。

キーボードショートカットには標準の `kbd("mod+shift+p")`、または型付きの `kbd_combo(keys).modifiers(modifiers)` を使います。Soft は各 cap を `kbd_key`、行を `kbd` の recipe で装飾します。`mod` はプラットフォームの command key に従います。キーの glyph、矢印の別名、Escape、Return、リテラルの `plus` cap は翻訳済みの正式な読み上げ名を保ちます。アプリケーションのメッセージカタログで `kbd.key.*` と `kbd.separator` を変更できます。

状態を示す色には標準の `badge(label)` と `.tone(Tone::Info/Success/Warning/Danger)` を使います。分類用の色には `.variant(soft::BadgeVariant::Blue)` など Neutral の色別 variant を使います。`.icon(view)` と `.trailing_icon(view)` には独自の視覚要素を渡せます。従来の寸法にはアイコンを 12 px にします。操作する badge には `.on_press(...)` を指定できます。

見出しには `kivact::ui::heading(value, level)` を使います。`HeadingDisplay::D1`、`D2`、`D3` を `.variant(..)` に渡すと、意味上のレベルを維持したまま表示用の文字組みを変えられます。色、太さ、行数制限、折り返し、整列、capsize、取り消し線はノードのスタイルで指定します。省略時のツールチップは `ui::tooltip(value).enabled(anchor.text_truncated()).attach(heading.node_ref(anchor))` で合成し、呼び出し側がアンカーを持ちます。進捗は従来の Bar の動きとトーンに加え、`ProgressVariant::Ring` と `Gauge` にも対応します。

## Chat と source block

Soft theme では `kivact::ui::{chat_thread, chat_message, chat_composer, typing_indicator, code_block}` を使います。message list と layout は `chat_thread(for_each(items, key, render))` で組みます。`empty`、`loading_top`、`streaming`、`gap`、`align`、`composer`、`viewport_ref`、`history_revision`、`scroll_button` は標準 builder の入力です。履歴を先頭へ足す更新では、実際の dataset revision を `history_revision` に渡します。

著者は `chat_message(author)`、参加者は `sender`、本文は `content` で指定します。従来の bubble、metadata、system wrapper の入力は `avatar`、`metadata`、`time`、`status`、`footer`、`icon`、`group`、`bubble_width` に対応します。system message には `ChatSender::System` を使います。composer の開閉できる drawer は `drawer` slot 内に標準 `disclosure` を合成します。独立した入力と送信操作には `text_area` と `button` を使います。

Soft の `ChatDensity`、`BubbleVariant`、`SystemMessageVariant`、`ComposerElevation` は標準の `.variant(...)` に渡します。`ChatAppearance` はそれらの reactive value を保持します。source の `CodeBlockSize`、`CodeBlockWidth`、`CodeBlockContainer` も `.variant(...)` に渡し、reactive value は `CodeAppearance` にまとめます。copy feedback、行番号、強調行、diff、折り返し、高さ、controlled expansion、開閉の閾値、末尾 content は `ui::code_block` の入力として残ります。

言語 tokenizer は Soft に残ります。組み込みの `CodeLanguageInput::Named` または独自の `CodeLanguage` を `code_highlight` に渡し、標準 `highlight` の行 callback から呼びます。callback の content source に現在の `Tokens.colors.syntax` または明示的な `SyntaxColors` を渡すと、palette の reactive な上書きを保てます。`syntax_theme` は引き続き子の palette を供給します。streaming text、tool call、token、アプリが所有する dictation は Soft の domain です。

## テーブル

静的なテーブルには `kivact::ui::{table, table_header, table_body, table_footer, table_row, table_head, table_cell}` を使います。Soft theme は実際の recipe を装飾します。`TableAlign` は `TableCell::align(TextAlign)` に対応し、`width` と reactive な `col_span` は標準 cell の入力です。`.variant(...)` に `TableRowAppearance::new(dividers, hover, striped)` または `TableCellAppearance::new(density, vertical_align, divider)` を渡します。各入力は reactive value を受け付けます。従来の `TableDensity` alias は `ItemDensity` に統一します。行の選択は `.with_node(|node| node.selected(selected))` で指定します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn summary() -> impl IntoView {
    kivact::ui::table((
        kivact::ui::table_header((kivact::ui::table_row((
            kivact::ui::table_head(text("Project")),
            kivact::ui::table_head(text("Status")),
        )),)),
        kivact::ui::table_body((kivact::ui::table_row((
            kivact::ui::table_cell(text("Design")).variant(
                soft::TableCellAppearance::new(soft::ItemDensity::Compact,
                    kivact::style::Align::Center, true)),
            kivact::ui::table_cell(soft::table_row_status("Passed", soft::RowStatusTone::Success)),
        )).variant(soft::TableRowAppearance::new(true, true, false)),)),
    ))
}
```

`table_selection_toolbar(selection)` は `clear_label`、`selection_label`、`actions` を維持し、表示と件数を呼び出し側の selection binding から導出します。clear button と toolbar は標準部品です。`table_row_status(label, RowStatusTone)` の四つの色は標準 badge の `RowStatusDot` appearance で描画します。

DataTable の 0 始まりの page binding には標準 `pagination(page_count, page)` を使い、`.summary(...)` でページ数を表示できます。`table_row_index(index)` は `text(move || index.get().to_string())` に `TextAlign::End` と theme の secondary text color を指定する形に対応します。操作できる表には引き続き `data_table` と `DataColumn` を使います。並べ替え、選択、filter、列幅と順序、sticky column、paging、row status、tree/group row、empty content、row action、toolbar/footer slot の持ち主と API は変わりません。
