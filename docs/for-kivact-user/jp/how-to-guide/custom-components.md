# 再利用できるコンポーネントを書く

[English](../../en/how-to-guide/custom-components.md)

繰り返し使う UI のかたまりを、ほかのコードから再利用できるコンポーネントにするためのガイドです。チップ、カード、アクションつきの通知、検索ボックスなどが対象です。props、子要素、コールバックの受け取り方、呼び出し側が変更できる既定の見た目の持たせ方、独自の色をテーマに載せる方法、別のコントロールを包むときにアクセシビリティを保つ方法を扱います。入門チュートリアルを読んでいることを前提にしています。

コンポーネントはただの関数です。使われる場所の setup で一度だけ実行され、表示できるものを返します。ほとんどの場合、次の 2 つの形で足ります。

| 形 | 使う場面 |
|---|---|
| `impl Component` を返す関数 | props が少なく、すべて必須のとき。 |
| ルートの `Node` を持つビルダー構造体 | 省略可能な props、スロット、メソッドにしたほうが読みやすい設定があるとき。 |

## setup に独立した寿命を持たせる

普通の Rust 関数を呼んだだけでは、状態を解放する境界は増えません。呼び出し元と同じ scope に状態や cleanup が登録されます。部品専用の状態、context の読み取り、cleanup を、その部品の mount 位置に結び付けたい場合は `component(move || ...)` を使います。

```rust,no_run
use kivact::prelude::*;

fn local_counter() -> View {
    component(move || {
        let count = signal(0);
        on_cleanup(|| println!("counter removed"));
        column((
            text(move || count.get().to_string()),
            button("Add").on_press(move || count.update(|n| *n += 1)),
        ))
    })
}
```

この closure は新しい子 scope が mount されるときに一度だけ実行され、親の context を受け継ぎます。値の変更で closure 全体が再実行されることはありません。取り除かれたときは、その scope の cleanup が実行されます。`component` 自体は context の値を登録しません。子へ値を渡す目的なら `provide(value, factory)` を使います。

`IntoView::into_view` は view への変換であり、変換の呼び出しだけで新しい scope が生まれるわけではありません。ビルダーの最終設定を読み取った後、mount 時の setup が必要な部分を `component` の中に置きます。一度だけ mount する内容は `impl IntoView`、開き直すたびに作る内容は `impl IntoViewFn<V>` または `Fn(...) -> V` で受け取ります。

## まずは関数で書く

再利用できる最小のコンポーネントは、props を引数で受け取り `impl Component` を返します。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ChipTone { Normal, Accent }
constant_value!(ChipTone);

/// A reusable label with a reactive tone and an optional action.
pub fn chip(
    label: impl IntoText,
    tone: impl IntoValue<ChipTone>,
    on_press: impl Into<Callback<()>>,
) -> impl Component {
    let tone = tone.into_value();
    button(label)
        .on_press(on_press)
        .style(move || style().radius(12).padding_x(10).padding_y(4)
            .background(if tone.get() == ChipTone::Accent {
                ColorToken::Accent
            } else {
                ColorToken::SurfaceAlt
            }))
}

fn app() -> impl IntoView {
    let highlighted = signal(false);
    chip("Kivact", move || if highlighted.get() { ChipTone::Accent } else { ChipTone::Normal },
        move || highlighted.update(|value| *value = !*value))
        .test_id("chip")
}

fn main() -> kivact::Result<()> {
    App::new("Chips").run(app)
}
```

`constant_value!(ChipTone)` は独自の enum を定数の prop として渡せるようにします。
`impl IntoValue<ChipTone>` ならシグナルやクロージャーも渡せます。先頭で一度だけ
`into_value()` に変換すると、小さな `Copy` ハンドルの `Value` を共有できます。
payload のないコールバックは `impl Into<Callback<()>>` で受け取り、クロージャーも既存の
コールバックも渡せます。`impl Component` を返すと呼び出し側はルートの modifier を
続けられます。複数のルートには `impl IntoView` を使います。

独自の型を prop にするときは、その型を定義したクレートで一度だけ登録すると、定数をそのまま渡せるようになります。

```rust,no_run
use kivact::prelude::*;
use kivact::state::constant_value;

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Info,
    Warning,
}

