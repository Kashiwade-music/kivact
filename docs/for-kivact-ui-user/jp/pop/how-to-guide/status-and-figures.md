# 状態と数値を表示する

[English](../../../en/pop/how-to-guide/status-and-figures.md)

Pop テーマと標準の `kivact::ui` 部品を使います。アプリは整形済みの文字列と読み取り専用のグラフのデータを渡し、見た目はキットが担当します。

## 統計、見出し、詳細

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
#[component]
fn summary() -> impl Element {
    let count = signal(1284usize);
    column((
        row((kivact::ui::badge("01").variant(pop::SectionNumber),
             kivact::ui::heading("Customers", 2))).style(style().gap(16)),
        kivact::ui::stat("Active customers", move || count.get().to_string())
            .delta("↗ 8.2%")
            .trend(kivact::ui::Trend::Up)
            .variant(pop::Hue::Mint)
            .icon(icon("users"))
            .chart(kivact::ui::sparkline(vec![2.0, 4.0, 3.0, 7.0, 9.0])),
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

`stat(label, value)` はリアクティブな文字列を受け取ります。`.delta(..)` は整形済みの変化量を、`.trend(Trend::Up | Down | Neutral)` はその意味を渡します。`pop::Hue` は統計の色相を選びます。`.icon(..)`、`.chart(..)`、`.label_content(..)`、`.value_content(..)` は、取り付けられるビューのスロットです。`kivact::ui::sparkline(points)` は標準のグラフを Pop の見た目で描き、どの chart のスロットにも置けます。

`heading(text, level)` は意味上の見出しレベルを公開します。節の番号は、標準のバッジに `pop::SectionNumber` を使って組み立て、任意の補足の文は見出しの横か下に置きます。`card(children)` は枠線付きの面を作ります。統計に、数値の状態を別に持たせる必要はありません。

`description_list(..)` は `description(term, detail)` の行をまとめます。detail には、バッジ、ボタン、進捗リングを含む任意のビューを置けます。`.term_content(..)` は、支援技術の名前を保ったまま、見える項目名だけを変えます。

`progress(Some(fraction))` は 0 から 1 の値を受け取り、`None` は不定を表します。`ProgressVariant::Ring` は円と見える百分率を、`Bar` は通常のバーを、`Gauge` は区切られたバーを表示します。`.name(..)` は支援技術向けの名前を与えます。`pop::Hue` は、固定の値でもシグナルから読む値でも、色を選びます。`kivact::ui::Striped` と `pop::Segments(n)` は、バーの見た目の指定を保ちます。必要なら、部品の隣に見えるラベルを置いてください。

## レコードの行とアバター

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
#[component]
fn customer() -> impl Element {
    kivact::ui::item_list((
        kivact::ui::item_row("Haruka Sato").description("Team plan")
            .media(kivact::ui::avatar("HS").fallback_content(||text("HS"))
                .variant(pop::AvatarSize(48.0))
                .variant(pop::AvatarHue(330.0))
                .status_content(pop::avatar_decorations().size(48.0).hue(330.0).halo(true).online(true)))
            .actions(kivact::ui::button("Open")),
    ))
}
```

`item_row` はリストの項目を公開し、`.media(..)`、`.description(..)`、`.actions(..)` のスロットを持ちます。アイコンのタイルは `.media(icon("user")).variant(pop::Hue::Blue)` で作れます。

`AvatarSize` と `AvatarHue` は、アバターのピクセル直径と連続した色相角を設定します。`pop::avatar_decorations()` は halo の環と online の点を描きます。`.status_content(..)` に取り付けてください。イニシャルは `.fallback_content(|| text("HS"))` で渡します。標準のアバターの画像、形、意味を持つ状態の指定も使えます。

## パンくずと工程

Pop の経路表示には、キーを持つ `kivact::ui::breadcrumb` の項目と `.separator(|| icon("chevron-right"))` を使います。工程の注記は `StepperItem::description` に属します。`pop::DiamondSteps` は菱形の見た目を選び、`.min_step_width(0)` はすべての工程を表示させます。標準のしきい値は、画面幅に応じて折り畳む表示を有効にします。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
#[component]
fn progress() -> impl Element {
    let current=signal(1usize);
    let notes=signal(vec!["10/01".to_owned(),"In progress".to_owned()]);
    kivact::ui::stepper(move || ["Received","Working","Reported"].into_iter().enumerate()
        .map(|(index,label)|kivact::ui::StepperItem::new(label)
            .description(notes.with(|notes|notes.get(index).cloned().unwrap_or_default())))
        .collect::<Vec<_>>(),current)
        .variant(pop::DiamondSteps).min_step_width(0)
}
```
