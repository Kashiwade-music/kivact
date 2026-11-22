# 再利用できるコンポーネントを書く

[English](../../en/how-to-guide/custom-components.md)

繰り返し使う UI のかたまりを、ほかのコードから再利用できるコンポーネントにするためのガイドです。チップ、カード、アクションつきの通知、検索ボックスなどが対象です。props、子、コールバックの受け取り方、呼び出し側が変更できる既定の見た目の持たせ方、独自の色をテーマに載せる方法、別のコントロールを包むときにアクセシビリティを保つ方法を扱います。チュートリアルを読んでいることを前提にしています。

コンポーネントは、`#[component]` を付けた関数として書きます。これが唯一の書き方です。アプリ、`kivact::ui` の標準コンポーネント、公式の kit はすべてこの形を使っています。Kivact が署名から props のビルダーを書くので、ビルダーを手で書く必要はありません。

| 戻り値の型 | 使う場面 |
|---|---|
| `#[component] fn card(..) -> impl Element` | 通常の場合です。結果のルートが 1 つあり、呼び出し側が `.style(..)`、`.name(..)`、`.test_id(..)` とハンドラーを付けられます。 |
| `#[component] fn section(..) -> impl IntoView` | 結果にルートが 1 つに定まらない場合です。`show(..)` の分岐やタプルなど。呼び出し側は修飾子を付けられません。 |

## まずはコンポーネントで書く

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq, Prop)]
pub enum ChipTone {
    Normal,
    Accent,
}

/// A reusable label with a reactive tone and an action.
#[component]
pub fn chip(
    label: Value<String>,
    on_press: Callback<()>,
    #[prop(default = ChipTone::Normal)] tone: Value<ChipTone>,
) -> impl Element {
    button(label).on_press(on_press).style(move || {
        style()
            .border_radius(12)
            .padding_x(10)
            .padding_y(4)
            .background(if tone.get() == ChipTone::Accent {
                color::ACCENT
            } else {
                color::SURFACE_ALT
            })
    })
}

fn app() -> impl IntoView {
    let highlighted = signal(false);
    chip("Kivact", move || highlighted.update(|value| *value = !*value))
        .tone(move || if highlighted.get() { ChipTone::Accent } else { ChipTone::Normal })
        .style(style().width(120))
}

fn main() -> kivact::Result<()> {
    App::new("Chips").run(app)
}
```

署名が API になる仕組みは次のとおりです。

- **必須の prop は位置引数になります。** 署名の順に並びます。`chip("Kivact", ..)` は、ラベルと押したときのハンドラーを位置で受け取ります。
- **`Option<..>` の prop と `#[prop(default = expr)]` の prop は setter になります。** 上の chip は、`chip("Kivact", ..)` と書けば tone を省略でき、`chip("Kivact", ..).tone(ChipTone::Accent)` と書けば指定できます。省略可能なコールバックは `on_close: Option<Callback<()>>` と書きます。呼び出し側は `.on_close(..)` と書き、本体では `Option<Callback<()>>` を受け取ります。
- **本体は一度だけ、自分の setup の owner の中で実行されます。** 本体が作るシグナル、effect、コールバック、クリーンアップは、このインスタンスに属し、view が取り除かれるときに破棄されます。コンポーネントは layout の箱を追加しません。本体のルートが、そのままコンポーネントのルートになります。
- **呼び出し側はルートを変えられます。** `chip` は `impl Element` を返すので、呼び出し側の `.style(..)` はボタンに適用されます。呼び出し側のスタイルは本体のスタイルの後に適用され、同じプロパティを両方が設定していたら呼び出し側が勝ちます。呼び出し側のハンドラーは本体のハンドラーより先に動きます。呼び出し側の `name`、`description`、`disabled`、`test_id`、`node_ref`、`variant` は、本体のものを置き換えます。
- **コールバックはクロージャーを受け取ります。** `Callback<()>` の prop は、クロージャー（`move || ..`）か、呼び出し側がすでに持っている `Callback<()>` を受け取ります。入力を持つ `Callback<E>` の prop は、クロージャー（`move |event| ..`）を受け取ります。引数の型は `E` から Rust が推論します。親が持つ `on_pick: Callback<i32>` を渡すときは、`picker(move |n| on_pick.call(n))` と書きます。

`Value<T>` は、定数、シグナル、`Value`、クロージャーのどれも受け取るので、1 つの prop を静的にも反応的にも使えます。`Value<String>` の prop は `&str` と `String` も受け取ります。値の prop の型は `Clone + PartialEq` でなければなりません。値が変わらなければ更新を省けるのは、このためです。