constant_value!(Tone);

pub fn badge(label: impl IntoText, tone: impl IntoValue<Tone>) -> impl Component {
    let label = label.into_text_value();
    let tone = tone.into_value();
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

`constant_value!(Tone)` がないと、`badge("Saved", Tone::Info)` はコンパイルできません。`Tone` のシグナル、`Value`、クロージャーは登録なしでも渡せます。prop の型は `Clone + PartialEq` でなければなりません。値が変わっていないときに更新を省くためです。

## 子要素とスロットを受け取る

内容を包むコンポーネントは、それを `impl IntoView` で受け取ります。複数の子はタプルです。ビューのタプルもビューだからです。`()` は「ここには何もない」を表すので、空のスロットとしても使えます。名前つきスロットを持つコンポーネントは、スロットごとに引数を 1 つ取ります。

```rust,no_run
use kivact::prelude::*;

/// タイトルつきの箱。`body` は呼び出し側が入れたいもの、`footer` は `()` でもかまいません。
pub fn card(title: impl IntoText, body: impl IntoView, footer: impl IntoView) -> impl Component {
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

`impl IntoView` の値は、mount されると使い切られます。コンポーネントが内容を何度も mount する場合、たとえばセクションを開くたびに作り直す場合は、`show` と同じように、内容を作る関数を受け取ります。

```rust,no_run
use kivact::state::Binding;
use kivact::prelude::*;

/// 開くときに内容を作り、閉じるときに破棄するセクション。
pub fn collapsible<V: IntoView>(
    title: impl IntoText,
    open: impl IntoBinding<bool>,
    content: impl kivact::view::IntoViewFn<V>,
) -> impl Component {
    let title = title.into_text_value();
    let open = open.into_binding();
    column((
        button(title).on_press(move || open.set(!open.get())),
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

分割表示の各ペインのように、子ごとに別のラッパーが必要な場合は `kivact::view::IntoChildren` を受け取ります。16 個までのビューのタプル、または同じ種類のビューの `Vec` を受け取り、ラップできるよう各子を `View` として渡してくれます。

一度だけ mount する子は `impl IntoView` で受け取り `View` に保存します。作り直すスロットは
`impl IntoViewFn<V>` で受け取り `ViewFn<()>` に保存します。引数のある行は
`impl Fn(A) -> V + 'static` で受け取り `ViewFn<A>` に保存します。

## コールバックを受け取る

コールバックの prop は、呼び出し側が渡すクロージャーです。payload のないものは `impl Into<Callback<()>>` で受け取ります。

```rust,no_run
use kivact::prelude::*;
use kivact::view::{Handled, KeyEvent};

/// 削除ボタンつきのチップ。Delete キーでも削除できます。
pub fn removable_chip(label: impl IntoText, on_remove: impl Into<Callback<()>>) -> impl Component {
    let label = label.into_text_value();
    // `on_remove` は 2 つのハンドラーが使います。素のクロージャーは片方にしか
    // move できませんが、`callback` は保存して Copy のハンドルを返します。
    let remove = on_remove.into();
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

`callback` が必要な理由は Rust の事情です。キャプチャしたデータを所有するクロージャーは、1 か所にしか move できず、`Fn` の境界ではクローンもできません。`on_remove.into()` はクロージャーをアプリのアリーナに保存して `Callback` を返します。`Callback` は `Copy` なので、いくつの `move` クロージャーからでも持てます。呼び出しは `.call(引数)` です。入力のないコールバックには、上のように引数として `()` を渡します。

押下のハンドラーはイベントを受け取りません。React の `event.getModifierState("Control")` のように修飾キーが押されているかを知るには、`modifiers()` を読みます。

```rust,no_run
use kivact::prelude::*;
use kivact::view::modifiers;

/// Opens its panel; Ctrl+click (Cmd+click) opens only this one.
fn open_button<R: kivact::view::HandlerResult>(open: impl Fn(bool) -> R + 'static) -> impl Component {
    button("Open").on_press(move || {
        let held = modifiers().get();
        open(held.ctrl || held.meta)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Open").run(|| open_button(|only| println!("only this panel: {only}")))
}
```

呼び出し側のクロージャーが失敗するかもしれない場合は、`?` が使えるように境界を広げます。`impl Fn(E) -> R + 'static where R: HandlerResult` は `()` と `Result<(), UiError>` の両方を受け取り、`callback` もどちらも受け取れます。失敗は一番近い `error_boundary` に報告されます ([エラーや更新の原因を調べる](diagnostics.md))。

## props が増えたらビルダーにする

省略可能な props、スロット、設定が増えたら、長い引数リストよりビルダーのほうが読みやすくなります。ビルダーはルートの `Node` を持つ構造体で、`NodeBuilder` を実装して日常的な modifier をすべて受け取り、`IntoView` を実装して仕上げを担当します。

```rust,test_harness
use kivact::state::Callback;
use kivact::prelude::*;
use kivact::style::{Align, Direction, use_theme};
use kivact::view::View;
use kivact::state::constant_value;

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Info,
    Warning,
}

constant_value!(Tone);

pub struct Notice {
    root: Node,
    message: Value<String>,
    tone: Value<Tone>,
    action: Option<View>,
    on_dismiss: Option<Callback<()>>,
}

/// メッセージバー。`#[track_caller]` により、各通知はこのファイルの中の行ではなく、
/// 作られた場所で報告されます。
#[track_caller]
pub fn notice(message: impl IntoText) -> Notice {
    let message = message.into_text_value();
    Notice {
        root: node().role(Role::Alert).name(message),
        message,
        tone: Tone::Info.into_value(),
        action: None,
        on_dismiss: None,
    }
}

impl Notice {
    pub fn tone(mut self, tone: impl IntoValue<Tone>) -> Self {
        self.tone = tone.into_value();
        self
    }

    /// 再試行ボタンのような、追加のコントロール 1 つのためのスロット。
    pub fn action(mut self, action: impl IntoView) -> Self {
        self.action = Some(action.into_view());
        self
    }

    pub fn on_dismiss(mut self, f: impl Into<Callback<()>>) -> Self {
        self.on_dismiss = Some(f.into());
        self
    }
}

impl NodeBuilder for Notice {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Notice {
    fn into_view(self) -> View {
        component(move || {
            let Notice { root, message, tone, action, on_dismiss } = self;
            // 既定の見た目。呼び出し側が足すどのスタイルよりも下に置かれます。
            let mut root = root.base_style(move || {
                let (background, color) = match tone.get() {
                    Tone::Info => (ColorToken::Accent, ColorToken::AccentText),
                    Tone::Warning => (ColorToken::Warning, ColorToken::Text),
                };
                let theme = use_theme().get();
                style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(theme.space().sm)
                    .padding(theme.space().md)
                    .radius(theme.radius().md)
                    .width(240)
                    .background(background)
                    .color(color)
            });
            root = root.child(text(message).style(style().grow(1)));
            if let Some(action) = action {
                root = root.child(action);
            }
            if let Some(dismiss) = on_dismiss {
                root = root.child(button("Dismiss").on_press(move || dismiss.call(())));
            }
            root.into_view()
        })
    }
}

fn app() -> impl IntoView {
    let visible = signal(true);
    column((
        notice("Saved").test_id("plain"),
        notice("Narrow").style(style().width(120)).test_id("narrow"),
        show(visible, move || {
            notice("Offline")
                .tone(Tone::Warning)
                .action(button("Retry"))
                .on_dismiss(move || visible.set(false))
        }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Notices").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caller_style_beats_the_default_and_the_slots_work() {
        let screen = kivact::test::render(app);

        // 既定の幅は 240 です。コンポーネントの `base_style` が呼び出し側の
        // `.style(..)` より後に実行されても、呼び出し側のスタイルが勝ちます。
        let plain = screen.get_by_test_id("plain").rect().expect("laid out");
        let narrow = screen.get_by_test_id("narrow").rect().expect("laid out");
        assert_eq!((plain.width, narrow.width), (240.0, 120.0));

        screen.get_by_role(Role::Button, "Retry");
        screen.get_by_role(Role::Button, "Dismiss").click();
        assert!(screen.query_by_text("Offline").is_none());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

この形で知っておきたい点です。

- **`NodeBuilder` はメソッド 1 つ。** `fn node_mut(&mut self) -> &mut Node` を書くだけで、`style`、`name`、`disabled`、`test_id`、`on_key`、`on_press`、`draw` などがビルダーで使えます。コンストラクターのあとで呼び出し側が書いた modifier は、コンストラクターが設定した値に勝ちます。
- **ルートを変えるには `with_node` を使う。** `role`、`checked`、`expanded`、`selected`、`value_text`、`base_style` は `Node` の固有メソッドです。ビルダーでは `.with_node(|root| root.role(Role::Alert))` と書きます。
- **仕上げは `into_view` で行う。** `into_view` ではすべての設定を受け取ったビルダーを view に変換します。mount 時の setup は `component(move || ...)` の中で一度だけ実行します。setter とスタイルのクロージャーのあいだで `Cell` を共有するのではなく、ここでフィールドを読みます。
- **状態はインスタンスに属する。** `component` の closure 内で作ったシグナルは、その子 scope が所有します。コンストラクターや通常の関数内で作るだけなら、呼び出した側の scope に属します。したがって同じ画面の 2 つの通知は何も共有しません。文字列キーやグローバルな ID は要りません。
- **`#[track_caller]` で各インスタンスに名前がつく。** 診断やテスト失敗のレポートは、ノードをビルダーが呼ばれた位置で呼びます。コンストラクターにこの属性を付ければ、各インスタンスは自分の呼び出し位置で報告されます。あとから (`into_view` の中で) ノードを作るコンポーネントは、追跡するコンストラクターでルートに `.located_at_caller()` を付けます。

### 呼び出し側が上書きできる既定のスタイル

`base_style` は、呼び出し側のすべての `.style(..)` の**下**にスタイルの層を足します。呼び出しの順序には関係ありません。コンポーネントの既定の見た目を置くのに適した場所です。上のテストがその例で、通知は既定の幅として `width(240)` を設定しますが、`.style(style().width(120))` が勝ちます。コンポーネントの中で `.style(..)` を使うのは、呼び出し側に勝たせたいスタイルのときだけにしてください。そういう場面はまれです。

base style にはクロージャーを渡せます。クロージャーの中でシグナル、`Value`、`use_theme()` を読むと、スタイルはその変化に追従し、再 mount は起きません。

## 独自のデザイントークンをテーマに載せる

コンポーネントに色を直接書くと、テーマに追従できません。コンポーネントの色、寸法、フォントは、`ThemeExtension` を実装した型としてテーマに載せましょう。テーマを作る人が値を設定できます。その型を知らないテーマには、標準のパレットから導いた fallback が渡されるので、どのテーマの下でもコンポーネントは描画できます。

```rust,test_harness
use kivact::prelude::*;
use kivact::style::{Color, ThemeExtension, use_theme};
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

pub fn status_dot(online: impl IntoValue<bool>) -> impl Component {
    let online = online.into_value();
    node()
        .role(Role::Status)
        .name(move || if online.get() { "online" } else { "offline" }.to_owned())
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
pub fn icon_button(name: impl IntoValue<Icon>, label: impl IntoText) -> impl Component {
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

## アクセシビリティを本物のコントロールへ転送する

アイコンとテキスト入力でできた検索ボックスのような複合コンポーネントでは、ルートはユーザーが入力するコントロールではありません。それでも呼び出し側はコンポーネントに `.name("Site search")`、`.disabled(..)`、`.invalid(..)` を書き、`field` は `.labelled_by(..)` や `.described_by(..)` で包みます。これらの設定はルートに付きます。ルートから読み取って本物のコントロールに適用しないと、読み上げソフトは名前のないテキストボックスを読み上げることになります。

```rust,test_harness
use kivact::state::Binding;
use kivact::prelude::*;
use kivact::view::View;

pub struct SearchBox {
    root: Node,
    value: Binding<String>,
}

#[track_caller]
pub fn search_box(value: impl IntoBinding<String>) -> SearchBox {
    SearchBox {
        root: row(()).style(style().gap(6).align_items(Align::Center)),
        value: value.into_binding(),
    }
}

impl NodeBuilder for SearchBox {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SearchBox {
    fn into_view(self) -> View {
        component(move || {
            let SearchBox { mut root, value } = self;

            // 呼び出し側がコンポーネント全体について指定したこと。
            let name = root.accessible_name();
            let disabled = root.accessible_disabled();
            let invalid = root.accessible_invalid();
            let labelled_by = root.accessible_labelled_by();
            let described_by = root.accessible_described_by();

            // 呼び出し側が名前を渡したらそれを使い、なければコンポーネント自身の名前を使います。
            let label: Value<String> = match name {
                Some(read) => read,
                None => "Search".into_text_value(),
            };

            // それらすべてを、キー入力を受けるコントロールへ渡します。
            let mut input = text_input(label, value).style(style().grow(1));
            if let Some(read) = disabled {
                input = input.disabled(move || read.get());
            }
            if let Some(read) = invalid {
                input = input.invalid(move || read.get());
            }
            for target in labelled_by {
                input = input.with_node(|host| host.labelled_by(target));
            }
            for target in described_by {
                input = input.with_node(|host| host.described_by(target));
            }

            root.child((icon("search"), input)).into_view()
        })
    }
}

fn app() -> impl IntoView {
    let query = signal(String::new());
    field("Site search", search_box(query)).hint("Searches every page")
}

fn main() -> kivact::Result<()> {
    App::new("Search").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_box_carries_the_name_and_the_description() {
        let screen = kivact::test::render(app);
        let input = screen.get_by_role(Role::TextBox, "Site search");
        assert_eq!(input.described_by().len(), 1);
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

`accessible_*` の読み出しは、呼び出し側の設定を消費せずにコピーします。そのためルートも自分の設定を保ったままです。コンポーネント全体に対して呼び出し側が設定でき、かつ内部のどれか 1 つの部品に属する情報なら、どれも同じ手順で転送できます。

フォーカスは転送しなくてかまいません。field のラベルを押すと、ルートの中で最初にフォーカスを受けられる要素（ここではテキストボックス）が操作されます。2 つ目のテストがそれを確かめています。

## コンポーネントの誤用を報告する

コンポーネントが受け入れられない引数を受け取ったときは、呼び出し側が原因を見られる場所で失敗させます。`kivact::custom::raise` は、Kivact 自身の誤用と同じように、呼び出し側で位置つきのメッセージとともにパニックします。アプリが終了し、テストなら失敗します。

```rust,no_run

use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::custom::raise;

#[track_caller]
pub fn progress_ring(done: u32, total: u32) -> impl Component {
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

## 関連項目

- [エラーや更新の原因を調べる](diagnostics.md): 送出されたエラーがどうなるか、どう見るか。
- [独自の見た目のコントロールを作る](custom-controls.md): 標準の振る舞い (選択、メニュー、ドラッグ、テキスト編集) に自前の見た目を付けたいコンポーネント。
- [テーマとコンポーネントの見た目を変える](theming.md): アプリ側から見たトークン、recipe、アイコン。
- [画面をレイアウトする](layout.md): base style で設定できるスタイルのプロパティ。
- `cargo doc -p kivact --open`: `kivact::view::NodeBuilder`、`kivact::view::Node`、`kivact::style::{Theme, ThemeExtension, IconSet}`。


セットアップ中にリアクティブな状態を作る部品は `component(move || view)` を返せます。
初期化処理は mount 時に一度だけ実行され、親の context を継承します。effect と
cleanup はその部品に属し、取り除くと破棄されます。表示上の箱は追加しません。
一度だけ mount する slot は `impl IntoView`、popup を開くたびに作り直す内容は
`impl Fn() -> V` で受け取ります。
