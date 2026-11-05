# データ表示

[English](../../../en/soft/how-to-guide/data-display.md)

一つのレコードの属性名と値を並べる場合は、メタデータリストを使います。セクションには見出しを付け、各値にはデータに合うビューを指定します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn build_details() -> impl IntoView {
    column((
        soft::heading(2, "Build details"),
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

## Chat と source block

Soft theme では `kivact::ui::{chat_thread, chat_message, chat_composer, typing_indicator, code_block}` を使います。message list と layout は `chat_thread(for_each(items, key, render))` で組みます。`empty`、`loading_top`、`streaming`、`gap`、`align`、`composer`、`viewport_ref`、`history_revision`、`scroll_button` は標準 builder の入力です。履歴を先頭へ足す更新では、実際の dataset revision を `history_revision` に渡します。

著者は `chat_message(author)`、参加者は `sender`、本文は `content` で指定します。従来の bubble、metadata、system wrapper の入力は `avatar`、`metadata`、`time`、`status`、`footer`、`icon`、`group`、`bubble_width` に対応します。system message には `ChatSender::System` を使います。composer の開閉できる drawer は `drawer` slot 内に標準 `disclosure` を合成します。独立した入力と送信操作には `text_area` と `button` を使います。

Soft の `ChatDensity`、`BubbleVariant`、`SystemMessageVariant`、`ComposerElevation` は標準の `.variant(...)` に渡します。`ChatAppearance` はそれらの reactive value を保持します。source の `CodeBlockSize`、`CodeBlockWidth`、`CodeBlockContainer` も `.variant(...)` に渡し、reactive value は `CodeAppearance` にまとめます。copy feedback、行番号、強調行、diff、折り返し、高さ、controlled expansion、開閉の閾値、末尾 content は `ui::code_block` の入力として残ります。

言語 tokenizer は Soft に残ります。組み込みの `CodeLanguageInput::Named` または独自の `CodeLanguage` を `code_highlight` に渡し、標準 `highlight` の行 callback から呼びます。callback の content source に現在の `Tokens.colors.syntax` または明示的な `SyntaxColors` を渡すと、palette の reactive な上書きを保てます。`syntax_theme` は引き続き子の palette を供給します。streaming text、tool call、token、アプリが所有する dictation は Soft の domain です。
