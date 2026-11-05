# ステップ 2: コンポーネントとレイアウト

[English](../../en/getting-started/02-components-and-layout.md)

このステップでは、`column` と `row` で画面を組み立て、ヘッダーと仮のタスク一覧を小さなコンポーネントに分けます。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

fn header() -> impl Component {
    column((
        text("タスク"),
        text("今日やること"),
    ))
    .style(style().gap(4))
}

fn task_row(title: impl IntoText) -> impl Component {
    row((text("-"), text(title))).style(style().gap(8))
}

fn task_list() -> impl Component {
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

**コンポーネントは普通の関数です。** `header()` や `task_row(...)` を呼ぶと、その場で view の記述を作ります。`task_row` の `impl IntoText` は、固定の文字列と、文字を返すリアクティブなクロージャのどちらも受け取れます。

この 3 つの関数は、単一のルートを持つ view を返すので、戻り値を `impl Component` にしています。`Component` は `IntoView` に加えてノードの修飾に対応し、呼び出し側でも `header().style(...)` のように書けます。表示するものを返すだけでよければ `impl IntoView` を使います。

`App::run(app)` は `app` を画面のセットアップとして一度呼びます。その中で普通の関数を呼んでも、新しい独立したライフサイクルは作られません。状態が変わると、次のステップで扱うリアクティブなクロージャが再評価されます。React の render 関数のように、画面全体の関数を毎回呼び直す仕組みではありません。

**`column` と `row`** は、子を縦と横に並べます。子はタプルで渡します (`column((a, b, c))`)。括弧が二重になっているのは、内側がタプルだからで、`column(a, b)` とは書けません。子の型はバラバラでよく、タプルには最大 16 個まで入れられます。

**`style()`** は、見た目とレイアウトを表します。`.style(..)` で view に付けます。

- `gap(8)` は、`column` や `row` の子どうしの間に 8 ピクセルの間隔を空けます。
- `padding(24)` は、view の内側、子の周りに 24 ピクセルの余白を作ります。

長さの単位は論理ピクセルなので、ディスプレイの拡大率に合わせて拡大されます。`style()` のメソッドは、どれも自分自身を返すので、続けて書けます。

`text` には、文字列リテラル、`&str`、`String` を渡せます。

## マウント時にセットアップする

子が画面に取り付けられる時点まで処理を遅らせ、その子専用の状態や後片付けを持たせたいときは `component(move || ...)` を使います。

```rust,no_run
use kivact::prelude::*;

fn counter_panel() -> impl IntoView {
    component(move || {
        let count = signal(0);
        column((
            text(move || format!("回数: {}", count.get())),
            button("増やす").on_press(move || count.update(|n| *n += 1)),
        ))
        .style(style().gap(8))
    })
}
```

`counter_panel()` を呼んだだけでは、クロージャの中はまだ実行されません。返された view がマウントされると一度だけ実行され、そこで作った状態、effect、後片付けは、その子のライフサイクルに結び付きます。子が取り除かれると一緒に破棄されます。`component` は箱のない `View` を返すため、この例のスタイルは内側の `column` に付けています。

`signal` や `memo` の初期化は、App のルート、`component`、`show`、`dynamic`、`for_each` が実行するセットアップに置きます。再計算される `text(move || ...)` や `style(move || ...)` の中では、セットアップ済みの値を読みます。

## 実行する

```text
cargo run
```

「タスク」というタイトルの下に「今日やること」、その下に、ハイフンで始まる 3 行が間隔を空けて表示されます。`gap(8)` を `gap(24)` に変えて実行すると、行の間隔が広がるのが分かります。

次へ: [ステップ 3: 状態](03-state.md)
