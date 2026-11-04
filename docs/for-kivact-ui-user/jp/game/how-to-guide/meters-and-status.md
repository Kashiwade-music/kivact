# メーターと状態を見せる

[English](../../../en/game/how-to-guide/meters-and-status.md)

体力やエネルギーのバー、状態ラベル、主要な数値、プロパティ表、イベントログなど、物事の状況を見せるための手引きです。ここで扱うのは表示用のコンポーネントです。渡された値を描くだけで書き換えることはないので、変化するメーターとは、アプリが更新するシグナルのことです。

## 変化する値をメーターで見せる

3 つのメーターがシグナルを受け取り、値が変わると描き直されます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{Tone};
use kivact::ui::{progress};
use kivact_ui_game as game;

fn vitals() -> impl IntoView {
    let hp = signal(7_usize);
    let shield = signal(64.0_f64);
    let loading = signal(0.35_f64);
    column((
        game::mono("HULL"),
        game::gauge("Hull", hp).total(10).tone(Tone::Danger),
        game::ring("SHIELD", shield, Tone::Info),
        progress(move || Some(loading.get()))
            .name("Loading")
            .variant(Tone::Success),
        button("Take a hit").on_press(move || {
            hp.update(|hp| *hp = hp.saturating_sub(1));
            shield.update(|s| *s = (*s - 8.0).max(0.0));
        }),
    ))
    .style(style().gap(12).margin(24))
}
# fn main() {}
```

| メーター | 値 | 見た目 |
| --- | --- | --- |
| `game::gauge(label, value)` | バーの本数（全体は `.total(n)`、既定は 100） | 斜めのバーの列 |
| `game::ring(label, percent, tone)` | 0 から 100 | 百分率を表示する区切り付きの円 |
| `kivact::ui::progress(value)` | `Some(0.0..=1.0)`、不明なあいだは `None` | 平らなバー |

- `gauge` の第 1 引数はアクセシブルな名前（メーターは `Role::Meter` を持つ）で、画面には表示されません。キャプションは `game::mono(..)` で自分で描きます。
- `.tone(Tone::..)` はゲージの色を決め、`ring` は最後の引数が色調です。プログレスバーは色調を variant として受け取ります（`.variant(Tone::Success)`）。
- `.size(ControlSize::Small)` はプログレスバーを細くします。`.variant(game::Striped)` と `.moving(true)` を組み合わせると、処理中を示す動く縞模様になります。
- プログレスバーには自前のラベルがないので、`.name(..)` で名前を付けます。
- `Tone` は `Neutral`、`Info`、`Success`、`Warning`、`Danger`、`Accent` です。値が何を意味するかを伝えると、色はテーマが選びます。

## 主要な数値を見せる

`game::kpi(label, target)` は、最初に現れたときに 0 から数え上げる大きな数字のタイルです。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn scoreboard() -> impl IntoView {
    row((
        game::kpi("KILLS", 1284.0)
            .decimals(0)
            .change("+12.4%")
            .tone(Tone::Info)
            .trend(vec![3.0, 5.0, 4.0, 8.0, 9.0, 12.0])
            .style(style().grow(1)),
        game::kpi("ACCURACY", 87.5)
            .decimals(1)
            .unit("%")
            .tone(Tone::Success)
            .style(style().grow(1)),
    ))
    .style(style().gap(16).margin(24))
}
# fn main() {}
```

- `.decimals(n)` は小数点以下の桁数（既定は 0）、`.format(|value| String)` は数値の整形の置き換え、`.unit(..)` は単位を付けます。
- `.change(text)` は変化量を表す小さな 1 行のテキストです。文字と矢印はアプリが書きます。`.tone(..)` で色が付きます。
- `.trend(samples)` は `f64` の標本のスパークラインを描きます。指定しなければ線は描かれません。
- `.heading(|| view)` は、キャプションを自分のビュー（アイコンとテキストなど）に置き換えます。
- 目標値は定数です。カウントアップは一度だけ再生されます。画面を開いているあいだ変わり続ける数字には、シグナルを読む `text`、`game::mono`、ゲージを使います。

## 状態を見せる

`game::badge(label, tone)` は、四角い点の付いた短い状態ラベルです。`game::tag_badge(label, tone, BadgeVariant::..)` は見た目を選べます。`Dot`（既定）、`Plain`、`Solid`、`Outline` です。どちらも、通常の `.style(..)` や `.test_id(..)` などの修飾子を受け取るノードを返します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn statuses() -> impl IntoView {
    row((
        game::badge("RUNNING", Tone::Info),
        game::tag_badge("v2.4.1", Tone::Neutral, game::BadgeVariant::Outline),
        game::tag_badge("OFFLINE", Tone::Danger, game::BadgeVariant::Solid),
    ))
    .style(style().gap(8).margin(24))
}
# fn main() {}
```

## プレイヤーにメッセージを出す

- `game::alert(title)` は、`.description(..)`、`.tone(..)`、`.actions(|| view)` を持つインラインの枠です。閉じるボタンは `.on_dismiss(..)` を渡したときだけ出ます。ハンドラーは要求を伝えるだけなので、アラートを消すには自分のシグナルを空にします（下の例を参照）。
- `game::banner(title, tone, action)` は、システムメッセージ向けの全幅の帯です。`.content(|| view)` がメッセージで、`action` は末尾に置きたいものです。
- `game::empty_state(title)` は、`.description(..)` と、操作用の `.content(|| view)` を持つ破線のパネルです。`game::skeleton().rows(n)` は読み込み中のプレースホルダーです。
- 短時間だけ出すメッセージはトーストです。`kivact::ui::toast_region(toaster())` を一度だけ配置し、toaster に `Toast::new("Saved").tone(Tone::Success)` を push します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn warning() -> impl IntoView {
    let shown = signal(true);
    show(shown, move || {
        game::alert("Low fuel")
            .tone(Tone::Warning)
            .description("Return to base within 2 minutes.")
            .on_dismiss(move || shown.set(false))
    })
}
# fn main() {}
```

## プロパティと履歴を見せる

`game::property_list(entries)` は、キーと値の行の表です。キーは文字列で、値はテキストでも任意のビューでもかまいませんが、すべて同じ型である必要があります。異なるビューを混ぜるときは `.into_view()` で共通の型にします。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn sheet() -> impl IntoView {
    game::property_list(vec![
        ("CALLSIGN".to_owned(), game::mono("VIPER-02").into_view()),
        ("STATUS".to_owned(), game::badge("ACTIVE", Tone::Success).into_view()),
        ("SQUAD".to_owned(), text("Alpha").into_view()),
    ])
}
# fn main() {}
```

`game::activity(entries)` は、`ActivityEntry::new(time, message, tone)` の時系列です。メッセージは文字列かビューです。エントリは自分のビューを持ち、タイムラインが作られるときに一度だけ使われます。そのため、増えていくログは `dynamic` でシグナルから作り直します。プロパティ表も同じく一度だけ作られるので、変化する値はそのビューの中（`game::mono(move || ..)`）に置きます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn log(events: Signal<Vec<(String, String)>>) -> impl IntoView {
    dynamic(events, |events| {
        game::activity(
            events
                .into_iter()
                .map(|(time, message)| game::ActivityEntry::new(time, message, Tone::Info))
                .collect(),
        )
    })
}
# fn main() {}
```

## 関連

- [テーマと密度を設定する](theme-and-density.md): 色とフォントについて。
- [一覧とツリーでアイテムを見せる](item-lists-and-selection.md)
- 各コンポーネントとそのオプション: `cargo doc -p kivact-ui-game --open`
