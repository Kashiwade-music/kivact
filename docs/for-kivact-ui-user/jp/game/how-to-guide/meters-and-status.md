# メーターと状態表示

[English](../../../en/game/how-to-guide/meters-and-status.md)

Game テーマで標準の `kivact::ui` 部品を使います。recipe が斜めのセグメント、区切り付きの円、KPI タイル、状態バッジ、フィードバックのパネルを描きます。

## 進捗を見せる

進捗の値は 0 から 1 の割合です。ゲージはアプリケーションの値を `value / total` に変換し、全体が 0 の場合を処理します。`game::GaugeSegments` に同じリアクティブな全体数を渡すとバーの本数が決まります。この variant は進捗の状態を所有しません。円には `ProgressVariant::Ring` を使い、小さなキャプションを添えられます。`game::Striped` は棒に動く斜線を加えます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact::ui::{self, ProgressVariant};
use kivact_ui_game as game;

fn meters() -> impl IntoView {
    let bars = signal(7usize);
    let total = signal(10usize);
    let percent = signal(76.0);
    row((
        ui::progress(move || Some(if total.get() == 0 { 0.0 } else {
            bars.get() as f64 / total.get() as f64
        })).name("Hull").variant(ProgressVariant::Gauge)
            .variant(game::GaugeSegments(total.into_value())).variant(Tone::Danger),
        ui::progress(move || Some(percent.get() / 100.0)).name("Shield")
            .variant(ProgressVariant::Ring).variant(Tone::Info)
            .with_node(|node| node.child(game::mono("SHIELD").style(style().font_size(9)))),
        ui::progress(move || Some(percent.get() / 100.0)).name("Upload")
            .moving(true).variant(game::Striped),
    )).style(style().gap(16))
}
# fn main() {}
```

## 数値を整形する

`stat(label, formatted_value)` に `game::StatVariant::Kpi` を指定します。数値の書式、小数桁、区切り、単位、アニメーションはアプリケーションが所有します。値にはリアクティブな文字列を渡します。`label_content` はアイコン付き見出し、`value_content` は別々に装飾する数値と単位、`chart` は `game::sparkline(samples)` などのチャートです。スパークラインは渡されたサンプルを描くだけです。`delta` は変化のキャプションで、`Tone` が意味に応じた色を選びます。`trend` は独立した変化の方向です。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact::motion::{Animated, Easing, Transition};
use kivact::style::Align;
use kivact_ui_game as game;

fn accuracy() -> impl IntoView {
    let target = signal(87.5);
    let shown = Animated::from(0.0, target, Transition::tween(900,
        Easing::CubicBezier(0.2, 0.7, 0.2, 1.0)));
    let formatted = memo(move || format!("{:.1}", shown.get()));
    kivact::ui::stat("ACCURACY", move || format!("{}%", formatted.get()))
        .variant(game::StatVariant::Kpi).variant(Tone::Success)
        .label_content(row((icon("target"), text("ACCURACY"))))
        .value_content(row((
            text(formatted),
            text("%").style(style().font_size(16).color(ColorToken::TextMuted)),
        )).style(style().gap(2).align_items(Align::Baseline)))
        .delta("+12.4%")
        .chart(game::sparkline(vec![3.0, 5.0, 4.0, 8.0, 9.0, 12.0]))
}
# fn main() {}
```

## バッジとアバターを見せる

バッジは `kivact::ui::badge(label).tone(tone)` です。`game::BadgeVariant` は `Dot`（既定の四角い点）、`Plain`、`Solid`、`Outline` を選びます。通常の修飾子も使えます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn statuses() -> impl IntoView {
    row((
        kivact::ui::badge("RUNNING").tone(Tone::Info),
        kivact::ui::badge("v2.4.1").tone(Tone::Neutral).variant(game::BadgeVariant::Outline),
        kivact::ui::badge("OFFLINE").tone(Tone::Danger).variant(game::BadgeVariant::Solid),
    ))
}
# fn main() {}
```

正確なイニシャルには `kivact::ui::avatar(name).fallback_content(..)` を使います。`.variant(game::AvatarVariant { side: 32., color })` は論理ピクセル単位の寸法と色、`.status(Some(Tone::Success))` は四角い在席マーカーです。`avatar_group(children).max(3usize)` でまとめます。一部だけ読み込んでいる場合、`.total(8usize)` は渡したアバターを含む参加者の総数です。3 人を表示するとカットコーナーの `+5` タイルが表示されます。テーマの重なりは 6 論理ピクセルです。

## フィードバックを見せる

`alert(title)` は説明、アイコン、操作、開くたびに作る詳細を持てます。`.visible(binding)` でアプリケーションを唯一の表示状態の所有者にします。setter は閉じる要求を拒否できます。`.on_dismiss(..)` は受け入れられた操作が完了してフォーカスが戻った後の通知です。`.dismiss_content(..)` は操作の見た目、`.dismiss_label(..)` はアクセシブルな名前です。既定は assertive ですが、情報やバナーには明示的に polite な状態通知を選べます。

バナーは `AlertVariant::Banner` で、縞の見出しと全幅の紺色の面を描きます。空状態は操作とイラストを持てます（`.icon(())` は Game recipe のイラストです）。`SkeletonVariant::Rows` はカットコーナーのアバターと 2 本のキャプションを表示します。標準スケルトンがアニメーションを所有し、動きを減らす設定で止まります。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn warning() -> impl IntoView {
    let visible = signal(true);
    column((
        kivact::ui::alert("Low fuel").tone(Tone::Warning)
            .icon(icon("alert-triangle"))
            .description("Return to base within 2 minutes.")
            .visible(visible).dismissible(true)
            .dismiss_label("Close alert").dismiss_content(text("×")),
        kivact::ui::alert("NOTICE").variant(kivact::ui::AlertVariant::Banner)
            .tone(Tone::Warning)
            .with_node(|node| node.role(Role::Status).live_region(kivact::view::LiveRegion::Polite))
            .content(|| text("Maintenance at 02:00 UTC"))
            .actions(button("Details")),
        kivact::ui::empty_state("NO RECORDS").icon(())
            .description("Try another search.").actions(button("Clear filters")),
        kivact::ui::skeleton().variant(game::SkeletonVariant::Rows).lines(2usize),
    ))
}
# fn main() {}
```

短時間のメッセージは `kivact::ui::toast_region(toaster())` と `Toast::new(label).tone(tone)` です。文言はアプリケーションが渡します。

## 用語と詳細を組み合わせる

`description_list` と `description` は、幅 120 ピクセルの陰影付きの用語列と任意の詳細ビューを組み合わせます。詳細ごとに型が違っていてもかまいません。更新したいシグナルは各ビューの中で読みます。`game::activity` は Game 固有の菱形付きタイムラインです。`ActivityEntry` は時刻、内容、意味を一度だけ消費します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn details() -> impl IntoView {
    column((
        kivact::ui::description_list((
            kivact::ui::description("PILOT", game::mono("Mika")),
            kivact::ui::description("STATUS", kivact::ui::badge("ACTIVE").tone(Tone::Success)),
        )),
        game::activity(vec![game::ActivityEntry::new(
            "09:40", text("Review passed"), Tone::Success,
        )]),
    ))
}
# fn main() {}
```
