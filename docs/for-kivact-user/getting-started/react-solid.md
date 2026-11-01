# React・SolidJSからKivactへ

Kivactの部品は、画面を組み立てるときに一度実行するRustの関数です。状態の変化では、状態を読んだ表示や計算を更新します。部品関数を状態更新のたびに実行するReactより、SolidJSのsetupとsignalの使い方に近い構造です。

ただしRustでは、値の所有権、クロージャが保持する値の寿命、引数と戻り値の型を明示します。このページでは、慣れた概念との対応と、その違いを順に説明します。通常の入口は `use kivact::prelude::*;` の1行です。描画や任意contextの検索など、専門的なAPIは修飾したpathで使います。

## 概念の対応

| 目的 | React | SolidJS | Kivact |
|---|---|---|---|
| component | 関数componentとJSX | setupで実行するcomponent | 普通の関数から `impl IntoView` を返す。通常は呼出元のsetupで一度実行 |
| props | props object | props | Rustの関数引数。動的な値を受ける部品では `impl IntoValue<T>`（文字列は `impl IntoText`）を使う |
| children | `children` | `props.children` | `V: IntoView` の引数。子ごとの具体的なRust型を受け、viewへ変換できることを制約で示す |
| 子から親への通知 | callback prop | callback prop | closureをそのまま渡す。部品は引数なしを `impl Fn() + 'static`、引数つきを `impl Fn(T) + 'static` で受け、closureの引数の型は推論される（`()` か `Result<(), UiError>` を返せる）。同じcallbackを複数箇所へ渡す例では `Copy` も要求する。アプリで同じ処理を共有するなら `callback(|x| ..)` で `Copy` の `Callback` にする（引数なしは `callback(|_| ..)`） |
| state | `useState` | `createSignal` | `signal(value)`。読む `get/with`、書く `set/update` を同じ `Signal<T>` が持つ |
| 派生値 | render中の計算・`useMemo` | `createMemo` | 一箇所の表示なら値を読むclosure、共有する計算なら `memo` |
| 副作用 | `useEffect` | `createEffect` | `effect`。実行中に読んだ反応値を自動追跡する。依存配列は書かない |
| context | Contextのproviderと `useContext` | contextのproviderと `useContext` | `provide(value, child)` と `context::<T>()`。任意なら `kivact::view::try_context::<T>()` |
| key付き一覧 | key付きの要素列 | `For` | `for_each(items, key, row)`。同じkeyの行とその状態を保つ |
| index単位の一覧 | indexを同一性に使う要素列 | `Index` | `for_index(items, row)`。同じindexの行を保ち、行が読む値を更新する |
| 条件分岐 | 条件付きJSX | `Show` | `show(condition, factory).fallback(factory)`。`Option` なら `show_some` |
| 複数状態からの表示 | `switch` / `match`相当の分岐 | `Switch` / `Match` | `show` の組合せや `dynamic(source, factory)` 内のRustの `match` で複数の分岐を表す |
| 入力に追従する非同期取得 | Effectやデータ取得ライブラリ | `createResource` | `resource(source, fetch)`。sourceの変化で取得し直し、最新要求の結果を採用する |
| 操作から始める非同期処理 | イベントhandlerからの非同期処理 | 操作を起点にした非同期処理 | `action(closure)` をsetupで作り、イベントで `run(())`（引数を受けるclosureなら `run(value)`） |
| 再描画を起こさない値の参照 | 値を保持する `useRef` | 非反応の変数など | `var(value)` の `Var<T>`。`get/with/set/update` は反応的な表示更新を起こさない。`Copy` でない設定値（path など）を複数の closure で共有する箱にも使う |
| 要素への参照 | DOMへの `ref` / `useRef` | 要素の `ref` | `node_ref()` で作った `NodeRef` を `.node_ref(...)` で結び、mount後のfocusや矩形の照会に使う。DOM objectではない |
| formの値 | controlled inputの値と変更callback | 入力の値とsetter | `Binding<T>`。`Signal<T>` を標準入力へ直接渡すか、モデルのfieldなら `model.field(..)`、一覧の一行なら `list.item(key, id)`。`select` のように `Option` の binding を受ける入力へは `.optional()` を付ける。別々の状態を結ぶときだけ `binding(read, write)` |
| キー入力 | `KeyboardEvent.key` / `KeyboardEvent.code` | 同じ | `on_key` の `KeyEvent` の `key`（`Key`。`KeyboardEvent.key` に当たる、配列に依存した論理キー）と `code`（`KeyCode`。`.code` に当たる物理キー）。`for_each` の行の key とは別物 |
| 見た目を持たない振る舞い（部品の作者向け） | react-aria の `useButton`・`useMenu` など `useX` | `@kobalte/core` などの headless 部品 | `kivact::interaction` の `use_x`（`use_button`・`use_menu`・`use_dialog`・`use_list` ...）。アプリは `kivact::ui` と公式 kit を使い、`use_x` は部品を作るときだけ使う |

