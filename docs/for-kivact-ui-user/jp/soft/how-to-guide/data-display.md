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

## Chat と source block

Soft theme では `kivact::ui::{chat_thread, chat_message, chat_composer, typing_indicator, code_block}` を使います。message list と layout は `chat_thread(for_each(items, key, render))` で組みます。`empty`、`loading_top`、`streaming`、`gap`、`align`、`composer`、`viewport_ref`、`history_revision`、`scroll_button` は標準 builder の入力です。履歴を先頭へ足す更新では、実際の dataset revision を `history_revision` に渡します。

著者は `chat_message(author)`、参加者は `sender`、本文は `content` で指定します。従来の bubble、metadata、system wrapper の入力は `avatar`、`metadata`、`time`、`status`、`footer`、`icon`、`group`、`bubble_width` に対応します。system message には `ChatSender::System` を使います。composer の開閉できる drawer は `drawer` slot 内に標準 `disclosure` を合成します。独立した入力と送信操作には `text_area` と `button` を使います。

Soft の `ChatDensity`、`BubbleVariant`、`SystemMessageVariant`、`ComposerElevation` は標準の `.variant(...)` に渡します。`ChatAppearance` はそれらの reactive value を保持します。source の `CodeBlockSize`、`CodeBlockWidth`、`CodeBlockContainer` も `.variant(...)` に渡し、reactive value は `CodeAppearance` にまとめます。copy feedback、行番号、強調行、diff、折り返し、高さ、controlled expansion、開閉の閾値、末尾 content は `ui::code_block` の入力として残ります。

言語 tokenizer は Soft に残ります。組み込みの `CodeLanguageInput::Named` または独自の `CodeLanguage` を `code_highlight` に渡し、標準 `highlight` の行 callback から呼びます。callback の content source に現在の `Tokens.colors.syntax` または明示的な `SyntaxColors` を渡すと、palette の reactive な上書きを保てます。`syntax_theme` は引き続き子の palette を供給します。streaming text、tool call、token、アプリが所有する dictation は Soft の domain です。
