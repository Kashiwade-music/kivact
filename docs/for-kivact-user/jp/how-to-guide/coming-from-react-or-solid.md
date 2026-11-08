# React や SolidJS の知識を Kivact に対応づける

[English](../../en/how-to-guide/coming-from-react-or-solid.md)

React や SolidJS を知っているなら、Kivact の概念のほとんどに対応するものがあります。このガイドでは、それらを 1 対 1 で対応づけ、大きな違いがただ 1 つであること(コンポーネントは、状態が変わるたびに実行される React のものではなく、Solid のように一度だけ実行されます)を説明し、Rust を書いたことがない開発者がつまずきやすい 7 つの細部を扱います。

[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

## 唯一の考え方: セットアップは一度だけ実行される

Kivact のコンポーネントはただの関数です。親が画面を組み立てるときに**一度だけ**実行されます。状態を作り、部品をつなぎ、view を返します。同じ mount の間、シグナルが変わっても関数全体は再実行されません。`show` の分岐などを取り外して作り直すと、その新しい mount では setup も実行されます。シグナルが変わると、それを読んでいる画面の部分だけが更新されます。

```tsx
// React: クリックのたびに関数全体が再実行される。
function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>count: {count}</button>;
}
```

```rust,test_harness
use kivact::prelude::*;

// Kivact: 関数は一度だけ実行される。再実行されるのは `text` のクロージャだけ。
fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("回数: {}", count.get())),
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
        screen.get_by_text("回数: 0");
        screen.get_by_role(Role::Button, "+1").click();
        screen.get_by_text("回数: 1");
    }
}
```

これは SolidJS のモデルです。結果として次の 2 つが言えます。

- **値が変わる場所にはクロージャを渡します。** `text(format!("回数: {}", n))` は、組み立て中に `n` を一度だけ読み、そのテキストを表示し続けます。`text(move || ...)` は、`n` が変わると Kivact が再実行するクロージャの中で読みます。`.disabled(..)`、`.style(..)`、`.name(..)` など、変化しうるすべてのプロパティも同じで、クロージャ、シグナル、memo を渡します。
- **依存配列はありません。** クロージャ、memo、effect の中で `.get()` や `.with()` で読んだものは、自動的に追跡されます。

## 概念の対応表

| 概念 | React | SolidJS | Kivact |
| --- | --- | --- | --- |
| コンポーネント | JSX を返す関数 | JSX を返す関数 | 一つのルートは `impl Component`、複数のルートは `impl IntoView` を返す関数 |
| props | props オブジェクト | props オブジェクト | 関数の引数。文字には `impl IntoText`、変化しうる値には `impl IntoValue<T>` |
| 子 | `children` | `props.children` | `V: IntoView` 型の引数 |
| コールバックの prop | `onSave` prop | `onSave` prop | 引数なしは `impl Into<Callback<()>>`。payload 付きは `Fn(T) -> R` と `HandlerResult`。`callback(..)` は共有できる `Copy` ハンドルを返す |
| 状態 | `useState` | `createSignal` | `signal(value)`。1 つの `Signal<T>` が `get`、`with`、`set`、`update` を持つ |
| 派生値 | render 内の計算、`useMemo` | `createMemo` | 1 か所の利用ならクロージャ、共有やキャッシュなら `memo(..)` |
| effect | `useEffect` | `createEffect` | `effect(..)`。最初の実行を飛ばすなら `effect_on(source, f)` |
| クリーンアップ | `useEffect` の戻り値 | `onCleanup` | `on_cleanup(..)` |
| マウント後に一度だけ実行 | `useEffect(.., [])` | `onMount` | `on_mount(..)` |
| 再描画を起こさない可変の値 | `useRef` | 通常の変数 | `var(value)` |
| 要素の参照 | DOM ノードへの `useRef` | `ref` | `node_ref()` と `.node_ref(r)`。`r.focus()` |
| context | `createContext`、`useContext` | `createContext`、`useContext` | `provide(value, \|\| child)` と `use_context::<T>()` |
| 条件 | `cond && <A/>` | `<Show when>` | `show(cond, \|\| a)`、`show_some(option, \|v\| a)` |
| 複数の分岐 | 三項演算子や `switch` | `<Switch>` | `dynamic(source, \|v\| match v { .. })` |
| 一覧 | `key` 付きの `items.map(..)` | `<For>` | `for_each(items, \|item\| item.id, \|key, item\| view)` |
| 位置による一覧 | `key={index}` | `<Index>` | `for_index(items, \|i, item\| row)` |
| key | `key` prop | `For` の同一性 | `for_each` に渡す key の関数 |
| 制御された入力 | `value` と `onChange` | `value` と `onInput` | 入力に渡す `Signal<String>`(または `Binding<T>`) |
| エラー境界 | `ErrorBoundary` | `ErrorBoundary` | `error_boundary(..)` |
| データの取得 | ライブラリーや `useEffect` | `createResource` | `resource(source, fetch)`、ユーザー操作には `action(..)` |
| 状態機械 | `useReducer` と effect、timer | reducer と effect、timer | `Machine` の `transition` が `Step` を返し、runner が副作用を実行する |
| 同じ要素の、場所をまたぐ共有（Framer Motion の `layoutId`） | `layoutId`（Framer Motion） | ライブラリ | 両方のノードに `.layout_id("id")`。新しいノードは、スタイルの layout transition を使って古い箱から滑る |
| 古い内容の退場を待ってから新しい内容を入れる（`AnimatePresence mode="wait"`） | `AnimatePresence mode="wait"` | ライブラリ | `show(cond, \|\| a).fallback(\|\| b).presence(PresenceMode::Wait)`。他のモードは `Sync` と `PopLayout` |
| スクロールに従う値（`useScroll`） | `useScroll` | スクロールのリスナー | `scroll_progress(scroller, Orientation::Vertical)`。1 つの要素なら `view_progress(target, scroller, axis, ViewRange::Entry)` |
| 1 つの時計で進む一連の動き（GSAP の `timeline`） | `gsap.timeline()` | ライブラリ | `timeline()`、`track(..)`、`to(track, target, transition).at(At::AfterPrevious(ms))`、`play()` |
| リストの項目ごとの遅延（GSAP の `stagger`） | `stagger` | ライブラリ | `Stagger::new(ms).from(StaggerFrom::Center)` と `stagger.delay(index, count)` |
| CSS の `transform: perspective(..) rotateX(..)` | `style={{ transform }}` | `style={{ transform }}` | `style().transform(Transform::new().perspective(..).rotate_x(..))` |
| CSS の `filter: blur(4px)` | `style={{ filter }}` | `style={{ filter }}` | `style().filter(Filter::new().blur(4))`（`kivact::draw`） |
| CSS の `mask`、`clip-path`、`mix-blend-mode` | `style={{ mask }}` など | 同じ | `style().mask(..)`、`.clip_path(shape)`、`.mix_blend_mode(BlendMode::Multiply)` |
| CSS の `font-variation-settings`、`font-stretch` | `style={{ fontVariationSettings }}` | 同じ | `style().font_variation_settings([("wght", 650.0)])`、`.font_width(75.0)` |
| CSS の `-webkit-text-stroke`、`paint-order` | `style={{ WebkitTextStroke }}` | 同じ | `style().text_stroke(width, color)`、`.paint_order(..)` |

上の表の名前は `use kivact::prelude::*;` で使えます。`callback` と `effect_on` は状態の操作、`error_boundary` と `component` は view の組み立てに使いますが、利用のたびに個別の import を足す必要はありません。任意の context を調べる `try_use_context` は `kivact::state` から、builder の option に使う enum はそれぞれの module から import します。


`Machine` は `useReducer` に effect と timer を足したものです。状態とイベントを明示し、
`transition` で `Step` を返し、runner で副作用を実行します。
[独自のコントロール](custom-controls.md#状態機械で振る舞いを書く)を参照してください。


## props、子、コールバック

```rust,test_harness
use kivact::prelude::*;

// `impl IntoText`: リテラル、String、または文字を返すクロージャ。
// `impl IntoValue<i32>`: 数値、シグナル、memo、クロージャ。
fn stat(label: impl IntoText, value: impl IntoValue<i32>) -> impl IntoView {
    let value = value.into_value();
    row((text(label), text(move || value.get().to_string()))).style(style().gap(8))
}

// `V: IntoView`: ほかのコンポーネントを含む、あらゆる view。
fn panel<V: IntoView>(title: &str, child: V) -> impl IntoView {
    column((text(title), child)).style(style().gap(8).padding(12))
}

// コールバックの prop: 呼び出し側が渡すクロージャ。
fn buttons(on_add: impl Into<Callback<()>>) -> impl IntoView {
    let on_add = on_add.into();
    row((button("追加").on_press(on_add), button("もう一度追加").on_press(on_add)))
}

fn app() -> impl IntoView {
    let hits = signal(0);
    panel(
        "統計",
        column((
            stat("固定値", 42),
            stat("シグナル", hits),
            stat("派生値", move || hits.get() * 2),
            buttons(move || hits.update(|h| *h += 1)),
        )),
    )
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
        screen.get_by_role(Role::Button, "もう一度追加").click();
        screen.get_by_text("1");
        screen.get_by_text("2");
    }
}
```

- 変わりうる prop は `impl IntoValue<T>` を受け取ります。呼び出し側は定数、シグナル、memo、クロージャのどれでも渡せ、あなたは `.into_value()` を一度呼んで、読み取り用の `Value<T>` ハンドルを得ます。
- 入力のないコールバックの prop は `impl Into<Callback<()>>` として受け取ります。`.into()` で一度だけ変換すると、得られるハンドルは `Copy` なので、両方のボタンで同じ操作を使えます。呼び出し側は `move || ...` のクロージャをそのまま渡せます。
- `()` を返すクロージャを受け取る場所は、`Result<(), UiError>` を返すクロージャも受け取るので、ハンドラーの中で `?` が使えます。エラーは最も近いエラーハンドラーに届きます。

## effect、クリーンアップ、マウント

```rust,no_run
use kivact::prelude::*;

fn tracker(query: Signal<String>) -> impl IntoView {
    component(move || {
        // mount 後に実行され、`query` が変わるたびに再実行される。
        effect(move || {
            println!("検索語: {}", query.get());
        });
        // このコンポーネントの node ができた後に一度だけ実行される。
        on_mount(|| println!("マウントしました"));
        // このコンポーネントが取り除かれたとき(分岐が閉じる、行が消える)に実行される。
        on_cleanup(|| println!("なくなりました"));

        text(move || format!("検索語: {}", query.get()))
    })
}

fn main() -> kivact::Result<()> {
    App::new("Effects").run(|| tracker(signal("rust".to_owned())))
}
```

- `effect` は、ログ、保存、UI の外にあるものとのやり取りといった副作用に使います。派生値を表示するために、あるシグナルを別のシグナルへ写すのには使わず、元の値をクロージャか `memo` の中で読みます。
- effect の中で呼んだ `on_cleanup` は、effect が再実行される前と、取り除かれるときに実行されます。React の effect が返す関数と同じです。
- 状態は、最も近い*構造的な境界*に属します。ルート、`show` の分岐、`for_each` の行、`provide`、`error_boundary`、`component` です。中で作ったシグナルは、その境界がなくなると解放されます。普通の関数呼び出しは境界を作りません。

## 条件、分岐、一覧

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let logged_in = signal(false);
    let user = signal(None::<String>);
    let names = signal(vec!["Ann".to_owned(), "Bob".to_owned()]);

    column((
        // `<Show when=.. fallback=..>`
        show(logged_in, || text("おかえりなさい")).fallback(|| text("サインインしてください")),
        // Option: ファクトリーは現在の値の `Value<String>` を受け取る。
        show_some(user, |name| text(move || format!("こんにちは、{}", name.get()))),
        // `<For>`: 行は key で状態を保つ。
        column(for_each(names, |name| name.clone(), |name| text(move || name.get()))),
        // `<Index>`: 行は位置で状態を保つ。
        column(for_index(names, |i, name| text(move || format!("{i}: {}", name.get())))),
        button("切り替え").on_press(move || logged_in.update(|v| *v = !*v)),
        button("Ann としてサインイン").on_press(move || user.set(Some("Ann".to_owned()))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Branches").run(app)
}
```

`show(condition, factory)` は、条件が true のときに分岐を組み立て、false になると(状態ごと)破棄します。`condition` は、シグナル、memo、`bool` を返すクロージャです。ファクトリーは view を返し、分岐が開くたびに実行されます。key と行の状態の詳細は[一覧と表](lists-and-tables.md)で扱います。

## `dynamic` と match の腕

Rust では `match` のすべての腕が同じ型でなければなりません。要素の関数(`text`、`column`、`node`)はすべて `Node` を返すので、これらだけの腕なら型が揃います。しかし `button(..)` のようなコンポーネントは独自のビルダー型を返すので、両者を混ぜると `match arms have incompatible types` という型エラーになります。`dynamic` の中では、各腕の末尾に `.into_view()` を付けて型を消します。

```rust,test_harness
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
            mode.update(|m| {
                *m = match m {
                    Mode::Read => Mode::Edit,
                    Mode::Edit => Mode::Read,
                }
            })
        }),
        // source は `Clone + PartialEq` が必要。別の値に変わると view が作り直される。
        dynamic(mode, |mode| match mode {
            Mode::Read => text("読み取り専用").into_view(),
            Mode::Edit => button("保存").into_view(),
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
        screen.get_by_text("読み取り専用");
        screen.get_by_role(Role::Button, "切り替え").click();
        assert!(screen.query_by_role(Role::Button, "保存").is_some());
        assert!(screen.query_by_text("読み取り専用").is_none());
    }
}
```

`.into_view()` が必要なのは、1 つの `match` や `if` が異なる種類の view を返すこの形のときだけです。タプル(最大 16 個で、それぞれ別の型)、`Vec`、`show` と `show_some` が別々に受け取るファクトリーには、変換は要りません。2 つの分岐なら `show(..).fallback(..)` を優先し、3 つ以上ある、または種類が違うときに `dynamic` を使います。`dynamic` は source が変わると view を作り直すので、文字を更新するだけの用途には使わず、クロージャを使ってください。

## context

`provide(value, || child)` は、子とその下のすべてで `value` を使えるようにし、`use_context::<T>()` は最も近いものを読みます。`T` は `Clone` が必要なので、`Copy` なハンドルの構造体か、重いものなら `Rc` を提供します。

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy)]
struct Session {
    user: Signal<String>,
}

fn greeting() -> impl IntoView {
    let session = use_context::<Session>(); // 提供されていなければ、型名つきで panic する
    text(move || format!("{} としてサインイン中", session.get().user.get()))
}

fn app() -> impl IntoView {
    let session = Session { user: signal("ann".to_owned()) };
    provide(session, || column((greeting(), settings())))
}

fn settings() -> impl IntoView {
    let session = use_context::<Session>();
    button("名前を変更").on_press(move || session.get().user.set("ann2".to_owned()))
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
        screen.get_by_text("ann としてサインイン中");
        screen.get_by_role(Role::Button, "名前を変更").click();
        screen.get_by_text("ann2 としてサインイン中");
    }
}
```

- 既定値はありません。何も提供されていないと、`use_context::<T>()` は型名つきで panic します。値が省略可能なら `kivact::view::try_use_context::<T>()` を使い、`Option<T>` を受け取ります。
- `provide` が影響するのは、その子だけです。後ろに作られた兄弟は値を見られません。
- テーマ、ロケール、文言、読み取りの向きには、それぞれ `kivact::view` に専用の `provide_*` 関数があります([テーマ](theming.md)、[翻訳とロケール](i18n.md))。

## ref

- `var(value)` は、変更しても何も通知しない可変の箱で、`useRef(value).current` に相当します。`Copy` でない値(パス、id 用のカウンターなど)を複数のクロージャで共有する方法でもあります。
- `node_ref()` は、`.node_ref(r)` で付けるハンドルを作ります。マウントの後は `r.focus()` で要素にフォーカスを移し、`r.rect()` でその箱を読めます。DOM オブジェクトではなく、書き換えられるものではありません。要素は Kivact が所有しています。

```rust,no_run
use kivact::prelude::*;

fn search() -> impl IntoView {
    let query = signal(String::new());
    let input = node_ref();
    let searches = var(0_u32); // 変更しても何も再描画されない

    column((
        text_input("検索", query).node_ref(input),
        button("検索する").on_press(move || {
            searches.update(|n| *n += 1);
            input.focus();
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Refs").run(search)
}
```

## Rust の 7 つのポイント

### 1. クロージャには `move` が必要

Kivact が保存するクロージャ(イベントハンドラー、テキストのクロージャ)は、それを作った関数より長く生きます。素の `|| count.get()` は、まもなく終わる関数から `count` を*借りる*だけなので、コンパイラが拒否します。

```text
error[E0373]: closure may outlive the current function, but it borrows `count`,
which is owned by the current function
help: to force the closure to take ownership of `count`, use the `move` keyword
```

`move || count.get()` と書きます。`move` は変数をクロージャに取り込みます。大きなデータが勝手にコピーされることはありません。保存されるクロージャは `'static` でなければならず、ローカル変数への参照を持てません。

### 2. ハンドルは `Copy`、データは必要に応じて clone

`Signal<T>`、`Value<T>`、`Binding<T>`、`Var<T>`、`Callback<E>` は、アプリの状態への小さな `Copy` ハンドルです。`move` のクロージャへ好きなだけコピーでき、すべて同じ値を指します。ハンドルに `.clone()` を書くことはありません。

中身のデータは別の話です。`signal.get()` は値の**複製**を返すので `T: Clone` が必要です。`String` や `Vec` を複製せずに見るには、借用します。`names.with(|n| n.len())` のように書き、必要な結果を返します。`with` の中から同じシグナルへ書き込んではいけません。

### 3. `Clone + PartialEq` の境界

`set` は新しい値が異なるときだけ読み手に通知するので、`T: PartialEq` が必要です。`get` には `T: Clone` が必要です。`memo`、`for_each` の項目、`dynamic` の source も値を比較します。自分の型には両方を derive します。

```rust,no_run
#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}
# fn main() {}
```

`signal(value)` 自体と、`with`、`update` は、どんな型でも使えます。

### 4. 「値がない」は `Option<T>`

`null` はありません。ないかもしれない値は `Option<T>`(`Some(x)` か `None`)です。`match`、`if let`、`.map(..)`、`.unwrap_or(..)` で扱い、安易な `.unwrap()` は使いません。view には `show_some(option_signal, |v| ..)` を使い、そのファクトリーは現在の中身の `Value<T>` を受け取ります。`select` と `radio_group` は `Option<K>` を編集します(何も選ばれていない状態が `None` です)。`.optional()` は、素の値を持つバインディングを、それに合わせます。

### 5. `String` と `&str`

`String` は文字列を所有し、`&str` は借用します。`impl IntoText` を受け取る prop は両方を受け付けるので、`text("こんにちは")` も `text(some_string)` も使えます。文字を返すクロージャは、`&'static str` のリテラルでも `String` でも返せます。

```rust,no_run
use kivact::prelude::*;

fn status(saved: Signal<bool>, name: Signal<String>) -> impl IntoView {
    column((
        text(move || if saved.get() { "保存済み" } else { "未保存" }),
        text(move || format!("名前: {}", name.get())),
        // `with` から所有する値を返す。借用は `with` の外へ出せない。
        text(move || name.with(|n| n.to_uppercase())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Strings").run(|| status(signal(false), signal("ann".to_owned())))
}
```

クロージャは、シグナルの中を指す `&str` を返せません。借用が `with` で終わるからです。所有する `String`(`n.clone()`、`format!(..)`)を返します。

### クロージャの引数に型が必要なとき

Rust は、クロージャの引数の型を、それを受け取る関数から推論します。`on_key(move |event| ..)` ではそれがうまくいきます。引数の型をクロージャ自身が決める `callback(..)` や `action(..)` では、その引数のメソッドを呼ぶときに型を書きます。`callback(move |word: String| len.set(word.len()))` のようにします。必要なのに書かないと、コンパイラが `type annotations needed` と言います。

### 6. 数値を返すクロージャーは戻り値の型を合わせる

整数の定数は小数のプロパティにも渡せますが、保持されるクロージャーはそのプロパティの型をそのまま返す必要があります。Rust は、クロージャーが返す整数を自動で小数へ変換しません。`count` が `Signal<i32>` なら、`move || count.get()` は `IntoValue<f64>` になりません。クロージャーの中で型を変換します。エラーの trait 名だけを見るより、まずクロージャーの戻り値を確認すると原因を絞れます。

```rust,no_run
use kivact::prelude::*;

fn count_slider(count: Signal<i32>) -> impl IntoView {
    let value = binding(
        move || count.get() as f64,
        move |next: f64| count.set(next.round() as i32),
    );
    slider("Count", value).range(0.0..=100.0)
}
```

通常の数値のプロパティには `move || count.get() as f64` と書きます。ハンドル自体を変換しても、後で読む値は変換されません。

### 7. prelude の名前を変数で隠さない

Rust は名前に最も近い束縛を使います。`let text = signal(String::new())` の後では、`text("Hello")` はそのシグナルを呼び出そうとし、E0618（expected function）になります。変数には `label`、`rows`、`choices`、`name_field`、`save_action` のようにデータを表す名前を付け、`text`、`row`、`list`、`field`、`message` は prelude の関数として使えるように残します。行の callback の引数も `row` ではなく `item`、`todo` など、データの意味で名付けます。import した `Action` のような型も同じで、別の item に同じ名前を使わないようにします。名前を残す必要があるときは、`kivact::view::text("Hello")`、`kivact::ui::field("Name", input)`、`kivact::task::Action` のように path を付けて書きます。

```rust,no_run
use kivact::prelude::*;

fn heading_row(label: String) -> impl Component {
    row((text(label), button("Open")))
}
```

## 関連項目

- [チュートリアル](../getting-started/README.md): アプリを順に作る。
- [一覧と表](lists-and-tables.md): key、行の状態、大きなコレクション。
- [フォーム](forms.md): 下書きと検証のパターン。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): `resource` と `action`。
- [再利用できるコンポーネントを書く](custom-components.md): 他の人が使うコンポーネントを書く。
- リファレンス: `cargo doc -p kivact --open`。