この対応は用途の対応です。たとえばReact Contextの既定値がそのままKivactにもあるわけではありません。Kivactの `context::<T>()` は未提供ならpanicし、任意の値を探すときは `try_context::<T>()` を使います。

## 部品は普通の関数、変化する値はclosure

次の例は、子を受け取る枠と、親へ通知する部品を組み合わせます。

```rust,no_run
use kivact::prelude::*;

fn panel<V: IntoView>(title: &str, child: V) -> impl IntoView {
    column((text(title), child)).style(style().gap(8).padding(12))
}

fn counter_controls(on_increment: impl Fn() + Copy + 'static) -> impl IntoView {
    row((
        button("+1").on_press(on_increment),
        button("もう一度+1").on_press(on_increment),
    ))
    .style(style().gap(8))
}

fn counter() -> impl IntoView {
    let count = signal(0_i32);
    let doubled = memo(move || count.get() * 2);

    panel(
        "カウンター",
        column((
            text(move || format!("回数: {}", count.get())),
            text(move || format!("2倍: {}", doubled.get())),
            counter_controls(move || count.update(|n| *n += 1)),
        )),
    )
}

fn main() -> kivact::Result<()> {
    App::new("カウンター").run(counter)
}
```

`V` は呼出元が渡す子の型です。`text(...)` と `column(...)`、自分の部品の戻り値などを受けられるよう、`IntoView` を満たす型を引数にします。子は組立てたviewとして一度使います。再表示のたびに新しい子を作る `show` のfactoryとは、寿命の役割が違います。

`counter_controls` は同じcallbackを二つのボタンへ渡すため `Copy` を要求しています。callbackを一箇所へ渡すだけなら、この理由で `Copy` を付ける必要はありません。引数なしの通知を、単位型の引数を取るclosureで包む必要もありません。

`text(format!(...))` と書くと、setup時に計算した値を渡します。更新に追従させるには上の `text(move || ...)` のようにclosureの中で読みます。表示のための派生値を `effect` で別のsignalへ書き写す代わりに、closureか `memo` を使います。`memo` は計算結果が同じなら、その先への更新を止めます。

通常の部品関数を分けただけでは、新しい状態の寿命は作りません。`show` の分岐や `for_each` の行などが寿命の境界になり、そのfactory内で作った状態は分岐や行の破棄とともに解放されます。`dynamic` はsourceが別の値になったときに表示を作り直すため、単に文字を更新したい場合の代用にはしません。

## Rustで押さえる5点

### 1. `move` は後から使うclosureへ値を渡す

表示やイベントのclosureは部品関数が戻った後も使われます。`move` は外側の変数をclosureへ取り込み、関数内のローカル変数を借りたままにしないために使います。表示・イベントに保存するclosureには原則として `'static` が必要です。

