# ステップ 2: コンポーネントとレイアウト

[English](../../en/getting-started/02-components-and-layout.md)

このステップでは、`column` と `row` で画面を組み立て、ヘッダーと仮のタスク一覧を小さなコンポーネントに分けます。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

fn header() -> impl IntoView {
    column((
        text("タスク"),
        text("今日やること"),
    ))
    .style(style().gap(4))
}

fn task_row(title: &str) -> impl IntoView {
    row((text("-"), text(title))).style(style().gap(8))
}

fn task_list() -> impl IntoView {
    column((
        task_row("牛乳を買う"),
        task_row("チュートリアルを書く"),
        task_row("散歩に行く"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    column((header(), task_list())).style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("タスク").run(app)
}
```

## 新しく出てきたもの

**コンポーネントは普通の関数です。** `header`、`task_row`、`task_list` は、ただの関数です。普通の引数を取れます (`task_row` はタイトルを受け取ります)。戻り値は `impl IntoView` です。Kivact は、画面を組み立てるときにコンポーネントを**一度だけ**呼びます。何かが変わるたびに呼び直すことはありません。これが何を意味するかは、次のステップで分かります。

**`column` と `row`** は、子を縦と横に並べます。子はタプルで渡します (`column((a, b, c))`)。括弧が二重になっているのは、内側がタプルだからで、`column(a, b)` とは書けません。子の型はバラバラでよく、タプルには最大 16 個まで入れられます。

**`style()`** は、見た目とレイアウトを表します。`.style(..)` で view に付けます。

- `gap(8)` は、`column` や `row` の子どうしの間に 8 ピクセルの間隔を空けます。
- `padding(24)` は、view の内側、子の周りに 24 ピクセルの余白を作ります。

長さの単位は論理ピクセルなので、ディスプレイの拡大率に合わせて拡大されます。`style()` のメソッドは、どれも自分自身を返すので、続けて書けます。

`text` には、文字列リテラル、`&str`、`String` を渡せます。

## 実行する

```text
cargo run
```

「タスク」というタイトルの下に「今日やること」、その下に、ハイフンで始まる 3 行が間隔を空けて表示されます。`gap(8)` を `gap(24)` に変えて実行すると、行の間隔が広がるのが分かります。

次へ: [ステップ 3: 状態](03-state.md)
