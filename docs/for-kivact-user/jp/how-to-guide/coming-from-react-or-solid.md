# React や SolidJS の知識を Kivact に対応づける

[English](../../en/how-to-guide/coming-from-react-or-solid.md)

React や SolidJS を知っているなら、Kivact の概念のほとんどには対応するものがあります。このガイドでは、それらを 1 対 1 で対応づけます。大きな違いは 1 つだけです。コンポーネントは、状態が変わるたびに実行される React と違い、Solid と同じく一度だけ実行されます。また、Rust を書いたことのない開発者がつまずきやすい細かい点も扱います。

[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

## 唯一の考え方: セットアップは一度だけ実行される

Kivact のコンポーネントは、`#[component]` を付けた関数です。画面に現れるときに**一度だけ**実行されます。状態を作り、部品をつなぎ、view を返します。その後、関数の本体が再び実行されることはありません。シグナルが変わると、それを読んでいる画面の部分だけが更新されます。

```tsx
// React: クリックのたびに関数全体が再実行される。
function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>count: {count}</button>;
}
```

```rust,test_harness
use kivact::prelude::*;

// Kivact: the body runs once; only the `text` closure runs again.
#[component]
fn counter() -> impl Element {
    let count = signal(0);
    column((
        text(move || format!("count: {}", count.get())),
        button("+1").on_press(move || count.update(|c| *c += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_reading_closure_updates() {
        let screen = kivact::test::render(counter);
        screen.get_by_text("count: 0");
        screen.get_by_role(Role::Button, "+1").click();
        screen.get_by_text("count: 1");
    }
}
```

これは SolidJS と同じモデルです。ここから次の 2 つのことが言えます。

- **値が変わる場所には、クロージャーを渡します。** `text(format!("count: {}", n))` は、組み立てのときに `n` を一度だけ読み、そのテキストを表示し続けます。`text(move || ...)` は、`n` が変わるたびに Kivact が再実行するクロージャーの中で `n` を読みます。`.disabled(..)`、`.style(..)`、`.name(..)` など、変わりうるプロパティも同じです。クロージャー、シグナル、memo のどれかを渡してください。
- **依存配列はありません。** クロージャー、memo、effect の中で `.get()` や `.with()` を使って読んだものは、自動的に追跡されます。

## 概念の対応表

| 概念 | React | SolidJS | Kivact |
| --- | --- | --- | --- |
| コンポーネント | JSX を返す関数 | JSX を返す関数 | `#[component] fn card(..) -> impl Element`。props の有無は問わない |
| props | props オブジェクト | props オブジェクト | コンポーネントの引数。変わりうる値は `Value<T>`、コンポーネントが書き込む値は `Signal<T>`、子は `View`、ハンドラーは `Callback<E>`。必須の引数は位置で渡し、`Option<..>` の引数は setter になる |
| 子 | `children` | `props.children` | `View` の prop（スロット 1 つ）、`Vec<View>`（複数）、`ViewFn`（開くたびに作り直す） |
| コールバックの prop | `onSave` prop | `onSave` prop | 入力のない `Callback<()>` の prop は、クロージャーか `Callback<()>` を受け取る。入力のある `Callback<E>` の prop は、クロージャー `move \|e\| ..` を受け取る |
| 共有するハンドラー | `useCallback` | 関数 | `callback(move \|_\| ..)` は `Copy` な `Callback` を作る。複数のハンドラーに渡せる |
| 状態機械 | `useReducer` と effect | reducer と effect | `Machine`: 明示的な状態、イベント、effect、タイマー |
| 状態 | `useState` | `createSignal` | `signal(value)`。1 つの `Signal<T>` が `get`、`with`、`set`、`update` を持つ |
| 双方向の値 | `value` と `onChange` | `value` と `onInput` | `Signal<T>` を渡す。構造体の 1 つの field には `bind!(form.name)`、変換した値には `binding(get, set)` |
| 派生値 | render の中で計算、`useMemo` | `createMemo` | 1 か所で使うならクロージャー、共有やキャッシュなら `memo(..)` |
| effect | `useEffect` | `createEffect` | `effect(..)`。最初の実行を飛ばすなら `effect_on(source, f)` |
| クリーンアップ | `useEffect` の戻り値 | `onCleanup` | `on_cleanup(..)` |
| マウント後に一度だけ実行 | `useEffect(.., [])` | `onMount` | `on_mount(..)` |
| 再描画を起こさない可変の値、アプリのオブジェクト | `useRef` | 通常の変数 | `var(value)`: 画面が監視しない `Copy` の箱 |
| 値のない変化 | state の中のカウンター | `createTrigger` | `trigger()`。`track()` と `notify()` |
| この行が選ばれているか | 各行で `selected === id` | `createSelector` | `kivact::state::selector(source)`、各行で `is(&key)` |
| 要素の参照 | DOM ノードへの `useRef` | `ref` | `node_ref()` と `.node_ref(r)`。`r.focus()` |
| context | `createContext`、`useContext` | `createContext`、`useContext` | `provide(value, \|\| child)` と `use_context::<T>()`。後者は値そのものを返す |
| Portal | `createPortal(child, el)`。新しいウィンドウは `window.open` と portal | `<Portal>` | 同じウィンドウの中は `overlay(..)`、OS のウィンドウは `window(title, open)` |
| 条件 | `cond && <A/>` | `<Show when>` | `show(cond, \|\| a)`、`show_some(option, \|v\| a)` |
| 複数の分岐 | 三項演算子か `switch` | `<Switch>` | `dynamic(source, \|v\| match v { .. })` |
| 一覧 | key 付きの `items.map(..)` | `<For>` | `for_each(items, \|i\| key, \|item\| row)` |
| 位置による一覧 | `key={index}` | `<Index>` | `for_index(items, \|i, item\| row)` |
| key | `key` prop | `For` の同一性 | `for_each` に渡す key の関数 |
| エラー境界 | `ErrorBoundary` | `ErrorBoundary` | `error_boundary(..)` |
| データの取得 | ライブラリーか `useEffect` | `createResource` | `resource(source, fetch)`。利用者の操作には `action(f)` |
| 読み込み中とエラーの表示 | `<Suspense>` と `<ErrorBoundary>` | 同じ | `async_view(resource, ready).loading(..).fallback(..)` |
| 別のスレッドからのデータ | ワーカーからの `postMessage` | 同じ | `kivact::task::remote(initial)`。`Signal` と `Send` な sender |
| 2 つの場所で共有する要素（Framer Motion の `layoutId`） | `layoutId`（Framer Motion） | ライブラリー | 2 つのノードに `.layout_id("id")`。新しいほうは古い箱からレイアウトのトランジションで滑らかに移動する |
| 古い内容が消えてから新しい内容が入る（`AnimatePresence mode="wait"`） | `AnimatePresence mode="wait"` | ライブラリー | `show(cond, \|\| a).fallback(\|\| b).presence(PresenceMode::Wait)`。ほかのモードは `Sync` と `PopLayout` |
| スクロールに追従する値（`useScroll`） | `useScroll` | scroll のリスナー | `scroll_progress(scroller, Orientation::Vertical)`。1 つの要素なら `view_progress(target, scroller, axis, ViewRange::Entry)` |
| 1 つの時計で進む一連の動き（GSAP の `timeline`） | `gsap.timeline()` | ライブラリー | `timeline()`、`track(..)`、`to(track, target, transition).at(At::AfterPrevious(ms))`、`play()` |
| 一覧の項目ごとの遅延（GSAP の `stagger`） | `stagger` | ライブラリー | `Stagger::new(ms).from(StaggerFrom::Center)` と `stagger.delay(index, count)` |
| CSS の `var(--accent)` | `var(--accent)` | 同じ | `color::ACCENT`、`space::MD`、`radius::MD`。値を置きたい場所にそのまま書くテーマ変数 |
| CSS の `:hover`、`:active`、`:focus-visible` | 同じ | 同じ | `style().hover(..)`、`.active(..)`、`.focus_visible(..)` |
| CSS の `transform: perspective(..) rotateX(..)` | `style={{ transform }}` | 同じ | `style().transform(Transform::new().perspective(..).rotate_x(..))` |
| CSS の `filter: blur(4px)` | `style={{ filter }}` | 同じ | `style().filter(Filter::new().blur(4))`（`kivact::style`） |
| CSS の `mask`、`clip-path`、`mix-blend-mode` | `style={{ mask }}` など | 同じ | `style().mask(..)`、`.clip_path(shape)`、`.mix_blend_mode(BlendMode::Multiply)` |
| CSS の `font-variation-settings`、`font-stretch` | `style={{ fontVariationSettings }}` | 同じ | `style().font_variation_settings([("wght", 650.0)])`、`.font_width(75.0)` |
| CSS の `-webkit-text-stroke`、`paint-order` | `style={{ WebkitTextStroke }}` | 同じ | `style().text_stroke(width, color)`、`.paint_order(..)` |

表に出てくる普通の名前は、`use kivact::prelude::*;` で使えます。`callback`、`trigger`、`error_boundary`、`effect_on` も含みます。`selector` は `kivact::state` から、`remote` と `remote_trigger` は `kivact::task` から import してください。状態機械の `Machine`、`Step`、`machine` は `kivact::state` から import します。

`Machine` は、effect とタイマーを持つ `useReducer` です。明示的な状態とイベントを定義し、`transition` から `Step` を返し、runner で effect を実行します。[独自のコントロール](custom-controls.md#状態機械で振る舞いを書く)を参照してください。

## props、子、コールバック

コンポーネントの props は、その引数です。Kivact が署名から props のビルダーを書きます。

```rust,test_harness
use kivact::prelude::*;

// `Value<String>`: a literal, a String, a signal or a closure that returns text.
// `Value<i32>`: a number, a signal, a memo or a closure.
#[component]
fn stat(label: Value<String>, value: Value<i32>) -> impl Element {
    row((text(label), text(move || value.get().to_string()))).style(style().gap(8))
}

// `View`: any view, including another component. `Option<View>`: an optional slot.
#[component]
fn panel(title: Value<String>, child: View, footer: Option<View>) -> impl Element {
    column((text(title), child, footer)).style(style().gap(8).padding(12))
}

// A callback prop: the caller passes a closure or a `Callback`.
#[component]
fn buttons(on_add: Callback<()>) -> impl Element {
    row((button("Add").on_press(on_add), button("Add again").on_press(on_add)))
}

fn app() -> impl IntoView {
    let hits = signal(0);
    panel(
        "Stats",
        column((
            stat("Fixed", 42),
            stat("Signal", hits),
            stat("Derived", move || hits.get() * 2),
            buttons(move || hits.update(|h| *h += 1)),
        )),
    )
    .footer(text("Counts start again when the app restarts."))
    .style(style().width(320))
}

fn main() -> kivact::Result<()> {
    App::new("Props").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_prop_type_takes_constants_signals_and_closures() {
        let screen = kivact::test::render(app);
        screen.get_by_text("42");
        assert_eq!(screen.all_by_text("0").len(), 2);
        screen.get_by_text("Counts start again when the app restarts.");
        screen.get_by_role(Role::Button, "Add again").click();
        screen.get_by_text("1");
        screen.get_by_text("2");
    }
}
```

- 変わりうる prop は `Value<T>` です。呼び出し側は定数、シグナル、memo、クロージャーのどれでも渡せ、本体は `Value<T>` を受け取って `.get()` や `.with(..)` で読みます。
- コールバックの prop は `Callback<E>` です。入力のない動作（`Callback<()>`）は、呼び出し側がクロージャーか、すでに持っている `Callback<()>` を渡します。入力のあるコールバックは、呼び出し側がクロージャーを渡します。共有している `Callback<E>` は `move |e| cb.call(e)` と書いて渡します。本体は `Copy` のハンドルを受け取るので、2 つのボタンが同じ動作を使えます。`Callback<()>` は任意の `on_press` に渡せ、どのコールバックも `.call(event)` で呼べます。
- `Option<..>` の prop は省略できます。呼び出し側は同じ名前の setter、たとえば `.footer(..)` で指定します。`#[prop(default = expr)]` を付けると、既定値を持つ prop になります。
- `()` を返すクロージャーを受け取る場所では、`Result<(), UiError>` を返すクロージャーも受け取れます。ハンドラーの中で `?` を使えます。エラーは、いちばん近いエラーハンドラーに届きます。
- コンポーネントは `impl Element` を返すので、呼び出し側は `app` の `panel` のように `.style(..)`、`.name(..)`、`.test_id(..)` を付けられます。呼び出し側のスタイルは、コンポーネント自身のスタイルの後に適用され、両方が同じプロパティを設定したら呼び出し側が勝ちます。
- コンポーネントは型引数を持てます（`fn picker<K: Clone + PartialEq + 'static>(..)`）。React なら任意の子を受け取るところでは、`View`、`Vec<View>`、`Value<T>` の prop を受け取ってください。

## effect、クリーンアップ、マウント

```rust,no_run
use kivact::prelude::*;

#[component]
fn tracker(query: Signal<String>) -> impl Element {
    // Runs once the screen is built, then again whenever `query` changes.
    effect(move || {
        println!("searching for {}", query.get());
    });
    // Runs once, after the nodes of this component exist.
    on_mount(|| println!("mounted"));
    // Runs when this component is removed (a branch closes, a row disappears).
    on_cleanup(|| println!("gone"));

    text(move || format!("query: {}", query.get()))
}

fn main() -> kivact::Result<()> {
    App::new("Effects").run(|| tracker(signal("rust".to_owned())))
}
```

- 外部への保存、ログ出力、UI の外との通信などの副作用には `effect` を使います。ある signal の値を別の signal にコピーして派生値を表示するために使ってはいけません。その場合は、クロージャーか `memo` の中で元の値を読んでください。
- effect の中で呼んだ `on_cleanup` は、effect が再び実行される前と、取り除かれるときに実行されます。React で effect が返す関数に相当します。
- 状態は、いちばん近い**境界**に属します。境界とは、コンポーネント、ルート、`show` の分岐、`for_each` の行、`provide`、`error_boundary` のことです。境界の中で作ったシグナルは、その境界が無くなるときに解放されます。

## 条件、分岐、一覧

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let logged_in = signal(false);
    let user = signal(None::<String>);
    let names = signal(vec!["Ann".to_owned(), "Bob".to_owned()]);

    column((
        // `<Show when=.. fallback=..>`
        show(logged_in, || text("Welcome back")).fallback(|| text("Please sign in")),
        // Option: the factory gets a `Value<String>` for the current value.
        show_some(user, |name| text(move || format!("Hello, {}", name.get()))),
        // `<For>`: rows keep their state by key.
        column(for_each(names, |name| name.clone(), |name| text(move || name.get()))),
        // `<Index>`: rows keep their state by position.
        column(for_index(names, |i, name| text(move || format!("{i}: {}", name.get())))),
        button("Toggle").on_press(move || logged_in.update(|v| *v = !*v)),
        button("Sign in as Ann").on_press(move || user.set(Some("Ann".to_owned()))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Branches").run(app)
}
```

`show(condition, factory)` は、条件が true のときに分岐を組み立て、false になると分岐を状態ごと破棄します。`condition` は、シグナル、memo、`bool` を返すクロージャーのどれかです。ファクトリーは view を返し、分岐が開くたびに実行されます。key と行の状態の詳しい説明は[一覧と表](lists-and-tables.md)にあります。

## `dynamic` と match の腕

Rust では、`match` のすべての腕が同じ型でなければなりません。要素の関数（`text`、`column`、`node`）はすべて `Node` を返すので、それだけで作った腕は一致します。しかし `button(..)` のようなコンポーネントは独自の型を返すので、両者を混ぜると型のエラーになります（`match arms have incompatible types`）。`dynamic` の中では、各腕の最後に `.into_view()` を付けて型を消してください。

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
enum Mode {
    Read,
    Edit,
}

#[component]
fn editor() -> impl Element {
    let mode = signal(Mode::Read);
    column((
        button("Switch").on_press(move || {
            mode.update(|m| {
                *m = match m {
                    Mode::Read => Mode::Edit,
                    Mode::Edit => Mode::Read,
                }
            })
        }),
        // The source must be `Clone + PartialEq`. The view is rebuilt
        // when it changes to a different value.
        dynamic(mode, |mode| match mode {
            Mode::Read => text("Read only").into_view(),
            Mode::Edit => button("Save").into_view(),
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Modes").run(editor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_swaps_the_arms() {
        let screen = kivact::test::render(editor);
        screen.get_by_text("Read only");
        screen.get_by_role(Role::Button, "Switch").click();
        assert!(screen.query_by_role(Role::Button, "Save").is_some());
        assert!(screen.query_by_text("Read only").is_none());
    }
}
```

`.into_view()` が必要になるのは、1 つの `match` や `if` が違う種類の view を返す場合だけです。タプル（最大 16 個の子。それぞれが独自の型を持てます）、`Vec`、`show` と `show_some` の別々のファクトリーは、変換が要りません。分岐が 2 つなら `show(..).fallback(..)` を使ってください。3 つ以上か、種類が違う場合に `dynamic` を使います。読み込み中、エラー、完了の 3 つの状態には、状態の上に `dynamic` を書くのではなく `async_view` を使います。`dynamic` は元の値が変わると view を作り直します。テキストを 1 つ更新するためだけに使ってはいけません。その場合はクロージャーを使ってください。

## Context

`provide(value, || child)` は、`value` を子とその下のすべてから使えるようにします。`use_context::<T>()` は、いちばん近い `T` を返します。`T` は `Clone` でなければならないので、`Copy` なハンドルをまとめた構造体か、大きいものなら `Rc` を provide してください。

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy)]
struct Session {
    user: Signal<String>,
}

#[component]
fn greeting() -> impl Element {
    let session = use_context::<Session>(); // panics, naming the type, if nothing provided it
    text(move || format!("Signed in as {}", session.user.get()))
}

#[component]
fn settings() -> impl Element {
    let session = use_context::<Session>();
    button("Rename").on_press(move || session.user.set("ann2".to_owned()))
}

fn app() -> impl IntoView {
    let session = Session { user: signal("ann".to_owned()) };
    provide(session, || column((greeting(), settings())))
}

fn main() -> kivact::Result<()> {
    App::new("Context").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children_share_the_provided_state() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Signed in as ann");
        screen.get_by_role(Role::Button, "Rename").click();
        screen.get_by_text("Signed in as ann2");
    }
}
```

- `use_context` は、provide された値そのものを返します。Solid の `useContext` と同じです。値は変わりません。構造体の中にシグナルを置いてください（`Session` がそうしているように）。変わる状態はそこに持たせます。
- 既定値はありません。何も provide されていないと、`use_context::<T>()` は型の名前を示して panic します。値が無いことがありうるなら、`Option<T>` を返す `kivact::state::try_use_context::<T>()` を使ってください。
- `provide` は、その子にだけ作用します。`provide` の後に作られた兄弟には、その値は見えません。
- テーマ、ロケール、文言、読み取りの向きには、それぞれ専用の `provide_*` 関数があります（`provide_theme`、`kivact::i18n::provide_locale`、`kivact::i18n::provide_messages`、`kivact::view::provide_direction`。[テーマ](theming.md)、[翻訳とロケール](i18n.md)）。

## ref とアプリのオブジェクト

- `var(value)` は、変わっても何にも通知しない可変の箱です。React の `useRef(value).current` に相当します。`Copy` でない、アプリのオブジェクト（履歴、パーサー、接続など）を、多くのクロージャーで共有できる `Copy` のハンドルに入れるためにも使います。
- `node_ref()` は、`.node_ref(r)` で付けるハンドルを作ります。マウントの後、`r.focus()` で要素にフォーカスし、`r.rect()` で箱を読めます。DOM のオブジェクトではなく、直接書き換えることはできません。要素を持つのは Kivact です。

```rust,no_run
use kivact::prelude::*;

/// An object of the app that is not `Copy`.
struct History {
    entries: Vec<String>,
}

#[component]
fn search() -> impl Element {
    let query = signal(String::new());
    let input = node_ref();
    let history = var(History { entries: Vec::new() }); // changing it redraws nothing

    column((
        text_input(query).label("Search").node_ref(input),
        button("Search").on_press(move || {
            history.update(|h| h.entries.push(query.get()));
            input.focus();
        }),
        button("Print history").on_press(move || {
            history.with(|h| println!("{} searches", h.entries.len()));
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Refs").run(search)
}
```

## 別のスレッドからの値

自分で作ったワーカースレッド（音声エンジン、ファイル監視、ネットワーク・クライアントなど）は、`Signal` に直接触れません。シグナルは UI スレッドにあるからです。`remote(initial)` は、シグナルと、どのスレッドにも渡せる sender を返します。

```rust,no_run
use std::time::Duration;

use kivact::prelude::*;
use kivact::task::remote;

#[component]
fn ticks_from_a_thread() -> impl Element {
    // `ticks` is an ordinary signal; `sender` is `Send` and goes to the thread.
    let (ticks, sender) = remote(0u64);
    std::thread::spawn(move || {
        for n in 1.. {
            std::thread::sleep(Duration::from_millis(500));
            sender.set(n);
        }
    });
    text(move || format!("ticks: {}", ticks.get()))
}

fn main() -> kivact::Result<()> {
    App::new("Remote").run(ticks_from_a_thread)
}
```

書き込みはまとめられます。UI は、次のターンの始めに最新の値を一度だけ受け取ります。スレッドが何回書いても同じです。コンポーネントが取り除かれると sender は閉じられ、その後の値は捨てられます。スレッドが「何かが変わった」と伝えるだけでよいなら、`remote_trigger()` が UI 側の `Trigger` と、`notify()` がブロックしない `Send` な sender を返します。[コンポーネントから非同期処理を実行する](async-and-tasks.md#別のスレッドから値を受け取る)を参照してください。

## 入力中の値を保存する

`.on_commit(..)` を付けたテキスト欄は、利用者が Enter を押すか欄から離れると値を書き込みます。キーボードのショートカットは欄から離れないので、保存する前に `commit_focused_edit()` で、フォーカスされているコントロールに確定を頼んでください。

```rust,no_run
use kivact::prelude::*;
use kivact::view::commit_focused_edit;

#[component]
fn document() -> impl Element {
    let title = signal(String::from("Untitled"));
    let saved = signal(String::new());
    let save = action(move || async move {
        // Commit the field the user is still typing in, then read the state.
        commit_focused_edit().await;
        saved.set(title.get());
        Ok(())
    });
    column((
        text_input(title)
            .label("Title")
            .on_commit(move |value: String| title.set(value)),
        text(move || format!("Saved: {}", saved.get())),
    ))
    .on_key(move |event: KeyEvent| {
        if event.key() == Key::Char('s') && event.modifiers().ctrl {
            save.run(());
            Handled::Yes
        } else {
            Handled::No
        }
    })
}

fn main() -> kivact::Result<()> {
    App::new("Document").run(document)
}
```

## Rust のポイント

### 1. クロージャーには `move` が必要

Kivact が保存するクロージャー（イベントのハンドラー、テキストのクロージャー）は、それを作った関数より長く生きます。`|| count.get()` と書くと、もうすぐ返る関数の `count` を*借用*するだけなので、コンパイラーは受け付けません。

```text
error[E0373]: closure may outlive the current function, but it borrows `count`,
which is owned by the current function
help: to force the closure to take ownership of `count`, use the `move` keyword
```

`move || count.get()` と書いてください。`move` は変数をクロージャーの中に移します。`move` 自体は、大きなデータをコピーするわけではありません。保存されるクロージャーは `'static` でなければならない、つまり、ローカル変数への参照を持てません。

### 2. ハンドルは `Copy`、データは必要に応じて clone

`Signal<T>`、`Value<T>`、`Var<T>`、`Callback<E>`、`Trigger` は、アプリの状態への小さな `Copy` のハンドルです。好きなだけ `move` クロージャーにコピーしてかまいません。どれも同じ値を指します。ハンドルに対して `.clone()` を書く必要はありません。

中のデータは別です。`signal.get()` は値の**clone** を返すので、`T: Clone` が必要です。`String` や `Vec` を clone せずに見るには、借用してください。たとえば `names.with(|n| n.len())` のように、必要な結果だけを返します。`with` の中から同じシグナルに書き込んではいけません。

### 3. `Clone + PartialEq` の境界

`set` は、新しい値が違うときだけ読み手に通知するので、`T: PartialEq` が必要です。`get` には `T: Clone` が必要です。`memo`、`for_each` の項目、`dynamic` の元の値も、値を比較します。自分の型では両方を derive してください。

```rust,no_run
#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}
# fn main() {}
```

`signal(value)` 自体、`with`、`update` は、どの型でも使えます。自分の型の定数を `Value<T>` の prop に渡すには、`Prop` も derive します（`#[derive(Clone, PartialEq, Prop)]`）。詳しくは[再利用できるコンポーネントを書く](custom-components.md#独自の型を-prop-にする)を参照してください。

### 4. 「値がない」は `Option<T>`

Rust には `null` がありません。ないかもしれない値は `Option<T>` です。`Some(x)` か `None` のどちらかです。`match`、`if let`、`.map(..)`、`.unwrap_or(..)` で扱い、むやみに `.unwrap()` を使ってはいけません。view には `show_some(option_signal, |v| ..)` を使います。そのファクトリーは、現在の中身の `Value<T>` を受け取ります。`select` と `radio_group` は `Option<K>` を編集します（何も選ばれていないと `None`）。`.optional()` は、それらのために `Signal<T>` を `Signal<Option<T>>` に変えます。

### 5. `String` と `&str`

`String` は文字列を所有し、`&str` はそれを借用します。テキストの引数は両方を受け取るので、`text("Hello")` も `text(some_string)` も使えます。テキストを返すクロージャーは、`&'static str` の literal か `String` を返せます。

```rust,no_run
use kivact::prelude::*;

#[component]
fn status(saved: Signal<bool>, name: Signal<String>) -> impl Element {
    column((
        text(move || if saved.get() { "Saved" } else { "Not saved" }),
        text(move || format!("Name: {}", name.get())),
        // Return an owned value out of `with`; a borrow cannot leave it.
        text(move || name.with(|n| n.to_uppercase())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Strings").run(|| status(signal(false), signal("ann".to_owned())))
}
```

クロージャーは、シグナルの中を指す `&str` を返せません。借用は `with` が終わると終わるからです。所有する `String` を返してください（`n.clone()`、`format!(..)`）。

### クロージャーの引数に型が必要なとき

Rust は、クロージャーの引数の型を、それを受け取る関数から推論します。`on_key(move |event| ..)` や、`Callback<E>` の prop では、それで動きます。`callback(..)` や `action(..)` のように、引数の型をクロージャー自身が決める場合は、メソッドを呼ぶ前に型を書いてください。たとえば `callback(move |word: String| len.set(word.len()))` です。型が必要なときは、コンパイラーが `type annotations needed` と教えてくれます。

### 6. 数値を返すクロージャーは戻り値の型を合わせる

定数の整数は、浮動小数点のプロパティでも受け付けられますが、保存されるクロージャーは、そのプロパティの型と正確に一致する値を返さなければなりません。`count` が `Signal<i32>` なら、`move || count.get()` は `f64` を返せません。クロージャーの中で変換してください。エラーに trait の名前が出たら、まずクロージャーの戻り値の型を確かめます。

```rust,no_run
use kivact::prelude::*;

#[component]
fn count_slider(count: Signal<i32>) -> impl Element {
    let value = binding(
        move || count.get() as f64,
        move |next: f64| count.set(next.round() as i32),
    );
    slider(value).label("Count").range(0.0..=100.0)
}
```

`binding(get, set)` は、2 つのクロージャーで読み書きする `Signal<f64>` を返します。普通の数値のプロパティなら、`move || count.get() as f64` と書きます。ハンドルそのものを変換しても、後で読まれる値は変換されません。

### 7. prelude の名前を変数で隠さない

Rust は、名前がいちばん近い束縛を使います。`let text = signal(String::new())` の後で `text("Hello")` と書くと、その変数を関数として呼ぼうとして E0618（「expected function」）になります。データを表す名前を使ってください。たとえば `label`、`rows`、`choices`、`name_field`、`save_action` です。`text`、`row`、`list`、`field`、`message`、`window` は prelude の関数として残しておきます。テーマ変数のモジュール `color`、`space`、`radius`、`font` も prelude の名前です。`let color = ..` は `color::ACCENT` を隠すので、`tint` のような名前にしてください。`window` は、よく使われるローカル変数の名前です（`let window = …`）。`settings_window` のような名前にしてください。行のコールバックの引数は、`row` ではなく `item` や `todo` のように、データに合わせた名前にしてください。`Action` のように import した型も同じ規則です。別の項目に再利用しないでください。名前を残す必要があるなら、`kivact::view::text("Hello")`、`kivact::ui::field("Name", input)`、`kivact::task::Action` のように修飾してください。

```rust,no_run
use kivact::prelude::*;

#[component]
fn heading_row(label: Value<String>) -> impl Element {
    row((text(label), button("Open")))
}
```

## 関連項目

- [チュートリアル](../getting-started/README.md): アプリを順に作る。
- [一覧と表](lists-and-tables.md): key、行の状態、大きなコレクション。
- [フォーム](forms.md): 下書きと検証のパターン。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): `resource`、`action`、`remote`。
- [再利用できるコンポーネントを書く](custom-components.md): 他の人が使うコンポーネントを書く。
- [旧 API から新しい API への移行](migrating-to-rework.md): 旧い名前とその置き換え。
- リファレンス: `cargo doc -p kivact --open`。
