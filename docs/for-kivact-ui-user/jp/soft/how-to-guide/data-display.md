# データ表示

[English](../../../en/soft/how-to-guide/data-display.md)

一つのレコードの属性名と値を並べる場合は、メタデータリストを使います。セクションには見出しを付け、各値にはデータに合うビューを指定します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn build_details() -> impl IntoView {
    column((
        kivact::ui::heading("Build details", 2),
        kivact::ui::description_list(vec![
            kivact::ui::description("Status", soft::text("Passed")),
            kivact::ui::description("Revision", soft::code("ae72f04")),
        ]),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
}
```

## 行とリスト

標準の `item_row(title)` と `item_list(children)` を使います。行には `.title_content`、`.description_content`、`.media`、`.actions`、`.marker` の slot があります。`.title_lines(Some(n))` と `.description_lines(Some(n))` は実際のプレーンテキストを制限し、明示的な None は無制限です。Soft の既定値は1行です。`.disabled` と `.selected` は標準の node modifier を使います。`.on_press` は行を操作し、`.delegate_to(target)` は追加のタブ停止点を作らず、実際の子コントロールにフォーカスと操作を渡します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn workspace() -> impl IntoView {
    let notifications=signal(false);
    let target=node_ref();
    kivact::ui::item_list((
        kivact::ui::item_row("Design").description("Active project"),
        kivact::ui::item_row("Notifications")
            .actions(kivact::ui::checkbox("Enable notifications",notifications).node_ref(target))
            .delegate_to(target),
    ))
    .header(text("Workspace"))
    .marker(kivact::ui::ItemListMarker::Decimal)
    .start(3u32)
    .variant(soft::ListLook::default().density(soft::ItemDensity::Compact)
        .dividers(true).edge(soft::ListEdge::Flush))
}
```

固定値には ItemDensity、ItemAlign、ItemLayout、ListEdge を `.variant(..)` で渡します。ItemLook は density、align、layout、highlighted の reactive な値を保持し、ListLook はリストの density、dividers、edge を保持します。4種類の edge はすべて選べます。マーカーは標準の `ItemListMarker::{None,Disc,Circle,Decimal}` で、`.start(..)` が連番の開始値です。単独の行をボタンや説明のグループにする場合は `.role(..)` を指定できます。

キー付きの reactive な内容はデータを一度だけ列挙し、項目をキーにして、同じ value から行の index を読みます。固定の直接の子には自動で位置が渡り、reactive な行には `.position(index, count)` を指定します。既存のキー付き行の identity と標準の pos_in_set / set_size を維持できます。

```rust,no_run
use kivact::prelude::*;

fn projects() -> impl IntoView {
    let items=signal(vec!["Design".to_string(),"Documentation".to_string()]);
    let rows=memo(move || items.get().into_iter().enumerate().collect::<Vec<_>>());
    let count=memo(move || rows.with(Vec::len));
    kivact::ui::item_list((for_each(rows,|(_,item)|item.clone(),move |row| {
        kivact::ui::item_row(move || row.get().1)
            .position(move || row.get().0,count)
    }),))
    .marker(kivact::ui::ItemListMarker::Decimal)
}
```


複数のレコードを列で比較する場合や並べ替えが必要な場合は、標準データテーブルを使います。数値順や日時順を表示文字列に左右されないよう、元の値をテーブルでも利用できる形にします。

キーボードショートカットには標準の `kbd("mod+shift+p")`、または型付きの `kbd_combo(keys).modifiers(modifiers)` を使います。Soft は各 cap を `kbd_key`、行を `kbd` の recipe で装飾します。`mod` はプラットフォームの command key に従います。キーの glyph、矢印の別名、Escape、Return、リテラルの `plus` cap は翻訳済みの正式な読み上げ名を保ちます。アプリケーションのメッセージカタログで `kbd.key.*` と `kbd.separator` を変更できます。

状態を示す色には標準の `badge(label)` と `.tone(Tone::Info/Success/Warning/Danger)` を使います。分類用の色には `.variant(soft::BadgeVariant::Blue)` など Neutral の色別 variant を使います。`.icon(view)` と `.trailing_icon(view)` には独自の視覚要素を渡せます。従来の寸法にはアイコンを 12 px にします。操作する badge には `.on_press(...)` を指定できます。

見出しには `kivact::ui::heading(value, level)` を使います。`HeadingDisplay::D1`、`D2`、`D3` を `.variant(..)` に渡すと、意味上のレベルを維持したまま表示用の文字組みを変えられます。色、太さ、行数制限、折り返し、整列、capsize、取り消し線はノードのスタイルで指定します。省略時のツールチップは `ui::tooltip(value).enabled(anchor.text_truncated()).attach(heading.node_ref(anchor))` で合成し、呼び出し側がアンカーを持ちます。進捗は従来の Bar の動きとトーンに加え、`ProgressVariant::Ring` と `Gauge` にも対応します。

用語と値の一覧には `ui::description_list` と `ui::description` を使い、一覧と各行に同じ `MetadataLook` を指定します。`columns(Single/Multi/Count(n))`、`label_position(Start/Top)`、`label_width(px)`、`orientation(Vertical/Horizontal)` で元の見た目の選択肢を維持し、値はリアクティブにもできます。単一列で横並びのラベル幅はグリッドの最も広い用語に揃い、計測値のコピーは不要です。アイコンや独自の用語表示は `term_content` に合成し、アクセシブルな用語は維持します。タイトルは一覧の外に置きます。表示件数の上限と開閉を制御する場合は、次の例のように呼び出し側で上限と Core disclosure の binding を持ち、トリガーの文字列をアプリ側で翻訳します。

```rust,no_run
use kivact::prelude::*;
use kivact::ui;
use kivact_ui_soft::{MetadataColumns, MetadataLook};

fn expandable_details() -> impl IntoView {
    let expanded = signal(false);
    let limit = signal(2usize);
    let layout = MetadataLook::default().columns(MetadataColumns::Multi);
    let attributes = [("Owner", "Ada"), ("Status", "Ready"), ("Revision", "ae72f04")];
    let pairs = attributes.into_iter().enumerate().map(move |(index, (term, value))| {
        node().hidden(move || index >= limit.get() && !expanded.get())
            .child(ui::description(term, text(value)).variant(layout))
    }).collect::<Vec<_>>();
    column((
        ui::heading("Project details", 3),
        ui::description_list(pairs).variant(layout),
        ui::disclosure(move || if expanded.get() { "Show less" } else { "Show more" }, expanded),
    ))
}
```

## Chat と source block

Soft theme では `kivact::ui::{chat_thread, chat_message, chat_composer, typing_indicator, code_block}` を使います。message list と layout は `chat_thread(for_each(items, key, render))` で組みます。`empty`、`loading_top`、`streaming`、`gap`、`align`、`composer`、`viewport_ref`、`history_revision`、`scroll_button` は標準 builder の入力です。履歴を先頭へ足す更新では、実際の dataset revision を `history_revision` に渡します。

著者は `chat_message(author)`、参加者は `sender`、本文は `content` で指定します。従来の bubble、metadata、system wrapper の入力は `avatar`、`metadata`、`time`、`status`、`footer`、`icon`、`group`、`bubble_width` に対応します。system message には `ChatSender::System` を使います。composer の開閉できる drawer は `drawer` slot 内に標準 `disclosure` を合成します。独立した入力と送信操作には `text_area` と `button` を使います。

Soft の `ChatDensity`、`BubbleVariant`、`SystemMessageVariant`、`ComposerElevation` は標準の `.variant(...)` に渡します。`ChatAppearance` はそれらの reactive value を保持します。source の `CodeBlockSize`、`CodeBlockWidth`、`CodeBlockContainer` も `.variant(...)` に渡し、reactive value は `CodeAppearance` にまとめます。copy feedback、行番号、強調行、diff、折り返し、高さ、controlled expansion、開閉の閾値、末尾 content は `ui::code_block` の入力として残ります。

言語 tokenizer は Soft に残ります。組み込みの `CodeLanguageInput::Named` または独自の `CodeLanguage` を `code_highlight` に渡し、標準 `highlight` の行 callback から呼びます。callback の content source に現在の `Tokens.colors.syntax` または明示的な `SyntaxColors` を渡すと、palette の reactive な上書きを保てます。`syntax_theme` は引き続き子の palette を供給します。streaming text、tool call、token、アプリが所有する dictation は Soft の domain です。

階層の collection は `kivact::ui::tree(label, nodes).expanded(expanded)` で作ります。Soft の tree recipe が行の密度、祖先のガイド線、開閉マーカーのアニメーションを描きます。`.variant(soft::TreeLook::default().density(soft::ItemDensity::Compact).guides(false))` で見た目を変更でき、密度とガイド表示には reactive な値も渡せます。選択、activation、header、行の独自の内容には `.selection(..)`、`.on_activate(..)`、`.header(..)`、`.item(..)` を使います。`.marker_content(..)` には実際の行と祖先の projection が渡されます。

枝の表示には `.reveal(2.0, kivact::motion::Transition::tween(734.77, kivact::motion::Easing::CubicBezier(0.327, 0.694, 0.116, 1.0)))` を使います。閉じた行は直ちにキーボード操作の対象から外れ、退出する絵はアニメーションの間保持されます。共通の開閉ラベルは `tree.toggle_children` です。

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
