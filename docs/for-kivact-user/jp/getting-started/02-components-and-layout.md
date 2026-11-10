# ステップ 2: コンポーネントとレイアウト

[English](../../en/getting-started/02-components-and-layout.md)

このステップでは、`column` と `row` で画面を組み立て、ヘッダーと仮のタスク一覧を小さなコンポーネントに分けます。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

fn header() -> impl Element {
    column((
        text("タスク"),
        text("今日やること"),
    ))
    .style(style().gap(4))
}

fn task_row(title: impl IntoText) -> impl Element {
    row((text("-"), text(title))).style(style().gap(8))
}

fn task_list() -> impl Element {
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

**view の helper は普通の関数です。** `header`、`task_row`、`task_list` は、呼び出し側が呼んだ時点で実行されます。`task_row` は `impl IntoText` を受け取るので、固定の文字列と、文字を返すリアクティブな値のどちらも渡せます。これらの helper は `impl Element` を返します。修飾できる単一のルートを持つ view なので、呼び出し側で `.style(..)`、`.name(..)` などの標準のノードの修飾を付けられます。分岐や fragment など、修飾できる単一のルートを持たない view を返す場合は `impl IntoView` を使います。

**props を取る helper はコンポーネントにします。** helper が props を取るなら `#[component]` を付け、prop を `Value`、`View`、`Callback` などで宣言します。props のビルダーは Kivact が書くので、手で書く必要はありません（[再利用できるコンポーネントを書く](../how-to-guide/custom-components.md)）。props が無く、呼び出し側がノードの修飾を付け続ける helper は、普通の関数のままでかまいません。

```rust,no_run
use kivact::prelude::*;

/// タスクの 1 行。`#[component]` が props のビルダーを書きます。
#[component]
fn task_row(title: Value<String>) -> impl IntoView {
    row((text("-"), text(title))).style(style().gap(8))
}

fn app() -> impl IntoView {
    column((
        task_row("牛乳を買う"),
        task_row("チュートリアルを書く"),
        task_row("散歩に行く"),
    ))
    .style(style().gap(8))
}
```

`Value<String>` の prop は `&str` もシグナルも受け取るので、`task_row("牛乳を買う")` と書けます。`header` や `task_list` のように props を持たない helper は、普通の関数のままにします。

**セットアップの境界は明示します。** Rust の helper を呼んでも、その状態に新しいライフサイクルは作られません。`App::run(app)` はルートを一度だけセットアップします。状態が変わると、次のステップで扱うリアクティブなクロージャだけが再評価されます。React の render 関数のように、画面全体の関数を毎回呼び直す仕組みではありません。自分専用のシグナル、effect、後片付けを持つ再利用可能な view は `scope(move || { .. })` を返します。

```rust,no_run
use kivact::prelude::*;

fn click_counter() -> impl IntoView {
    scope(move || {
        let count = signal(0);
        column((
            text(move || format!("{} 回押しました", count.get())),
            button("押す").on_press(move || count.update(|n| *n += 1)),
        ))
        .style(style().gap(8))
    })
}
```

クロージャはこの view がマウントされた時に一度だけ実行されます。そこで作った状態は view がアンマウントされると破棄されます。`count` を変えるとセットアップを再実行せずに文字だけが更新されます。この境界は見えるレイアウトの箱を追加しません。返される `View` には修飾できる単一のルートがないため、スタイルは内側の `column` に付けます。シグナルとリアクティブなクロージャは次のステップで使います。

`signal` や `memo` は、App のルート、`scope`、`show`、`dynamic`、`for_each` が実行するセットアップで作ります。`text(move || ..)` や `style(move || ..)` のようなリアクティブなクロージャは変更のたびに再実行されるため、セットアップ済みの値を読みます。

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