マクロは、prop の種類を書かれた名前で判断します。`Value<T>`、`Signal<T>`、`Callback<E>`、`View`、`ViewFn`、`Vec<View>` です。これらの型の別名は、ただの prop になります。名前そのものを書いてください。

## 独自の型を prop にする

prop に独自の型を使うなら、その型に `Prop` を derive すると、定数を直接渡せます。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq, Prop)]
pub enum Tone {
    Info,
    Warning,
}

/// A label with a tone that follows a signal.
#[component]
pub fn badge(label: Value<String>, tone: Value<Tone>) -> impl Element {
    node()
        .name(label)
        .style(move || match tone.get() {
            Tone::Info => style().background(color::ACCENT).color(color::ACCENT_TEXT),
            Tone::Warning => style().background(color::WARNING).color(color::TEXT),
        })
        .child(text(label))
}

fn app() -> impl IntoView {
    let online = signal(true);
    row((
        badge("Saved", Tone::Info),
        badge("Offline", move || if online.get() { Tone::Info } else { Tone::Warning }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Badges").run(app)
}
```

`Prop` が無いと、`badge("Saved", Tone::Info)` はコンパイルできません。`Tone` のシグナル、`Value`、クロージャーは、どちらの場合でも使えます。

## 子要素とスロットを受け取る

内容を包むコンポーネントは、子を `View` として受け取ります。複数の子はタプルで渡します。タプルの view は view だからです。`()` は「ここには何もない」を表すので、空のスロットとしても使えます。名前付きのスロットを持つコンポーネントは、スロットごとに引数を 1 つ持ちます。`Option<View>` のスロットは省略できます。

```rust,no_run
use kivact::prelude::*;

/// A titled box. `body` is whatever the caller wants inside, `footer` may be `()`.
#[component]
pub fn card(title: Value<String>, body: View, footer: View) -> impl Element {
    column((
        text(title).style(style().padding_bottom(4)),
        body,
        footer,
    ))
    .style(
        style()
            .padding(space::MD)
            .gap(8)
            .border(1, color::BORDER)
            .border_radius(radius::MD),
    )
}

fn app() -> impl IntoView {
    column((
        card("Profile", (text("Ann"), text("Admin")), ()),
        card("Danger zone", text("Deleting cannot be undone"), button("Delete")),
    ))
    .style(style().gap(12).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Cards").run(app)
}
```

`View` のスロットは、コンポーネントが使われたときに一度だけ消費されます。コンポーネントが内容を何度もマウントするなら、たとえばセクションが開くたびに、`ViewFn` を受け取ってください。呼び出し側は関数を渡し、内容は開くたびに作り直されます。

```rust,no_run
use kivact::prelude::*;

/// A section that builds its content when it opens and drops it when it closes.
#[component]
pub fn collapsible(title: Value<String>, open: Signal<bool>, content: ViewFn) -> impl Element {
    column((
        button(title).on_press(move || open.update(|open| *open = !*open)),
        show(open, content),
    ))
}

fn app() -> impl IntoView {
    let open = signal(false);
    collapsible("Details", open, || text("Created by Ann"))
}

fn main() -> kivact::Result<()> {
    App::new("Collapsible").run(app)
}
```

`Signal<bool>` の prop は、コンポーネントが読みも書きもする状態です。呼び出し側は自分のシグナルを渡します。構造体の 1 つの field なら `bind!(form.details_open)` と書きます。

それぞれの子に独自のラッパーが要る場合（分割ビューの各ペインなど）は、`Vec<View>` を受け取ります。呼び出し側は、最大 16 個の view のタプルか、同じ種類の view の `Vec` を渡します。本体は、子を 1 つずつ `View` として受け取り、包めます。

## コールバックを受け取る

```rust,no_run
use kivact::prelude::*;

/// A chip with a remove button. Delete removes it too.
#[component]
pub fn removable_chip(label: Value<String>, on_remove: Callback<()>) -> impl Element {
    // `on_remove` is a Copy handle: the button and the key handler both use it.
    row((text(label), button("Remove").on_press(on_remove)))
        .name(label)
        .focusable(true)
        .on_key(move |event: KeyEvent| {
            if event.key() == Key::Delete {
                on_remove.call(());
                Handled::Yes
            } else {
                Handled::No
            }
        })
}

fn app() -> impl IntoView {
    let removed = signal(0);
    column((
        removable_chip("Rust", move || removed.update(|n| *n += 1)),
        text(move || format!("removed: {}", removed.get())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Remove").run(app)
}
```

本体は `Copy` の `Callback` を受け取ります。それを任意の `on_*` に渡すか、`.call(argument)` で呼べます。入力のないコールバックでは、引数は `()` です。

押したときのハンドラーはイベントを受け取りません。修飾キーが押されているかを見るには、React の `event.getModifierState("Control")` に当たる `modifiers()` を読みます。

```rust,no_run
use kivact::prelude::*;
use kivact::view::modifiers;

/// Opens its panel; Ctrl+click (Cmd+click) opens only this one.
#[component]
pub fn open_button(open: Callback<bool>) -> impl Element {
    button("Open").on_press(move || {
        let held = modifiers().get();
        open.call(held.ctrl || held.meta)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Open").run(|| open_button(|only| println!("only this panel: {only}")))
}
```

`Callback<E>` の prop に渡すクロージャーの引数の型は `E` から決まるので、`|only| ..` には型を書く必要がありません。呼び出し側のクロージャーが失敗しうるなら、`()` の代わりに `Result<(), UiError>` を返せます。失敗は、いちばん近い `error_boundary` に報告されます（[エラーや更新の理由を調べる](diagnostics.md)を参照）。

## 既定の見た目を渡す

コンポーネントは既定のスタイルを持ち、呼び出し側はそれを変えられます。

- **本体でルートにスタイルを付けます。** 信号や `Value` を読むスタイルのクロージャーは、再マウントせずに変化に追従します。
- **固定の色ではなく、テーマの変数を使います。** `color::ACCENT`、`space::MD`、`radius::MD` は、クロージャーなしで現在のテーマに従います。固定の色はテーマに追従できません。
- **意味を prop として公開します。** tone、サイズ、variant の prop は、何が変わりうるかを表し、コンポーネントは残りを一貫させます。呼び出し側の `.style(..)` は、残りのためのものです。本体のスタイルの後に適用されます。

## 独自の値をテーマに載せる

コンポーネントの中に固定の色を書くと、テーマに追従できません。コンポーネントライブラリーに必要な値ごとに、テーマ変数を宣言してください。テーマを組み立てる人がその値を設定できます。テーマがその変数を知らないときは、フォールバックを使います。フォールバックはコアの変数にできるので、どのテーマでもコンポーネントは調和します。

```rust,test_harness
use kivact::prelude::*;
use kivact::style::ThemeVar;
use kivact::test::{RenderOptions, render_with};

/// The colors of `status_dot`. Without a value in the theme, they follow the
/// core success and muted colors.
pub static ONLINE_VAR: ThemeVar<Color> = ThemeVar::alias("status.online", &color::SUCCESS_VAR);
pub static OFFLINE_VAR: ThemeVar<Color> = ThemeVar::alias("status.offline", &color::TEXT_MUTED_VAR);
pub const ONLINE: Color = Color::var(&ONLINE_VAR);
pub const OFFLINE: Color = Color::var(&OFFLINE_VAR);

#[component]
pub fn status_dot(online: Value<bool>) -> impl Element {
    node()
        .role(Role::Status)
        // `Value` reads are tracked, so the name follows the signal.
        .name(move || if online.get() { "online" } else { "offline" })
        .style(move || {
            style()
                .size(12, 12)
                .border_radius(6)
                .background(if online.get() { ONLINE } else { OFFLINE })
        })
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().set(&ONLINE_VAR, rgb(0x00aa00)).set(&OFFLINE_VAR, rgb(0x888888));
    App::new("Status").theme(theme).run(|| status_dot(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot_color(theme: Theme) -> u32 {
        let screen = render_with(RenderOptions::with_theme(theme), || {
            row(status_dot(true)).style(style().padding(8))
        });
        let dot = screen.get_by_role(Role::Status, "online").rect().expect("laid out");
        let shot = screen.screenshot();
        shot.pixel((dot.x + 6.0) as u32, (dot.y + 6.0) as u32).expect("inside the window")
    }

    #[test]
    fn the_theme_sets_the_color_and_a_plain_theme_falls_back() {
        let themed = Theme::light().set(&ONLINE_VAR, rgb(0x00aa00));
        assert_eq!(dot_color(themed), 0x00aa00);
        assert_ne!(dot_color(Theme::light()), 0x00aa00);
    }
}
```

- テーマ変数は、名前とフォールバックを持つ `static` の `ThemeVar` と、それを指す `const` の `Color`（または `Length::var` の `Length`）です。値を置きたい場所には `const` を書きます。テーマが変わると、それを使うすべてのスタイルが追従します。CSS の `var(--x)` と同じです。
- `theme.set(&VAR, value)` は、そのテーマで変数に値を与えます。値は別の変数でもかまいません。たとえば `theme.set(&ONLINE_VAR, &color::ACCENT_VAR)` です。
- 別の変数に従い、テーマが設定するまでその値を使う変数は、`ThemeVar::alias(name, &OTHER_VAR)` で宣言します。上の 2 つの変数はそうしています。すべての種類の変数について、別名を書く方法はこれだけです。
- 名前は、診断と DevTools で変数を指します。ライブラリーの名前を接頭辞にしてください。
- `ThemeExtension` は、値ではない表のためのものです。たとえば標準コンポーネントの recipe の表です（[テーマ](theming.md)）。

アプリの実行中にテーマを切り替えるには、ルートを `provide_theme` で包み、クロージャーかシグナルを渡します（[アプリの実行中にテーマを切り替える](theming.md#アプリの実行中にテーマを切り替える)）。

## アイコンはテーマ経由で描く

コンポーネントの中に画像を埋め込まないでください。アイコンの名前を指定し、その名前がどう見えるかはテーマに決めさせます。こうすると、kit はコンポーネント専用のグリフを与えられ、アプリはそのどれも差し替えられます。普通のアイコンボタンには標準の `icon_button(icon, label)` を使います。別の形が必要なときだけ、自分で書いてください。

```rust,test_harness
use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::style::Icon;
use kivact::test::{RenderOptions, render_with};

/// A square tool button that shows the icon named `name`.
#[component]
pub fn tool_button(name: Value<Icon>, label: Value<String>) -> impl Element {
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .style(style().size(32, 32))
        .child(icon(name))
}

const SEARCH: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'>\
<circle cx='10' cy='10' r='6' fill='none' stroke='black' stroke-width='2'/>\
<path d='M15 15l6 6' stroke='black' stroke-width='2'/></svg>";

fn main() -> kivact::Result<()> {
    // The app (or a kit's theme) supplies the image for the name.
    let theme = Theme::light().with_icon("search", SEARCH);
    App::new("Icons").theme(theme).run(|| tool_button("search", "Search"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_theme_resolves_the_name() {
        let theme = Theme::light().with_icon("search", SEARCH);
        let screen = render_with(RenderOptions::with_theme(theme), || tool_button("search", "Search"));
        screen.get_by_role(Role::Button, "Search");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn an_unknown_name_is_reported_in_debug_builds() {
        let screen = render_with(RenderOptions::default(), || tool_button("nope", "Nope"));
        screen.run_until_idle();
        if cfg!(debug_assertions) {
            let errors = screen.errors();
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].kind(), &ErrorKind::Asset);
        }
    }
}
```

`Icon::named("search")` と、そのまま書いた `"search"` は同じものです。そのため、`Value<Icon>` の prop は、名前、`Asset`（そのまま使う画像）、どちらかのシグナルを受け取ります。標準のセットには、標準コンポーネントが描く `check` や `chevron-right` などのマークがあります。自分のものは `Theme::with_icon(name, image)` で足します。知らない名前は何も描かず、デバッグビルドでは一度だけ報告されます。

## アクセシブルな名前を本物のコントロールに付ける

複合コンポーネント（アイコンと文字入力を組み合わせた検索ボックスなど）のルートは、利用者が入力するコントロールとは別のものです。名前と無効の状態は、そのコントロールに属します。ラッパーに置いたままだと、スクリーンリーダーは名前の無いテキストボックスを読み上げてしまいます。`field(label, control)` はラベルを表示し、そのラベルを名前としてコントロールに渡します。

```rust,test_harness
use kivact::prelude::*;

/// A search box: an icon and a labelled text field that carries the caller's state.
#[component]
pub fn site_search(
    value: Signal<String>,
    #[prop(default = "Site search")] label: Value<String>,
    #[prop(default = false)] disabled: Value<bool>,
) -> impl Element {
    row((
        icon("search"),
        field(label, text_input(value).disabled(disabled)).style(style().flex_grow(1)),
    ))
    .style(style().gap(6).align_items(Align::Center))
}

fn app() -> impl IntoView {
    let query = signal(String::new());
    site_search(query)
}

fn main() -> kivact::Result<()> {
    App::new("Search").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_box_carries_the_name() {
        let screen = kivact::test::render(app);
        let input = screen.get_by_role(Role::TextBox, "Site search");
        input.type_text("kivact");
        assert_eq!(input.value().as_deref(), Some("kivact"));
    }

    #[test]
    fn pressing_the_label_focuses_the_text_box() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Site search").click();
        assert!(screen.get_by_role(Role::TextBox, "Site search").is_focused());
    }
}
```

`field` は、どんな要素も受け取ります。`impl Element` を返す自分のコンポーネントも含みます。フォーカスを受けられるのに名前の無いコントロールは、マウントされたときに報告されます。デバッグビルドと `kivact::test` では、その報告はエラーになります。

フォーカスを転送する必要はありません。フィールドのラベルを押すと、中のコントロールが動作します。2 つ目の試験がそれを示しています。

## 呼び出し側が設定した内容を読む

ライブラリーのコンポーネントは、呼び出し側が呼び出しの後に書いた内容に依存することがあります。たとえば、タグは `.on_press(..)` を与えられたときだけ押せそうに見えるべきです。`#[caller]` を付けた引数を足してください。Kivact は、呼び出し側がこのコンポーネントに設定した修飾子（variant、無効の状態、説明、押したときのハンドラーがあるか）のスナップショットを渡します。呼び出し側がそれを渡す必要はありません。

```rust,no_run
use kivact::custom::Caller;
use kivact::prelude::*;

/// A tag that looks pressable only when the caller gave it `on_press`.
#[component]
pub fn tag(label: Value<String>, #[caller] caller: Caller) -> impl Element {
    let pressable = caller.has_press();
    text(label).style(
        style()
            .padding_x(8)
            .border_radius(radius::SM)
            .background(color::SURFACE_ALT)
            .cursor(if pressable { Cursor::Pointer } else { Cursor::Default }),
    )
}

fn app() -> impl IntoView {
    row((tag("Draft"), tag("Open").on_press(|| println!("open")))).style(style().gap(8))
}

fn main() -> kivact::Result<()> {
    App::new("Tags").run(app)
}
```

スナップショットは、コンポーネントがマウントされたときに取られます。上の例のように本体の中で読んでください。アプリがこれを必要とすることは、ほとんどありません。

## コンポーネントの誤用を報告する

コンポーネントが処理できない引数を受け取ったら、開発者が原因を見られる場所で失敗させてください。`kivact::custom::raise` は、場所付きのメッセージで panic します。Kivact 自身の誤用と同じ扱いです。アプリは終了し、試験は失敗します。

```rust,no_run
use kivact::ErrorKind;
use kivact::custom::raise;
use kivact::prelude::*;

#[component]
pub fn progress_ring(done: u32, total: u32) -> impl Element {
    if total == 0 {
        raise(
            ErrorKind::Message,
            "progress_ring: `total` must be greater than zero",
        );
    }
    text(format!("{done} / {total}"))
}

fn main() -> kivact::Result<()> {
    App::new("Ring").run(|| progress_ring(3, 10))
}
```

これはプログラマーのミスにだけ使ってください。利用者が起こしうる問題、たとえば不正なファイルは、コンポーネントが表示する `Result` であり、誤用ではありません。

## `#[component]` が受け付けないもの

マクロは、props のビルダーにできない署名を報告します。報告には何を変えればよいかが書かれます。

- `impl Element` と `impl IntoView` 以外の戻り値の型。
- lifetime の引数と const の引数（型の引数は受け付けます）。
- メソッド。`#[component]` は自由関数です。
- 分解のパターン。各 prop は名前の付いた引数にします。
- `#[prop(..)]` の未知のキー。
- `async`、`const`、`unsafe`、`extern` の関数。`#[component]` の関数は普通の `fn` です。
- doc comment、`allow`、`warn`、`deny`、`deprecated` 以外の属性。`#[expect(..)]` の代わりに `#[allow(..)]` と書いてください。

## 関連項目

- [エラーや更新の理由を調べる](diagnostics.md): 発生したエラーの行き先と、それを見る方法。
- [独自の見た目のコントロールを作る](custom-controls.md): 選択、メニュー、ドラッグ、文字の編集など、標準の振る舞いを持ちつつ見た目を自分のものにするコンポーネント。
- [テーマと部品の見た目を変える](theming.md): アプリ側から見た、テーマ変数、コンポーネントのテーマ、アイコン。
- [画面を組む](layout.md): スタイルのプロパティ。
- リファレンス: `cargo doc -p kivact --open`。`kivact::view::Element`、`kivact::view::Node`、`kivact::component`、`kivact::style::{Theme, ThemeVar, IconSet}`。