`move` がすべての値を複製するわけではありません。取り込む値が `Copy` ならコピーされ、`String` などの所有する値なら所有権が移ります。`Fn()` は繰り返し呼べるcallbackなので、捕まえた `String` を呼ぶたびに外へ移してしまうclosureとは区別します。

### 2. ハンドルの `Copy` と中のデータのcloneは別

`Signal<T>`、`Value<T>`、`Binding<T>`、`Var<T>` は小さな `Copy` ハンドルです。同じハンドルを複数の `move` closureへ渡しても、状態のコピーは作られず、同じ値を指します。ハンドルに `.clone()` を付ける必要はありません。

一方、`signal.get()` は中の `T` をcloneして返すため `T: Clone` が必要です。`String` や `Vec` を数えるだけなら `signal.with(|value| value.len())` で借りられます。借用中に同じsignalへ書かず、必要な結果を返してから更新します。これらのハンドルはUI thread内で使い、別threadへ送信する値とは分けます。

### 3. `Clone + PartialEq` は使う操作の境界に合わせる

`signal(value)` 自体は、すべての `T` に `Clone` や `PartialEq` を要求しません。借用で読む `with`、その場で変更する `update` と、値を複製する `get`、等値を比較する `set` では必要な条件が違います。

アプリのモデルを `get()` して編集し `set()` で戻す場合は、モデルに `#[derive(Clone, PartialEq)]` を付けるのが分かりやすい入口です。下の例の `Form` がそれに当たります。行の値を比較する `for_each` などにも型の条件があります。すべてのsignalデータへ機械的にderiveする規則ではありません。

### 4. 値がない状態は `Option<T>`

`Option<T>` は、値のある `Some(value)` と、ない `None` を表します。未選択、未保存、最後の成功値がまだない状態などに使います。`Option<String>` を読むときは `match`、`if let`、`unwrap_or_else` などで値のない場合も扱います。必須でない値を無条件に `unwrap()` する必要はありません。

`show_some` のfactoryが受け取るのは、生の `T` ではなく、その表示中の値を読む `Value<T>` です。値が引き続き `Some` なら同じ分岐の状態を保ちながら中の値を更新します。

### 5. `String` は所有する文字列、`&str` は借りる文字列

`String` は自分で文字列を所有し、`&str` はどこかの文字列を借ります。`text("こんにちは")` のような直接の文字列propsは、入口でownedな `String` に正規化されます。

動的な文字列propsのclosureは、`String` に変換できる値（`&'static str` や `String`）を返します。文字列リテラルなら `move || if saved.get() { "保存済み" } else { "未保存" }` とそのまま書け、組立てるなら `format!(...)` と書きます。signal内部を借りた `&str` はclosureの外へ返せないので、`with(|value| value.name.clone())` のように所有する結果を返します。

## `()` と `run(())`

`()` は「値がない」ことを表す単位型です。TypeScript の `void`（値を返さない関数の戻り型）に当たりますが、`()` は値として書けて、型としても使えます。Kivact で目にする場所は三つです。

- 入力のない `action` の入力の型です。`action(|| async { .. })` で作った処理は `save.run(())` で始めます。`run` は入力を一つ受ける一つの method なので、入力が要らないときの入力が `()` です。入力のある処理は同じ `run` に値を渡します（`work.run(21)`）。入力がない場合だけ別の method を足さないのは、書き方を一通りに保つためです。
- 何も描かない view です。`()` は `IntoView` を満たし、空の fragment になります。`Option<V>` の `None` も同じ空の view です。
- `callback(|_| ..)` のように、入力のない通知の入力です。`Callback<()>` は `.call(())` で呼びます。

JS の `run()` のように引数を省く書き方はありません。「引数が一つある」ことをコンパイラが確かめられるようにしてあります。

## 分岐の arm の型

