# 選択肢とクイック操作を用意する

[English](../../../en/game/how-to-guide/choices-and-toolbars.md)

ゲーム風のメニューで、プレイヤーに 1 つを選ばせ（難易度、プラン、画面など）、ツールバーでクイック操作を提供するための手引きです。キットは選択のための新しいコントロールを足しません。標準の `radio_group` と `tabs` に、`.variant(..)` で選ぶ 4 つの見た目を足します。動作、キーボード操作、アクセシビリティは標準のままです。

## 斜めのセグメントで 1 つを選ぶ

セグメントコントロールは選択肢を必ず 1 つ選ぶので、ラジオグループです。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game as game;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

fn difficulty() -> impl IntoView {
    let level = signal(Some(Difficulty::Normal));
    column((
        radio_group(
            "Difficulty",
            vec![
                ListItem::new(Difficulty::Easy, "EASY"),
                ListItem::new(Difficulty::Normal, "NORMAL"),
                ListItem::new(Difficulty::Hard, "HARD"),
            ],
            level,
        )
        .orientation(Orientation::Horizontal)
        .variant(game::Segmented),
        game::mono(move || format!("{:?}", level.get())),
    ))
    .style(style().gap(12).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Difficulty")
        .theme(game::theme(ThemeMode::Dark))
        .run(difficulty)
}
```

値は選択肢のキーを包んだ `Option` で、`None` はまだ何も選ばれていない状態です。キーには、この例のように自分の `enum` を使えます。矢印キーで移動と選択ができ、端で折り返し、無効な選択肢（`ListItem::disabled(true)`）は飛ばされます。この用途に `tabs` を使わないでください。タブはビューを切り替えるもので、セグメントコントロールは値を設定するものです。

## カードでプランを選ぶ

`ChoiceCard` は選択肢を大きなカードとして描きます。`game::card_label` は項目のアイコン、タイトル、説明を描き、スロットには価格など、ほかに載せたいものを足します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn plans() -> impl IntoView {
    let plan = signal(Some("team"));
    radio_group(
        "Plan",
        vec![
            ListItem::new("solo", "SOLO").with_description("1 seat"),
            ListItem::new("team", "TEAM").with_description("50 seats"),
        ],
        plan,
    )
    .variant(game::ChoiceCard)
    .item(|item, state| {
        let price = match item.with(|item| *item.key()) {
            "solo" => "$9/month",
            _ => "$39/month",
        };
        (game::card_label(item, state), text(price))
    })
}
# fn main() {}
```

`.item(|item, state| view)` は、選択肢の表示内容を変える標準の方法です。カードの中に複数のビューを置くには、タプルを返します。

## 縦のタブでナビゲーションする

`Navigation` の見た目のタブは、サイドメニューになります。タブは開いているセクションのキーを書き込み、`.content(..)` がセクションを表示します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn menu() -> impl IntoView {
    let section = signal("inbox");
    tabs(
        "Sections",
        vec![
            ListItem::new("inbox", "INBOX").with_icon("bell"),
            ListItem::new("squad", "SQUAD").with_icon("users"),
        ],
        section,
    )
    .orientation(Orientation::Vertical)
    .variant(game::Navigation)
    .item(|item, state| {
        let count = match item.with(|item| *item.key()) {
            "inbox" => Some(game::badge("3", Tone::Info)),
            _ => None,
        };
        (game::navigation_label(item, state), count)
    })
    .content(|key| text(format!("Section: {key}")))
}
# fn main() {}
```

`game::navigation_label` はアイコン、ラベル、説明を描き、スロットはその後ろにバッジや件数を置きます。横のタブは、既定では下線の見た目で、`.variant(game::Boxed)` を付けるとタブが箱の中に収まります（[チュートリアル](../getting-started/03-tabs-and-items.md) と同じ）。タブを切り替えても、パネルに入力した内容を保ちたいときは `.keep_mounted()` を足します。すべてのパネルが一度だけ作られ、選択されていないタブのパネルは隠されます。

## ツールバーにクイック操作を置く

`game::toolbar(children)` は枠線付きの密な行、`game::button_group(children)` は関連するボタンを寄せ集め、`game::tool_button(label)` はアイコン用の正方形のゴーストボタンです。ツールボタンにはアイコンを付けてラベルを隠します。ラベルはアクセシブルな名前として残ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn tools(zoom: Signal<f64>) -> impl IntoView {
    game::toolbar((
        game::button_group((
            game::tool_button("Zoom out")
                .icon(icon("chevron-down"))
                .hide_label(true)
                .on_press(move || zoom.update(|z| *z = (*z - 0.25).max(0.25))),
            game::tool_button("Zoom in")
                .icon(icon("chevron-up"))
                .hide_label(true)
                .on_press(move || zoom.update(|z| *z += 0.25)),
        )),
        game::mono(move || format!("{:.0}%", zoom.get() * 100.0)),
        game::icon_menu(
            "More",
            vec![MenuEntry::item("reset", "Reset view"), MenuEntry::item("quit", "Quit")],
        )
        .on_select(move |key| {
            if key == "reset" {
                zoom.set(1.0);
            }
        }),
    ))
}
# fn main() {}
```

`game::chevron(label, caption)` は等幅のキャプションの付いた大きな呼びかけ用のボタンです。`game::stepper(labels, current)`、`game::breadcrumb(items)`、`game::pagination(page, count)` は、現在位置をシグナルから受け取るナビゲーションの補助です。

## 関連

- [一覧とツリーでアイテムを見せる](item-lists-and-selection.md)
- [メーターと状態を見せる](meters-and-status.md)
- すべての見た目とコンポーネント: `cargo doc -p kivact-ui-game --open`
