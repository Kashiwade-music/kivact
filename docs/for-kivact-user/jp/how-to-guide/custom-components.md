# 再利用できるコンポーネントを書く

[English](../../en/how-to-guide/custom-components.md)

繰り返し使う UI のかたまりを、ほかのコードから再利用できるコンポーネントにするためのガイドです。チップ、カード、アクションつきの通知、検索ボックスなどが対象です。props、子要素、コールバックの受け取り方、呼び出し側が変更できる既定の見た目の持たせ方、独自の色をテーマに載せる方法、別のコントロールを包むときにアクセシビリティを保つ方法を扱います。入門チュートリアルを読んでいることを前提にしています。

コンポーネントは、表示できるものを返すただの関数です。props を取るなら `#[component]` を付けます。Kivact が関数の署名から props のビルダーを書くので、ビルダーを手で書くことはありません。ほとんどの場合、次の 3 つの形で足ります。

| 形 | 使う場面 |
|---|---|
| `#[component] fn card(title: Value<String>, ..) -> impl IntoView` | props、子要素、コールバック（省略可能なものを含む）を取るとき。 |
| `fn header() -> impl Element` | props が無く、呼び出し側が `.style(..)`、`.test_id(..)` などの node の modifier を付けたいとき。 |
| `fn section() -> impl IntoView` | 変更できるルートが 1 つに定まらないとき。分岐、fragment、`scope(..)` など。 |

## まずはコンポーネントで書く

再利用できる最小のコンポーネントは、props を引数で受け取り `impl IntoView` を返します。`#[component]` が、構造体、そのコンストラクター、省略可能な prop ごとの setter を作ります。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ChipTone {
    Normal,
    Accent,
}
constant_value!(ChipTone);

/// A reusable label with a reactive tone and an action.
#[component]
pub fn chip(
    label: Value<String>,
    #[prop(default = ChipTone::Normal)] tone: Value<ChipTone>,
    on_press: Callback<()>,
) -> impl IntoView {
    button(label).on_press(on_press).style(move || {
        style()
            .radius(12)
            .padding_x(10)
            .padding_y(4)
            .background(if tone.get() == ChipTone::Accent {
                ColorToken::Accent
            } else {
                ColorToken::SurfaceAlt
            })
    })
}

fn app() -> impl IntoView {
    let highlighted = signal(false);
    chip("Kivact", move || highlighted.update(|value| *value = !*value)).tone(move || {
        if highlighted.get() {
            ChipTone::Accent
        } else {
            ChipTone::Normal
        }
    })
}

fn main() -> kivact::Result<()> {
    App::new("Chips").run(app)
}
```

署名が API になる規則です。

- **必須の prop は位置引数。** `chip("Kivact", ..)` はラベルと押下のハンドラーを位置で受け取ります。
- **`#[prop(default = expr)]` と `#[prop(optional)]` は setter になる。** 上のチップは、tone を省いて `chip("Kivact", ..)` とも、`chip("Kivact", ..).tone(ChipTone::Accent)` とも書けます。省略可能な prop は `Option<X>` と宣言し、その setter は `X` が受け取る入力を受け取ります。省略可能なコールバックは `#[prop(optional)] on_close: Option<Callback<()>>` と書きます。本体が受け取るのは、コンポーネントの setup で変換済みの `Option<Callback<()>>` です。
- **本体は専用の setup の owner の中で一度だけ実行される。** 生成される `impl IntoView` は本体を `scope(..)` の中に置きます。本体が作るシグナル、effect、コールバック、cleanup はそのインスタンスに属し、表示が取り除かれると破棄されます。レイアウトの箱は追加しません。本体のルートがコンポーネントのルートです。
- **公開されるのは戻り値だけ。** 生成される構造体は関数名を UpperCamelCase にした名前（`chip` → `Chip`）で、`IntoView` は実装しますが `Element` は実装しません。呼び出し側は `.style(..)` や `.test_id(..)` を書けません。呼び出し側が変えてよいものは prop にし、それ以外はコンポーネントの判断にします。

`Value<T>` は定数、シグナル、`Value`、クロージャを受け取るので、prop を静的な値にもリアクティブな値にもできます。API を二つに分ける必要はありません。`Value<String>` の prop は `&str` と `String` も受け取ります。値の prop の型は `Clone + PartialEq` でなければなりません。値が変わっていないときに更新を省くためです。

## 独自の型を prop にする

