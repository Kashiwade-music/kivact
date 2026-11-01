# 2. 部品とレイアウト

Kivact の部品は Rust の関数です。引数で内容を受け取り、`impl IntoView` を返します。

```rust,no_run
use kivact::prelude::*;

fn message(title: &str, body: &str) -> impl IntoView {
    column((
        text(title),
        text(body),
    ))
    .style(style().gap(4).padding(12))
}

fn app() -> impl IntoView {
    column((
        message("お知らせ", "Kivact を始めました"),
        row((text("左"), text("右"))).style(style().gap(16)),
    ))
    .style(style().gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("部品の例").run(app)
}
```

`message` は二つの文字列を借りて受け取る部品です。`text` は入口で文字列を所有するので、呼出し元の短い `&str` を渡せます。表示に保存されるクロージャは引き続き `'static` が必要です。画面の内容が変わる場合の渡し方は [状態と反応性](state.md) で扱います。

`column` は縦、`row` は横に子を並べます。複数の異なる種類の子を渡すときは tuple にします。どちらも一つの子や `Vec` を受け取れます。`.style(style().gap(12))` は子の間隔を指定します。

## どの部品を使うか

| 書き方 | 用途 |
| --- | --- |
| `text("こんにちは")` | 文字を表示する |
| `column((a, b))` / `row((a, b))` | 縦 / 横に並べる |
| `grid((a, b, c))` | 格子に並べる（列は `style().grid_template_columns(..)`） |
| `node().child(a)` | 形や属性を付ける土台にする |
| `scroll(a)` | 縦スクロールさせる（横は `.orientation(Orientation::Horizontal)`、縦横の両方は `.both_axes()`） |
| `spacer()` | 余った空間を伸びて埋める |
| `stack((a, b))` | 子を重ねる。先頭の子が大きさを決める |

`text`、レイアウト、`node` は `kivact::prelude::*` から使えます。標準部品の `button`・`checkbox`・`switch`・`text_input` も同じ prelude から使えます。自分で操作用の箱を作るときは、`node()` に `role`、`name`、`focusable` と `on_press` を付けます。操作を共有する独自部品には [interaction の Part](../../for-component-author/interaction.md)を付けます。

## 関数を呼ぶ時点

`message(...)` のような部品関数は、親の部品を組み立てるときに一度呼ばれます。状態が変わっても関数全体を毎回呼び直す仕組みではありません。変化させる文字や属性は、状態を読むクロージャとして渡します。次の [状態と反応性](state.md) で実際に作ります。


## 自分で動かして確認する

`kivact-example-counter` の `src/main.rs` を上のコードへ置き換えて起動します。「左」「右」の間隔を `16` から `32` に変えて再起動し、横の間隔だけが変わることを確認してください。
