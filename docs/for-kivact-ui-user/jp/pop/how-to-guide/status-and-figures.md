# 業務画面で数値・状態・レコードを見せる

[English](../../../en/pop/how-to-guide/status-and-figures.md)

業務アプリで定番の概要画面を作るための手引きです。傾向付きの主要な数値、状態バッジ付きのレコード一覧、詳細カードを作ります。ここで扱うのは表示用のコンポーネントです。渡された値を表示するだけで、取得や集計や保存は行いません。

## 数値をカードにする

`pop::stat_card(title, value)` は数値を載せたカードです。ほかはすべて任意のビルダーメソッドです。

```rust,no_run
use kivact::prelude::*;
use kivact::style::Icon;
use kivact_ui_pop as pop;

fn figures() -> impl IntoView {
    let open = signal(128_u32);
    row((
        pop::stat_card("Open orders", move || open.get().to_string())
            .icon(Icon::named("clipboard-list"))
            .trend("+12% this week")
            .tone(pop::Hue::Mint)
            .points(vec![3.0, 5.0, 4.0, 8.0, 7.0, 10.0])
            .style(style().grow(1)),
        pop::stat_card("Refunds", "7")
            .icon(Icon::named("coin"))
            .trend("+2 this week")
            .tone(pop::Hue::Red)
            .style(style().grow(1)),
    ))
    .style(style().gap(16).padding(24))
}
# fn main() {}
```

- 値と傾向はテキストです。ロケールや単位を知っているのはアプリなので、数値の整形はアプリが行います。シグナルに追従させたいときは、上のようにクロージャを渡します。
- `.tone(hue)` は傾向のテキストの色を決めます。良い傾向は `Mint`、悪い傾向は `Red`、中立は `Gray`（既定）です。何が良い傾向かはキットは判断しません。
- `.points(..)` は、数字の下に標本のスパークラインを描きます。標本は `f64` で、カードが書き換えることはありません。
- `.icon(..)` にはキットのアイコンセットの名前を渡します。`home`、`users`、`calendar`、`chart-bar`、`coin`、`clipboard-list` などが含まれています。
- カードは、ほかの Pop のビルダーと同様に、`.style(..)`、`.test_id(..)` など、すべての Kivact ノードが持つ修飾子を受け取ります。

## パネルと見出しでまとめる

`pop::panel(children)` は、枠線付きで子要素を縦に並べるカードです。`pop::section_heading(number, title, subtitle)` は、複数のセクションを持つ画面向けの番号付き見出しです。セクションごとに `panel` で囲みます。

## 状態付きのレコードを並べる

`pop::list_item(title)` は、レコード 1 件の行です。丸いアイコン（任意）、タイトルとサブタイトル、末尾に置くビューを持ちます。`kivact::ui::badge(label)` は状態のチップです。

```rust,no_run
use kivact::prelude::*;
use kivact::style::Icon;
use kivact_ui_pop as pop;

#[derive(Clone, PartialEq)]
struct Customer {
    name: &'static str,
    plan: &'static str,
    active: bool,
}

fn customers(rows: Signal<Vec<Customer>>) -> impl IntoView {
    pop::panel((
        pop::section_heading("01", "Customers", "Last 7 days"),
        for_each(
            rows,
            |customer: &Customer| customer.name,
            |customer| {
                let customer = customer.get_untracked();
                pop::list_item(customer.name)
                    .subtitle(customer.plan)
                    .icon(Icon::named("user"))
                    .tone(pop::Hue::Blue)
                    .trailing({
                        if customer.active {
                            kivact::ui::badge("Active").variant(pop::BadgeHue::new(pop::Hue::Mint))
                        } else {
                            kivact::ui::badge("Paused").variant(pop::BadgeHue::new(pop::Hue::Gray))
                        }
                    })
            },
        ),
    ))
    .style(style().width(420).margin(24).gap(8))
}
# fn main() {}
```

`for_each(items, key, row)` は顧客ごとに `list_item` を 1 つ作り、一覧が変わっても各行（とその状態）を保ちます。source がシグナルなので、行のクロージャーは `Binding<Customer>` を 1 つ受け取ります。顧客のデータが変わる場合は、reactive な文字やスタイルのクロージャーで読みます。`.trailing(..)` は一度だけ mount するビューを受け取ります。

`badge` は `.variant(pop::BadgeVariant::..)` で 5 つの見た目を持ちます。`Soft`（既定）、`Dot`（ラベルの前に色付きの点）、`Solid`、`Slant`、`Count`（小さな数字）です。色は `.hue(pop::Hue::..)` で、`Blue`、`Mint`、`Yellow`、`Pink`、`Red`、`Gray` のいずれかです。

## ラベルと値の行で詳細を見せる

`pop::key_value(key, value)` は、詳細カードの 1 行です。値は任意のビューなので、テキストでもバッジでもボタンでもかまいません。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn details() -> impl IntoView {
    pop::panel((
        pop::key_value("Name", text("Haruka Sato")),
        pop::key_value("Plan", text("Team")),
        pop::key_value("Status", kivact::ui::badge("Active").variant(pop::BadgeHue::new(pop::Hue::Mint))),
        pop::key_value(
            "Storage",
            pop::progress_ring(72.0, "Used", pop::Hue::Blue.into_value()),
        ),
    ))
    .style(style().width(360).margin(24))
}
# fn main() {}
```

`pop::progress_ring(percent, label, hue)` は 0 から 100 の値を円で描き、丸めた百分率を内側に表示します。値にはシグナルを渡せて、変わるとリングがアニメーションします。単純なバーが欲しいときは、標準の `kivact::ui::progress(Some(0.4))` を使います。値は 0 から 1 で、`None` なら不定、Pop テーマが見た目を描きます。支援技術向けの名前は `.name("Upload progress")` で付けます。

## 誰かを見せる

`kivact::ui::avatar(name)` に `pop::AvatarLook` を渡すと、Pop のピクセル直径、連続色相、halo と online の装飾を使えます。イニシャルの文字列は `.fallback_content(|| text("AS"))` に渡し、`look.decorations()` は `.status_content(..)` に一度だけ構築します。同じ look を `.variant(look)` に渡します。

## Hue と Tone: どちらを使うか

似た 2 つの enum があります。`pop::Hue` は Pop のパレットの色で、バッジ、統計カード、リスト項目、カレンダーが使います。`kivact::interaction::Tone`（`Neutral`、`Info`、`Success`、`Warning`、`Danger`、`Accent`）は意味で、アラート、トースト、コア由来のものが使います。Pop のコンポーネントが `Hue` を求めるときは色を自分で決め、標準やメッセージ系のコンポーネントが `Tone` を求めるときは意味を伝えて、色はテーマに任せます。

## 関連

- [テーブルで行を表示する](tables.md): 並べ替えとページ送りのある多数のレコード。
- [メッセージと通知を出す](messages-and-notifications.md): アラート、空の状態、トースト。
- [Pop のテーマとトークンを設定する](theme-and-tokens.md): 色相の変更。
- すべての表示コンポーネント: `cargo doc -p kivact-ui-pop --open`

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn person() -> impl IntoView {
    let look = pop::AvatarLook::default().size(48).hue(330.0).halo(true).online(true);
    kivact::ui::avatar("AS")
        .fallback_content(|| text("AS"))
        .variant(look)
        .status_content(look.decorations())
}
```
