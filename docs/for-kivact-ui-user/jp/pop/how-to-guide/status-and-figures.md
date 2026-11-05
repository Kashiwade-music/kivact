# 状態と数値を表示する

[English](../../../en/pop/how-to-guide/status-and-figures.md)

Pop テーマと標準の `kivact::ui` 部品を使います。アプリが整形済みの文字列と読み取り専用のグラフデータを渡し、キットが見た目を担当します。

## 統計、見出し、詳細

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn summary() -> impl IntoView {
    let count = signal(1284usize);
    column((
        row((kivact::ui::badge("01").variant(pop::SectionNumber),
             kivact::ui::heading("Customers", 2))).style(style().gap(16)),
        kivact::ui::stat("Active customers", move || count.get().to_string())
            .delta("↗ 8.2%")
            .trend(kivact::ui::Trend::Up)
            .variant(pop::StatHue::new(pop::Hue::Mint))
            .icon(icon("users"))
            .chart(pop::sparkline(vec![2.0, 4.0, 3.0, 7.0, 9.0])),
        kivact::ui::description_list((
            kivact::ui::description("Name", text("Haruka Sato")),
            kivact::ui::description("Status", kivact::ui::badge("Active").variant(pop::Hue::Mint)),
            kivact::ui::description("Used", kivact::ui::progress(Some(0.72))
                .name("Used capacity").variant(kivact::ui::ProgressVariant::Ring)
                .variant(pop::Hue::Blue)),
        )),
    )).style(style().gap(16).width(420))
}
```

`stat(label, value)` はリアクティブな文字列を受け取ります。`.delta(..)` は整形済みの変化量、`.trend(Trend::Up | Down | Neutral)` はその意味です。`pop::StatHue` で任意のリアクティブな Pop の色相を指定できます。`.icon(..)`、`.chart(..)`、`.label_content(..)`、`.value_content(..)` は一度だけ構築するビューのスロットです。`pop::sparkline(points)` は独立した描画部品として残り、任意の chart スロットに置けます。

`heading(text, level)` は見出しレベルを公開します。標準のバッジと `pop::SectionNumber` で節の番号を組み、説明文は見出しの横か下に置きます。`card(children)` は枠線付きの面です。統計の部品は数値の状態を別に持ちません。

`description_list(..)` は `description(term, detail)` の行をまとめます。detail にはバッジ、ボタン、進捗リングなど任意のビューを置けます。`.term_content(..)` はアクセシブルな名前を保って表示内容だけを変えます。

`progress(Some(fraction))` は 0 から 1 の値、`None` は不定を表します。`ProgressVariant::Ring` は円と百分率、`Bar` は通常のバー、`Gauge` は分割したバーです。`.name(..)` は支援技術向けの名前です。固定の `pop::Hue` またはリアクティブな `pop::ProgressHue` で色を指定し、`pop::Striped` と `pop::Segments(n)` でバーの見た目も選べます。必要なら可視ラベルを隣に置きます。

## レコードの行とアバター

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn customer() -> impl IntoView {
    let look=pop::AvatarLook::default().size(48).hue(330.0).halo(true).online(true);
    kivact::ui::item_list((
        kivact::ui::item_row("Haruka Sato").description("Team plan")
            .media(kivact::ui::avatar("HS").fallback_content(||text("HS"))
                .variant(look).status_content(look.decorations()))
            .actions(kivact::ui::button("Open")),
    ))
}
```

`item_row` はリスト項目を公開し、`.media(..)`、`.description(..)`、`.actions(..)` を持ちます。アイコンのタイルは `.media(icon("user")).variant(pop::ItemHue::new(pop::Hue::Blue))` で作れます。

`AvatarLook` はピクセル直径、連続した色相角、halo、online の点を保持し、リアクティブな値も受け取ります。イニシャルは `.fallback_content(|| text("HS"))` に渡し、`look.decorations()` は `.status_content(..)` に一度だけ構築します。標準の画像、形、サイズ、意味を持つ状態の指定も使えます。

## パンくずと工程

パンくずはキーを持つ標準の `kivact::ui::breadcrumb` と `.separator(|| icon("chevron-right"))` を使います。工程の注記は `StepperItem::description` に渡します。`pop::DiamondSteps` は菱形の見た目を選び、`.min_step_width(0)` はすべての工程を表示します。標準のしきい値を有効にするとレスポンシブな折り畳み表示を使えます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn progress() -> impl IntoView {
    let current=signal(1usize);
    let notes=signal(vec!["10/01".to_owned(),"In progress".to_owned()]);
    kivact::ui::stepper(move || ["Received","Working","Reported"].into_iter().enumerate()
        .map(|(index,label)|kivact::ui::StepperItem::new(label)
            .description(notes.with(|notes|notes.get(index).cloned().unwrap_or_default())))
        .collect::<Vec<_>>(),current)
        .variant(pop::DiamondSteps).min_step_width(0)
}
```