独自の型を prop にするときは、その型を定義したクレートで一度だけ登録すると、定数をそのまま渡せるようになります。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Info,
    Warning,
}

constant_value!(Tone);

/// A label with a tone that follows a signal.
#[component]
pub fn badge(label: Value<String>, tone: Value<Tone>) -> impl IntoView {
    node()
        .name(label)
        .style(move || match tone.get() {
            Tone::Info => style().background(ColorToken::Accent).color(ColorToken::AccentText),
            Tone::Warning => style().background(ColorToken::Warning).color(ColorToken::Text),
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

`constant_value!(Tone)` がないと、`badge("Saved", Tone::Info)` はコンパイルできません。`Tone` のシグナル、`Value`、クロージャーは登録なしでも渡せます。

## 普通の関数に独立した setup の寿命を持たせる

`#[component]` には上の setup の境界が既にあります。普通の Rust 関数が、インスタンスごとに属するべきシグナル、effect、cleanup を作る場合は、代わりに `scope(move || ...)` を返します。Kivact はこの初期化処理を、そのインスタンスが mount されるときに、専用の owner と mount 先の親の context の下で一度だけ呼び、インスタンスが取り除かれるとそれらを破棄します。レイアウトの箱は追加しません。

```rust,no_run
use kivact::prelude::*;

fn local_counter(label: impl IntoText) -> impl IntoView {
    let label = label.into_text_value();
    scope(move || {
        let count = signal(0);
        on_cleanup(|| println!("Counter removed"));
        column((
            text(label),
            text(move || count.get().to_string()),
            button("Increment").on_press(move || count.update(|n| *n += 1)),
        ))
    })
}
```

`count` を変えると文字が更新されますが、初期化処理は再び呼ばれません。この境界には `impl IntoView` を返します。透過的な `View` は modifier を付けるルートのノードを公開しないためです。見えるルートが必要なら内側に置きます。`scope(..)` 自体は context の値を登録しません。子へ値を渡す目的なら `provide(value, factory)` を使います。

## 子要素とスロットを受け取る

内容を包むコンポーネントは、それを `View` で受け取ります。複数の子はタプルです。ビューのタプルもビューだからです。`()` は「ここには何もない」を表すので、空のスロットとしても使えます。名前つきスロットを持つコンポーネントは、スロットごとに引数を 1 つ取ります。

```rust,no_run
use kivact::prelude::*;

/// タイトルつきの箱。`body` は呼び出し側が入れたいもの、`footer` は `()` でもかまいません。
#[component]
pub fn card(title: Value<String>, body: View, footer: View) -> impl IntoView {
    column((
        text(title).style(style().padding_bottom(4)),
        body,
        footer,
    ))
    .style(style().padding(12).gap(8).border(1, ColorToken::Border).radius(8))
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

`View` のスロットは、コンポーネントが使われるときに一度だけ使い切られます。コンポーネントが内容を何度も mount する場合、たとえばセクションを開くたびに作り直す場合は `ViewFn` を取ります。呼び出し側は関数を渡し、開くたびに内容が作り直されます。

```rust,no_run
use kivact::prelude::*;

/// 開くときに内容を作り、閉じるときに破棄するセクション。
#[component]
pub fn collapsible(
    title: Value<String>,
    open: Binding<bool>,
    content: ViewFn,
) -> impl IntoView {
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

シグナルは `Binding<bool>` に変換できるので、呼び出し側はシグナルをそのまま渡せます。`Binding` は、コンポーネントが読み書きの両方を行う状態のための型です。

分割表示の各ペインのように、子ごとに別のラッパーが必要な場合は `Vec<View>` を受け取ります。コンストラクターは `impl IntoChildren`（16 個までのビューのタプル、または同じ種類のビューの `Vec`）を受け取り、本体はラップできるよう各子を `View` として受け取ります。

## コールバックを受け取る

payload のないコールバックは `Callback<()>` で受け取ります。

```rust,no_run
use kivact::prelude::*;

/// 削除ボタンつきのチップ。Delete キーでも削除できます。
#[component]
pub fn removable_chip(label: Value<String>, on_remove: Callback<()>) -> impl IntoView {
    // `on_remove` は 2 つのハンドラーが使います。素のクロージャーは片方にしか
    // move できませんが、`Callback` は Copy のハンドルです。
    let remove = on_remove;
    row((
        text(label),
        button("Remove").on_press(move || remove.call(())),
    ))
    .name(label)
    .focusable(true)
    .on_key(move |event: KeyEvent| {
        if event.key() == Key::Delete {
            remove.call(());
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

コンストラクターは `impl Into<Callback<()>>` を受け取るので、呼び出し側はクロージャーも既存のコールバックも渡せます。クロージャーはアプリのアリーナに一度だけ保存され、得られる `Callback` は `Copy` です。いくつの `move` クロージャーからでも持てます。呼び出しは `.call(引数)` です。入力のないコールバックには、上のように引数として `()` を渡します。

押下のハンドラーはイベントを受け取りません。React の `event.getModifierState("Control")` のように修飾キーが押されているかを知るには、`modifiers()` を読みます。

```rust,no_run
use kivact::prelude::*;
use kivact::view::modifiers;

/// Opens its panel; Ctrl+click (Cmd+click) opens only this one.
#[component]
pub fn open_button(open: Callback<bool>) -> impl IntoView {
    button("Open").on_press(move || {
        let held = modifiers().get();
        open.call(held.ctrl || held.meta)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Open").run(|| open_button(|only| println!("only this panel: {only}")))
}
```

payload のあるコールバックは `Callback<E>` と宣言します。setter は `impl Fn(E) -> R` を受け取り、クロージャーの引数の型を推論するので、`|only| ..` に型注釈は要りません。呼び出し側のクロージャーが失敗するかもしれない場合、`R` は `()` の代わりに `Result<(), UiError>` でもかまいません。失敗は一番近い `error_boundary` に報告されます ([エラーや更新の原因を調べる](diagnostics.md))。

## 既定の見た目を渡す

コンポーネントは自分のルートと既定のスタイルを持ちます。`#[component]` の型には node の modifier が無いので、呼び出し側が変えるのはルートそのものではなく、props が公開する値です。

- **本体でコンポーネントのルートをスタイルする。** シグナル、`Value`、`use_theme()` を読むスタイルのクロージャーは、再 mount なしで変化に追従します。
- **見た目は prop として公開する。** tone、size、variant の prop は、呼び出し側の `.style(..)` より明確です。何を変えてよいかを示し、残りはコンポーネントが一貫して保ちます。
- **テーマの既定値は定数ではなく値。** `chip` が `ColorToken` を使うように、スタイルのクロージャーの中で `use_theme()` からトークンを読みます。色を直接書くとテーマに追従できません。

element のビルダー（`Element` を実装する型）は別です。呼び出し側が同じルートを持つので、既定の見た目は `base_style` に置きます。`base_style` は、呼び出しの順序に関係なく、呼び出し側のすべての `.style(..)` の**下**に置かれます。そういうコンポーネントの中で `.style(..)` を使うのは、呼び出し側に勝たせたいスタイルのときだけにしてください。その場面はまれです。`#[component]` では代わりに prop を使います。

### `Element` が必要なとき

呼び出し側が node の modifier を付けなければならないとき、または標準コンポーネントが `Element` を要求するとき（たとえば `field(label, control)` は `Element` を受け取ります）だけ、`Element` を実装する型を書きます。`IntoView` で表示を仕上げ、`Element` は `node_mut` からルートのノードを返して実装します。

- **`Element` はメソッド 1 つ。** `fn node_mut(&mut self) -> &mut Node` を書くと、`style`、`name`、`disabled`、`test_id`、`on_key`、`on_press`、`draw` などがその型で使えます。`node_mut` が `doc(hidden)` なのは、これが trait の実装用の口であり、日常的に呼ぶ API ではないからです。trait は seal していないので外部の型も実装できますが、アプリのコードがこのメソッドを呼ぶことはありません。
- **ルートの設定には `with_node` を使う。** `role`、`checked`、`expanded`、`selected`、`value_text`、`base_style` は `Node` の固有メソッドで、ビルダーからは `.with_node(|root| root.role(Role::Alert))` で届きます。
- **状態は setup の owner に属する。** インスタンスのリソースは、コンストラクターではなく `into_view` の中の `scope(..)` で作ります。コンストラクターは呼び出し側の setup で実行されるので、リソースが呼び出し側と同じ寿命になってしまいます。

## 独自のデザイントークンをテーマに載せる

コンポーネントに色を直接書くと、テーマに追従できません。コンポーネントの色、寸法、フォントは、`ThemeExtension` を実装した型としてテーマに載せましょう。テーマを作る人が値を設定できます。その型を知らないテーマには、標準のパレットから導いた fallback が渡されるので、どのテーマの下でもコンポーネントは描画できます。

```rust,test_harness
use kivact::prelude::*;
use kivact::style::ThemeExtension;
use kivact::test::{RenderOptions, render_with};

/// `status_dot` の色。コンポーネントライブラリごとに 1 つの型を用意します。
#[derive(Debug, PartialEq)]
pub struct StatusColors {
    pub online: Color,
    pub offline: Color,
}

impl ThemeExtension for StatusColors {
    /// `StatusColors` を持たないテーマが受け取る値。パレットから導きます。
    fn fallback(theme: &Theme) -> Self {
        StatusColors {
            online: theme.colors().success,
            offline: theme.colors().text_muted,
        }
    }
}

pub fn status_dot(online: impl IntoValue<bool>) -> impl Element {
    let online = online.into_value();
    node()
        .role(Role::Status)
        // `Value` の読み取りは追跡されるので、名前もシグナルに追従します。
        .name(move || if online.get() { "online" } else { "offline" })
        .style(move || {
            // スタイルのクロージャーの中でテーマを読むと、テーマの変更に追従します。
            let colors = use_theme().get().ext::<StatusColors>();
            style()
                .size(12, 12)
                .radius(6)
                .background(if online.get() { colors.online } else { colors.offline })
        })
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().with(StatusColors {
        online: rgb(0x00aa00),
        offline: rgb(0x888888),
    });
    App::new("Status").theme(theme).run(|| status_dot(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot_color(theme: Theme) -> u32 {
        let options = RenderOptions::default().theme(theme);
        let screen = render_with(options, || row(status_dot(true)).style(style().padding(8)));
        let dot = screen.get_by_role(Role::Status, "online").rect().expect("laid out");
        let shot = screen.screenshot();
        shot.pixel((dot.x + 6.0) as u32, (dot.y + 6.0) as u32).expect("inside the window")
    }

    #[test]
    fn the_theme_sets_the_color_and_a_plain_theme_falls_back() {
        let themed = Theme::light().with(StatusColors {
            online: rgb(0x00aa00),
            offline: rgb(0x888888),
        });
        assert_eq!(dot_color(themed), 0x00aa00);
        assert_ne!(dot_color(Theme::light()), 0x00aa00);
    }
}
```

`Theme::with(値)` で拡張を載せ、`theme.ext::<T>()` で読みます (共有された `ThemeRef<T>` が返り、何もコピーされません)。速さと正しさのために守る規則が 2 つあります。

- 型は `PartialEq + Debug + Send + Sync` を満たす必要があります。2 つのテーマは拡張が等しいときに等しいとされるので、変わっていないテーマでスタイルが再計算されることはありません。
- 拡張が単純な値ではなくスタイル関数の表を持つ場合は、一度だけ作って共有してください (テーマのコンストラクターがクローンする `static OnceLock` など)。呼ぶたびに作り直した表は毎回別の表になり、その下の全部が再スタイルされます。

アプリの実行中にテーマを切り替えるには、ルートを `Signal<Theme>` つきの `provide_theme` で包みます ([アプリの実行中にテーマを切り替える](theming.md#アプリの実行中にテーマを切り替える))。`use_theme()` を呼んでいるスタイルのクロージャーはすべて、新しいテーマで再実行されます。

## アイコンはテーマ経由で描く

コンポーネントに画像を埋め込まないでください。アイコンに名前をつけ、その名前がどう見えるかはテーマに決めさせます。そうすれば、kit が自前のグリフをコンポーネントに与えられ、アプリも好きなものを差し替えられます。

```rust,test_harness
use kivact::ErrorKind;
use kivact::style::Icon;
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};

/// `name` という名前のアイコンを表示する正方形のボタン。
pub fn icon_button(name: impl IntoValue<Icon>, label: impl IntoText) -> impl Element {
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
    // 名前に対応する画像は、アプリ (または kit のテーマのコンストラクター) が渡します。
    let theme = Theme::light().with_icon("search", SEARCH);
    App::new("Icons").theme(theme).run(|| icon_button("search", "Search"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_theme_resolves_the_name() {
        let theme = Theme::light().with_icon("search", SEARCH);
        let options = RenderOptions::default().theme(theme);
        let screen = render_with(options, || icon_button("search", "Search"));
        screen.get_by_role(Role::Button, "Search");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn an_unknown_name_is_reported_in_debug_builds() {
        let screen = render_with(RenderOptions::default(), || icon_button("nope", "Nope"));
        screen.run_until_idle();
        if cfg!(debug_assertions) {
            let errors = screen.errors();
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].kind(), &ErrorKind::Asset);
        }
    }
}
```

`Icon::named("search")` と素の `"search"` は同じものなので、`impl IntoValue<Icon>` の prop は、名前、`Asset` (画像をそのまま使う)、そのどちらかのシグナルを受け取ります。標準セットには、`check` や `chevron-right` のように、標準コンポーネントが描くマークが入っています。自分の名前は `Theme::with_icon(name, image)` で足します。未知の名前は何も描かず、デバッグビルドでは一度だけ報告されます。

## アクセシブルな名前を本物のコントロールに付ける

アイコンとテキスト入力でできた検索ボックスのような複合コンポーネントでは、ルートはユーザーが入力するコントロールではありません。名前と無効の状態はそのコントロールに属します。ラッパーに残すと、読み上げソフトは名前のないテキストボックスを読み上げることになります。props として受け取り、コントロールに適用してください。

```rust,test_harness
use kivact::prelude::*;

/// 検索ボックス。アイコンとテキスト入力が、呼び出し側の名前と状態を持ちます。
#[component]
pub fn site_search(
    value: Binding<String>,
    #[prop(default = "Site search")] label: Value<String>,
    #[prop(default = false)] disabled: Value<bool>,
) -> impl IntoView {
    let input = text_input(label, value)
        .style(style().grow(1))
        .disabled(move || disabled.get());
    field(
        label,
        row((icon("search"), input)).style(style().gap(6).align_items(Align::Center)),
    )
}

fn app() -> impl IntoView {
    let query = signal(String::new());
    site_search(query).label("Site search")
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

`field(label, control)` も `Element` だけを受け取り、`#[component]` の型は受け取りません。呼び出し側がコンポーネントを field で包むべきなら、ラベルとヒントを props として受け取ってコンポーネントの中で `field` を組み立てるか、普通の関数から element のビルダーを返してください。

フォーカスは転送しなくてかまいません。field のラベルを押すと、その中で最初にフォーカスを受けられる要素（ここではテキストボックス）が操作されます。2 つ目のテストがそれを確かめています。

## コンポーネントの誤用を報告する

コンポーネントが受け入れられない引数を受け取ったときは、呼び出し側が原因を見られる場所で失敗させます。`kivact::custom::raise` は、Kivact 自身の誤用と同じように、呼び出し側で位置つきのメッセージとともにパニックします。アプリが終了し、テストなら失敗します。

```rust,no_run

use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::custom::raise;

#[track_caller]
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

プログラマーのミスにだけ使ってください。不正なファイルのようにユーザーが起こせる問題は、誤用ではなく、コンポーネントが表示する `Result` として扱います。

## `#[component]` が受け付けないもの

マクロは、ビルダーにできない署名を報告し、メッセージで直し方を示します。

- ジェネリックの引数や `where` 節: 代わりに `Value<T>` の prop にします。
- `impl IntoView` 以外の戻り値。
- メソッド: `#[component]` は free function です。
- 分解したパターン: 各 prop は名前つきの引数です。
- `Option<..>` でない型への `#[prop(optional)]`。
- `#[prop(optional)]` と `#[prop(default = ..)]` の併用。
- `#[prop(..)]` の未知の key。
- `async`、`const`、`unsafe`、`extern` の関数: `#[component]` の関数は普通の `fn` です。
- doc コメント、`allow`、`warn`、`deny`、`deprecated` 以外の属性（`#[expect(..)]` の代わりに `#[allow(..)]` を書きます）。

## 関連項目

- [エラーや更新の原因を調べる](diagnostics.md): 送出されたエラーがどうなるか、どう見るか。
- [独自の見た目のコントロールを作る](custom-controls.md): 標準の振る舞い (選択、メニュー、ドラッグ、テキスト編集) に自前の見た目を付けたいコンポーネント。
- [テーマとコンポーネントの見た目を変える](theming.md): アプリ側から見たトークン、recipe、アイコン。
- [画面をレイアウトする](layout.md): base style で設定できるスタイルのプロパティ。
- `cargo doc -p kivact --open`: `kivact::view::Element`、`kivact::view::Node`、`kivact::component`、`kivact::style::{Theme, ThemeExtension, IconSet}`。
