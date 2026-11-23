# ステップ 2: コンポーネントとレイアウト

[English](../../en/getting-started/02-components-and-layout.md)

このステップでは、`column` と `row` で画面を組み、小さなコンポーネントに分けます。ヘッダーと、仮のタスク一覧です。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(title: Value<String>) -> impl Element {
    row((text("-"), text(title))).style(style().gap(8))
}

#[component]
fn task_list() -> impl Element {
    column((
        task_row("Buy milk"),
        task_row("Write the tutorial"),
        task_row("Go for a walk"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    column((header(), task_list())).style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

## 新しいこと

**コンポーネントは `#[component]` を付けた関数です。** `header`、`task_row`、`task_list` がコンポーネントです。コンポーネントを書く方法はこれだけです。`button` などの標準のコンポーネントや、UI キットのコンポーネントも同じ書き方をします。コンポーネントは `impl Element` を返します。これは、呼び出し側が変更できる、ルートが 1 つの view です。`task_list().style(..)`、`.name(..)`、`.test_id(..)` は、その中の `column` に適用されます。

**props は引数です。** `task_row` は `title: Value<String>` を受け取ります。`Value` の prop は、固定の値（`"Buy milk"` や `String`）も、変わる値（シグナル、または次のステップから使うクロージャ）も受け取れるので、`task_row("Buy milk")` と書けます。引数は位置で対応します。順番はシグネチャの宣言の順です。`Option<..>` 型の引数は省略できる引数で、代わりに `.on_close(..)` のような setter になります。詳しくは [再利用できるコンポーネントを書く](../how-to-guide/custom-components.md) を参照してください。

**本体は一度だけ実行されます。** Kivact は、コンポーネントが画面に現れたときに本体を一度だけ実行し、それ自身の寿命を与えます。本体の中で作った状態は、そのインスタンスだけのものになり、消えるときに後片付けされます。状態が変わると、それを読む小さなクロージャだけが再実行されます（次のステップで扱います）。本体が再実行されることはありません。これは React の render 関数の仕組みではなく、SolidJS の仕組みです。

```rust,no_run
use kivact::prelude::*;

#[component]
fn click_counter() -> impl Element {
    let count = signal(0);
    column((
        text(move || format!("Pressed {} times", count.get())),
        button("Press me").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    row((click_counter(), click_counter())).style(style().gap(24).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Counters").run(app)
}
```

それぞれの `click_counter` は、自分の `count` を持ちます。一方のボタンを押すと、その隣の文字だけが更新されます。シグナルとこれらのクロージャは、次のステップで使います。

シグナルと memo は、セットアップのコードで作ります。セットアップのコードとは、コンポーネントの本体、ルートの関数、そして `show`、`dynamic`、`for_each` が実行するコードのことです。`text(move || ..)` のようなクロージャは変更のたびに再実行されるので、セットアップで作られた値を読みます。

**ルートは普通の関数です。** `App::new("Tasks").run(app)` は、`app` を一度呼び、返された view を表示します。`run` には、view を返す関数であればどれでも渡せます。このチュートリアルのルートは `app` という普通の関数のままです。その下にあるものは、すべてコンポーネントです。

**`column` と `row`** は、子を縦と横に並べます。子はタプルで渡します (`column((a, b, c))`)。括弧が二重になっているのは、内側がタプルだからで、`column(a, b)` とは書けません。子の型はバラバラでよく、タプルには最大 16 個まで入れられます。

**`style()`** は、view の見た目とレイアウトを表します。`.style(..)` で view に付けます。

- `gap(8)` は、`column` や `row` の子どうしの間に 8 ピクセルの間隔を空けます。
- `padding(24)` は、view の内側、子の周りに 24 ピクセルの余白を作ります。

名前は CSS にならっています。`gap`、`padding`、`flex_grow`、`border_radius`、`font_weight` などです。長さの単位は論理ピクセルなので、ディスプレイの拡大率に合わせて拡大されます。`style()` のメソッドは、どれも自分自身を返すので、続けて書けます。

`text` には、文字列リテラル、`&str`、`String`、`Value<String>` を渡せます。

## 実行する

```text
cargo run
```

「Tasks」というタイトルの下に「Things to do today」が表示され、その下に、ハイフンで始まる 3 行が間隔を空けて表示されます。`gap(8)` を `gap(24)` に変えて実行すると、行の間隔が広がるのが分かります。

次へ: [ステップ 3: 状態](03-state.md)
