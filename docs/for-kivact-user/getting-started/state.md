# 3. 状態と反応性

カウンターの値のように後から変わるものは `signal` に入れます。`signal` が返す `Signal<T>` は状態そのものではなく、小さなハンドルです。ハンドルは `Copy` なので、表示とボタンの両方で使えます。

```rust,no_run
use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0_i32);
    let doubled = memo(move || count.get() * 2);

    column((
        text(move || format!("回数: {}", count.get())),
        text(move || format!("2 倍: {}", doubled.get())),
        row((
            button("増やす").on_press(move || count.update(|n| *n += 1)),
            button("戻す").on_press(move || count.set(0)),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("状態の例").run(counter)
}
```

`count.get()` は値を読みます。`text(move || ...)` の内側で読むと、表示がその状態の変化を追います。`set(0)` は別の値なら書き込みます。`update(|n| *n += 1)` は現在値を直接変更し、毎回変更を通知します。`set` を使う型には `PartialEq` が必要です。

`memo(move || count.get() * 2)` は計算結果を表す `Value<i32>` を作ります。元の値が変わっても計算結果が同じなら、その先の更新を止めます。`Value<T>` は読むためのハンドルで、`Signal<T>` のように書き込むものではありません。

## setup の値と追跡する値

部品の関数は、その owner の setup 中に一度呼びます。signal の更新で関数全体を呼び直すことはなく、値を読んだ表示・memo・effect だけが更新されます。`show` の再表示や keyed list の新しい行では新しい owner が作られ、その factory がもう一度走ります。通常の部品関数を分けるだけでは owner は増えません。

React・SolidJS の経験がある場合は、次の対応から始められます。

| 目的 | React | SolidJS | Kivact |
|---|---|---|---|
| 部品の組立て | 状態更新に応じて関数を再実行 | setup で関数を実行 | setup で関数を実行し、Node を返す |
| 書ける状態 | `useState` | `createSignal` | `signal` が返す Copy の `Signal<T>` |
| 派生値 | render 内の式・`useMemo` | `createMemo` | 一箇所の表示なら closure、共有するなら `memo` |
| 副作用 | `useEffect` と依存配列 | `createEffect` と自動追跡 | `effect` が読んだ依存を自動追跡 |
| cleanup | effect が返す関数 | `onCleanup` | `on_cleanup` |
| mount 後の処理 | `useLayoutEffect` など | `onMount` | `on_mount`。host の commit・layout 後 |
| 要素への参照 | `useRef` と `ref` | `ref` | `node_ref()` と `.node_ref(...)` |
| 条件・keyed 一覧 | 条件付き JSX・key | `Show`・`For` | `show`・`for_each` |

`on_cleanup` は setup で登録すると owner の破棄時に動き、effect 内で登録すると次の実行前と破棄時に動きます。`effect` は turn の書込みを flush した後に動きます。表示用の派生値は effect で別の signal に写さず、closure または memo で表します。React の render と同じつもりで、状態を読む closure の中に新しい signal や部品の組立てを置かないでください。

`text(format!("回数: {}", count.get()))` は setup 時の値を固定して渡します。変化を表示したい場合は上の `text(move || ...)`、複数箇所で使う派生値には `memo` を使います。`effect` で派生値を別の signal にコピーする必要はありません。

ハンドルの `Copy` と値の複製は別です。`String` や `Vec` の `get()` はその値を clone します。長いデータを数えるだけなら `state.with(|rows| rows.len())` のように短いクロージャ内で借用できます。借用したまま同じ状態へ書き込むと `BorrowedWrite` になります。必要な判断結果を返し、`with` が戻ってから書き込みます。

## `move` が必要な理由

表示やイベントに渡すクロージャは、部品関数が終わった後にも使われます。`move` は `count` などのハンドルをクロージャへ取り込みます。ハンドルは `Copy` なので、上の例では複数の `move` クロージャに渡せます。

```text
状態を作る → 表示が読む → イベントが書く → 読んだ表示だけが更新される
```

`signal` や `memo` は部品を組み立てる間に作ります。イベント内では既存の状態を `set` / `update` してください。別の画面を作り直す必要がある場合は [条件分岐と一覧](conditional-lists.md) の `show` などを使います。

画面の一部を閉じたときに状態を捨てたい場合は、[条件分岐と一覧](conditional-lists.md) の factory の中で状態を作ります。

次は [入力とイベント](input-events.md) で、文字列の状態を入力欄につなぎます。


## 自分で動かして確認する

`cargo run -p kivact-example-counter` で元の数と派生値（`memo` の `Doubled`）を確認します。上のコードを試した後、計算を 2 倍から 3 倍へ変えて再起動し、一回の操作で両方の表示が更新されることを確かめてください。

`kivact::view::constant(value)` や定数の `.into_value()` は、現在の setup owner に属する読取り専用 `Value` を作ります。定数は graph node や購読を作りません。`Value<T>` は値の型が Copy / Clone でなくてもコピーでき、`with` なら値を借りて読めます。handle をコピーしても owner の寿命は延びず、破棄後や別 App 内での読出しは診断されます。App 外で保持する入力の記述には `Reactive` を使い、利用する setup で束縛します。
