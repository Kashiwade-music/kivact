# ステップ 3: 状態

[English](../../en/getting-started/03-state.md)

このステップでは、残りタスク数のカウンターと、それを変えるボタンを加えます。状態が変わると、画面は自動で更新されます。

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
    let completed = signal(0);
    let left = memo(move || 3 - completed.get());

    column((
        header(),
        task_list(),
        text(move || format!("残り {} 件", left.get())),
        button("1 件完了")
            .disabled(move || left.get() == 0)
            .on_press(move || completed.update(|n| *n += 1)),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("タスク").run(app)
}
```

一覧はまだ仮のものです。カウンターとボタンは、ステップ 5 で本物の一覧を作る前に、状態の使い方を練習するための仮のものです。

## 新しく出てきたもの

**`signal(0)`** は、初期値 `0` の状態を作ります。戻り値は `Signal<i32>` で、数値そのものではなく、状態への小さなハンドルです。ハンドルは `Copy` なので、明示的な `clone()` なしで、いくつものクロージャに渡せます。どのハンドルも同じ状態を読み書きします。

シグナルは、次のメソッドで使います。

- `completed.get()` は、現在の値を読みます。
- `completed.set(5)` は、値を置き換えます。
- `completed.update(|n| *n += 1)` は、値をその場で変更します。新しい値が古い値に依存するときに使います。
- `completed.with(|n| ..)` は、値をコピーせずに借用します。一覧のような大きな値に便利で、ステップ 5 で使います。

**リアクティブなクロージャ。** `text("タスク")` は固定の文字列を表示します。`text(move || format!(..))` はクロージャを受け取ります。Kivact はクロージャを実行し、`get()` で読んだシグナルを記録して、そのどれかが変わるたびにもう一度実行します。更新されるのは、その 1 か所の文字だけです。`.disabled(move || ..)` のようなプロパティでも同じように使えます。

**`memo`** は、ほかの状態から値を計算します。`left` は `3 - completed` です。`completed` が変わると再計算され、結果が変わったときだけ、`left` を読んでいるクロージャが更新されます。複数の場所が読む値には `memo` を使います。1 か所だけなら、クロージャで十分です。

**`button(..).on_press(..)`** は、ボタンを作り、押されたときにクロージャを実行します。クリックのほか、フォーカスがあるときの Enter キーや Space キーでも押せます。ボタンのラベルは、アクセシブルな名前も兼ねます。

**なぜ `move` なのか。** これらのクロージャは、`app` が戻った後でも使われるため、ローカル変数を借用できません。`move` は、ハンドルをクロージャの中へコピーします。`Signal` は `Copy` なので、同じ `completed` を複数の `move` クロージャに渡せます。`move` がないと、借用した値よりクロージャが長く生きる可能性がある、というコンパイルエラーになります。

**セットアップはマウントごとに一度だけ実行されます。** `App::run` はルートのセットアップを一度だけ呼びます。普通の helper は呼び出し側のセットアップの中で実行され、`scope(move || ..)` はステップ 2 で紹介した明示的なセットアップと後片付けの境界を作ります。React の render 関数より Solid のコンポーネントに近い仕組みです。状態が変わると、セットアップ全体ではなく、その状態を読む小さなリアクティブな式だけが再実行されます。シグナル、memo、binding はセットアップの中で作り、イベントハンドラはそこで作ったハンドルを更新します。

## React／Solid との対応

React や Solid を知っている場合、名前を一対一で置き換えるより、値の役割で選ぶと整理しやすくなります。よく使う値、binding、callback、effect の名前は prelude に含まれています。

| React／Solid の概念 | Kivact |
| --- | --- |
| `useState`／`createSignal` | `signal(initial)`。`get` か `with` で読み、`set` か `update` で書く |
| `useMemo`／`createMemo` | 共有する派生値には `memo(move || ..)` |
| 値の prop や accessor | `impl IntoValue<T>` は定数、シグナル、memo、クロージャを受け取る。`Value<T>` は読み取り専用のハンドル |
| 制御された入力 | `Binding<T>` が読み取りと書き込みをつなぐ。シグナルをそのまま渡せる |
| イベントの prop | クロージャ、または `callback(..)` で作って保持する `Callback<T>` |
| `useEffect`／`createEffect` | `effect(move || ..)` は読んだ状態の変化に応じて UI の外側と同期する。表示する値の計算には `memo` を使う |
| `useReducer` に effect と timer を加えたもの | `Machine` は関係する状態とイベントをまとめる。遷移は `Step` を返し、effect や期限を付けられる |

このカウンターには `signal` と `memo` で十分です。独立して編集する値はそれぞれのシグナルに置き、そこから導く値には memo を使います。長押し、待機、実行、キャンセルのように、複数のフラグが一つの操作の段階を表す場合は、複数のシグナルを同期させず、一つの Machine を使います。Machine の定義には `kivact::state::{Machine, Step, machine}` を使います。詳しくは [React／Solid から来た人向けのガイド](../how-to-guide/coming-from-react-or-solid.md) を参照してください。

## 実行する

```text
cargo run
```

「残り 3 件」と「1 件完了」ボタンが表示されます。押すたびに、数が 1 つずつ減ります。「残り 0 件」になると、ボタンは無効になります。

次へ: [ステップ 4: 入力](04-input.md)
