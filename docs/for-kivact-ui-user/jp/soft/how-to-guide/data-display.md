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
