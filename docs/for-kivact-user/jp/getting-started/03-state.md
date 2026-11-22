# ステップ 3: 状態

[English](../../en/getting-started/03-state.md)

このステップでは、残りのタスク数を表示するカウンターと、その数を変えるボタンを加えます。状態が変わると、画面は自動的に更新されます。

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
    let completed = signal(0);
    let left = memo(move || 3 - completed.get());

    column((
        header(),
        task_list(),
        text(move || format!("{} tasks left", left.get())),
        button("Complete one")
            .disabled(move || left.get() == 0)
            .on_press(move || completed.update(|n| *n += 1)),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

一覧はまだ仮のものです。カウンターとボタンは、本物の一覧がステップ 5 で届くまで、状態の扱い方を練習するための仮の部品です。

## 新しいこと

**`signal(0)`** は、`0` から始まる状態を作ります。返り値は `Signal<i32>` で、数そのものではなく、状態への小さなハンドルです。ハンドルは `Copy` なので、いくつのクロージャに渡しても、何もコピーせずに済みます。`Signal` は、書き込みに使う唯一のハンドルです。ステップ 4 の入力欄も、ステップ 5 のチェックボックスも `Signal` を受け取ります。

シグナルには、次のようなメソッドがあります。

- `completed.get()` は、現在の値を読みます。
- `completed.set(5)` は、値を置き換えます。
- `completed.update(|n| *n += 1)` は、値をその場で変更します。新しい値が古い値に依存するときに使います。
- `completed.with(|n| ..)` は、値をコピーせずに借ります。一覧のような大きな値に便利で、ステップ 5 で使います。

**状態に従うクロージャ。** `text("Tasks")` は、固定の文字列を表示します。`text(move || format!(..))` は、代わりにクロージャを受け取ります。Kivact はそのクロージャを実行し、`get()` で読んだシグナルを覚えます。そのどれかが変わると、クロージャをもう一度実行します。更新されるのは、その文字の部分だけです。`.disabled(move || ..)` のような props でも同じことができます。

**`memo`** は、ほかの状態から値を計算します。`left` は `3 - completed` です。`completed` が変わると再計算され、`left` を読むクロージャは、結果が変わったときだけ更新されます。返り値は `Value<i32>` で、読むことはできても書くことはできないハンドルです。複数の場所で読まれる値には `memo` を使います。読む場所が 1 つだけなら、クロージャで十分です。

**`button(..).on_press(..)`** は、ボタンを作ります。利用者が押したとき（クリック、またはフォーカスがあるときの Enter キーや Space キー）に、クロージャを実行します。ボタンのラベルは、そのままアクセシブルな名前にもなります。

**なぜ `move` が必要か。** これらのクロージャは、`app` が返った後も長く使われるので、ローカル変数を借用することはできません。`move` は、ハンドルをクロージャの中へコピーします。`Signal` は `Copy` なので、同じ `completed` を複数の `move` クロージャに渡せます。`move` がないと、借用された値より長く生きるかもしれないとコンパイラが報告します。

**セットアップは、マウントごとに一度だけ実行されます。** `App::run` はルートを一度呼び、各コンポーネントの本体は、そのコンポーネントが現れたときに一度だけ実行されます。状態が変わると、その状態を読む小さなクロージャが再実行されます。セットアップは再実行されません。シグナルと memo はセットアップの間に作り、ハンドラーはそこで作ったハンドルを更新します。

## React と Solid との対応

React や Solid に慣れている場合は、名前を一対一で置き換えるのではなく、値が果たす役割で選んでください。prelude には、よく使う名前がすでに含まれています。

| React / Solid の概念 | Kivact |
| --- | --- |
| `useState` / `createSignal` | `signal(initial)`。読むには `get` か `with`、書くには `set` か `update` |
| `useMemo` / `createMemo` | 共有する派生値には `memo(move \|\| ..)` |
| 値の prop や accessor | `Value<T>` の prop は、定数、シグナル、memo、クロージャを受け取る |
| 制御された入力 | `Signal<T>` を渡す。`bind!(form.name)` で、シグナルの中の構造体の 1 つのフィールドを編集する |
| イベントの prop | クロージャ、または `callback(..)` で作る共有可能な `Callback<E>` |
| `useRef` | `var(initial)`。画面が監視しない箱 |
| `useEffect` / `createEffect` | `effect(move \|\| ..)`。読んでいる状態が変わったとき、UI の外のものと同期させる。表示する値は `memo` で計算する |
| 効果とタイマーを持つ `useReducer` | `Machine` は関連する状態とイベントをまとめて持つ。遷移は、任意の効果と期限を持つ `Step` を返す |

このカウンターに必要なのは `signal` と `memo` だけです。独立した編集可能な値ごとにシグナルを使い、そこから導かれる値には memo を使います。いくつかのフラグが 1 つの操作の段階（押し続け、待機、実行、キャンセルなど）を表すときは、複数のシグナルを同期させる代わりに、1 つの Machine を使います。Machine の定義には `kivact::state::{Machine, Step, machine}` を使います。より詳しい説明は、[React と Solid のガイド](../how-to-guide/coming-from-react-or-solid.md) にあります。

## 実行する

```text
cargo run
```

「3 tasks left」と「Complete one」ボタンが表示されます。押すたびに数が 1 ずつ減ります。「0 tasks left」になると、ボタンは無効になります。

次へ: [ステップ 4: 入力](04-input.md)