Rust の `match` や `if` は、すべての arm が同じ型でなければなりません。JS のように arm ごとに別の種類の値を返すことはできません。`text(..)`・`node()`・`column(..)` などの要素関数はすべて `Node` を返すので、これらだけの arm は揃います。`button(..)` のような部品の builder は部品ごとに別の型なので、`text(..)` と混ぜるとコンパイラが `match arms have incompatible types` と報告します。

`dynamic(source, |value| match value { .. })` の factory はこの状況になりやすい場所です。arm の末尾に `.into_view()` を付けて、どの arm も `View`（型を消した view）にします。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
enum Mode {
    Read,
    Edit,
}

fn editor() -> impl IntoView {
    let mode = signal(Mode::Read);
    column((
        button("切り替え").on_press(move || {
            mode.update(|mode| {
                *mode = match mode {
                    Mode::Read => Mode::Edit,
                    Mode::Edit => Mode::Read,
                }
            })
        }),
        dynamic(mode, |mode| match mode {
            Mode::Read => text("読み取り専用").into_view(),
            Mode::Edit => button("保存").into_view(),
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("分岐の例").run(editor)
}
```

`.into_view()` が要るのはこの形の分岐だけです。ほかの場所では型を揃える必要がありません。

- `column((a, b))` と `row((a, b))` の tuple は、要素ごとに別の型でかまいません（16 個まで）。`Vec` は同じ型の要素を並べます。
- 部品関数は具体的な型を書かず `-> impl IntoView` を返します。
- `show(condition, factory)` と `show_some` は、分岐の片方ずつを別の factory で渡すので、arm の型を揃える必要がありません。

状態に応じて切り替える表示は、まず `show` の組合せで書けないかを見てください。`match` で三つ以上に分ける、または部品の種類が変わるときだけ `dynamic` を使います。

## closure の引数に型が要るとき

closure の引数の型は、渡す先の関数から推論されます。ただし `callback` や `action` のように入力の型が呼出しの側で決まる関数では、closure の中で引数の method を呼ぶとき、rustc が `impl Fn(E)` を通して型を決められないため、引数の型を書く必要があります。

```rust,no_run
use kivact::prelude::*;
use kivact::view::callback;

fn lengths() -> impl IntoView {
    let length = signal(0_usize);
    // closure の中で `word.len()` を呼ぶので、引数の型 `String` を書く。
    let set_length = callback(move |word: String| length.set(word.len()));
    // `action` の入力の型も、`run` の呼出しより先に決める必要がある。
    let work = action(|n: u32| async move { Ok(n * 2) });

    column((
        button("測る").on_press(move || set_length.call("kivact".to_owned())),
        button("倍にする").on_press(move || work.run(21)),
        text(move || format!("{} 文字", length.get())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("引数の型の例").run(lengths)
}
```

引数を使わない closure（`callback(|_| ..)`、`on_press(move || ..)`）や、`on_key(move |event| ..)` のように受け先の関数が引数の型を決めている closure は、型を書かなくてかまいません。型が要るのに書かないと `type annotations needed` と報告されます。そのときは `|word: String|` のように closure の引数へ型を書きます（キー入力なら `callback(|event: KeyEvent| event.key)`）。

## formのbinding、値のref、要素のref

読み書きを部品へ渡す `Binding`、反応しない値を保持する `Var`、要素を指す `NodeRef` は用途が異なります。この例ではモデルの名前だけをbindingにし、保存回数は `Var`、入力欄へのfocus要求は `NodeRef` で扱います。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Form {
    name: String,
    saved_message: Option<String>,
}

fn form() -> impl IntoView {
    let model = signal(Form {
        name: String::new(),
        saved_message: None,
    });
    let saves = var(0_u32);
    let name_ref = node_ref();
    let name = model.field(|value| &value.name, |value| &mut value.name);

    column((
        text_input("名前", name).node_ref(name_ref),
        button("名前へ移動").on_press(move || name_ref.focus()),
        button("保存").on_press(move || {
            saves.update(|count| *count += 1);
            let mut next = model.get();
            next.saved_message = Some(format!("{}回目の保存", saves.get()));
            model.set(next);
        }),
        text(move || {
            model.with(|value| {
                value.saved_message.clone().unwrap_or_else(|| "未保存".to_owned())
            })
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("フォーム").run(form)
}
```

`model.field(get, get_mut)` はモデルの1つのfieldを読み書きする `Binding` です。入力欄はIMEの未確定文字をbindingへ書かず、確定した編集を渡します。独立した文字列状態だけなら、`text_input("名前", signal(String::new()))` のように `Signal<String>` を直接渡せます。別々の状態を結ぶとき（摂氏の入力で華氏の signal を更新する、など）は `binding(read, write)` で読み方と書き戻し方を closure で渡します。

`saves` だけを変更しても表示更新は起こりません。上の例で保存文言が変わるのは `model.set(next)` によります。Reactの `useRef` の「値を保持し、変更自体では再描画しない」用途が `Var` に対応します。

`name_ref` はRustのデータを入れる箱ではありません。`.node_ref(name_ref)` で入力欄と結び、mount後のイベントで `focus()` を要求します。DOM objectを取り出して書き換えるAPIではありません。必要なら `rect()` などの公開された照会を使います。

## contextと非同期処理

`provide` は子孫に型で値を提供します。必須の `context::<T>()` は最寄りの提供値を返し、見つからなければ呼出位置と型名を持つpanicになります。未提供が通常のケースなら `kivact::view::try_context::<T>()` の `Option` を扱います。任意検索も、破棄済みの状態を有効にするための口ではありません。

次は、提供された状態をresourceのsourceとして読み、操作から始めるactionと分ける最小例です。非同期部分は説明用に即座に値を返しています。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy)]
struct Session {
    name: Signal<String>,
}

#[derive(Clone, Copy)]
struct Hint(&'static str);

fn profile() -> impl IntoView {
    let session = context::<Session>();
    let hint = kivact::view::try_context::<Hint>();
    let greeting = resource(session.name, |name| async move { Ok(format!("こんにちは、{name}")) });
    let save = action(|| async { Ok(()) });

    column((
        text(hint.map_or("ヒントなし", |value| value.0)),
        show(move || greeting.state().is_pending(), || text("読込中…")),
        show_some(move || greeting.error(), |error| text(move || error.get().to_string()).role(Role::Alert)),
        show_some(move || greeting.latest(), |text_value| text(text_value)),
        button("保存")
            .disabled(move || save.state().is_pending())
            .on_press(move || save.run(())),
    ))
}

fn app() -> impl IntoView {
    let session = Session { name: signal("Kivact".to_owned()) };
    provide(session, profile)
}

fn main() -> kivact::Result<()> {
    App::new("プロフィール").run(app)
}
```

`resource` はsourceが読む状態の変化に追従し、古い要求の結果を新しい要求へ上書きしません。`action` はイベントの `run(())` から始まります。引数を受けるclosure（`action(|draft| async move { .. })`）なら `run(value)` で渡します。いずれもsetup中に作り、イベント内で作り直しません。

`state()` は `AsyncState`、`is_pending()` はその処理中判定、`latest()` は最後に成功した `Option<T>`、`error()` は失敗の誤り（`Option<UiError>`）です。失敗のときだけ出す表示は `show_some(move || greeting.error(), ..)` と書けます。最後の成功値は、次の処理中や失敗時にも残ります。実際の通信や保存では `AsyncState::Failed` も表示で扱い、処理中だけの表示で失敗を隠さないようにします。失敗は状態に保存されるため、error boundaryで囲むだけで失敗表示が作られるわけではありません。

さらに詳しくは、[状態と反応性](state.md)、[入力とイベント](input-events.md)、[条件分岐と一覧](conditional-lists.md)、[非同期処理](async.md)へ進んでください。

