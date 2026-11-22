# テーマを適用、切り替え、カスタマイズする

[English](../../../en/basic/how-to-guide/theme-and-tokens.md)

この手引きでは、アプリに Basic の見た目を与え、実行中にライトとダークを切り替え、キットの色を読み書きし、標準コンポーネントの描き方を調整し、Lucide のアイコンを使う方法を説明します。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

## テーマを適用する

`basic::theme(mode)` を `App::theme` に渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

fn main() -> kivact::Result<()> {
    App::new("Basic")
        .theme(basic::theme(ThemeMode::Light))
        .run(|| button("Save"))
}
```

テーマは、Neutral のパレット、12 px の文字、角の四角い見た目、32 px の操作部品、そして `kivact::ui` のすべての標準コンポーネントの描き方（recipe）を持ちます。対象は `button`、`toggle`、`checkbox`、`switch`、`text_input`、`text_area`、`select`、`menu`、`dialog`、`tooltip`、`tabs`、`radio_group`、`slider`、`range_slider`、`toast_region`、`splitter`、`split`、`progress`、`disclosure`、`data_table` などです。キットはこれらの部品のコピーを持ちません。`kivact::ui::button` と `kivact::ui::card` を並べても糊は要らず、テーマを変えれば両方が一緒に描き替わるのはそのためです。

見た目をウィンドウの一部だけに限るには、その部分を `kivact::style::provide_theme(theme, || ..)` で包みます。外側の兄弟は、上位のテーマを保ちます。

## 実行中にライトとダークを切り替える

固定のテーマではなく、シグナルを読むクロージャを渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

#[component]
fn root() -> impl IntoView {
    let dark = signal(false);
    kivact::style::provide_theme(
        move || {
            basic::theme(if dark.get() { ThemeMode::Dark } else { ThemeMode::Light })
        },
        move || {
            column((
                switch("Dark mode", dark),
                button("Save"),
                text_input(signal(String::from("Ada"))).label("Name"),
            ))
            .style(
                style()
                    .size(pct(100), pct(100))
                    .padding(24)
                    .gap(12)
                    .background(basic::BACKGROUND)
                    .color(basic::FOREGROUND),
            )
        },
    )
}

fn main() -> kivact::Result<()> {
    App::new("Theme").run(root)
}
```

- 配置済みの部品は作り直されず描き替えられるので、入力中の文字、フォーカス、開いている状態は切り替えのあとも残ります。
- `provide_theme` は何も描きません。サブツリーの最上位ノードには、上の例のようにページの色を自分で指定してください。色のないノードの下の文字は、`App` のテーマの色を受け取ります。
- モードをどう保存するかはアプリが決めます。実行をまたいで覚えさせたいなら、`dark` の値を自分で保存し、シグナルを作るときに読み戻してください。
- クロージャの中でテーマを組み立てても、コストはわずかです。キットはテーマのあいだで recipe の表とアイコンの集合を共有するので、表が変わらなければ何も描き替わりません。

## フォントを選ぶ

キットはフォントを同梱しません。Inter に合わせた大きさで、コードのブロックとインラインのコードには `Geist Mono` を指定します。本文のファミリーはテーマの `font::FAMILY_VAR` に書き、ファイルは `Theme::with_fonts` で登録します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{FontFamily, Theme, ThemeMode, font};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    // Add a CJK family after Inter when the app shows Japanese, Chinese or Korean.
    basic::theme(mode).set(&font::FAMILY_VAR, FontFamily::from("Inter, 'Noto Sans JP', sans-serif"))
}

fn app(fonts: &[&'static [u8]]) -> App {
    App::new("Basic").theme(theme(ThemeMode::Light).with_fonts(fonts.iter().copied()))
}
```

`main` の中で `include_bytes!` のファイルからバイト列を渡します（`app(&[include_bytes!("../assets/Inter[opsz,wght].ttf") as &[u8], include_bytes!("../assets/GeistMono[wght].ttf")])`）。ファミリー名は、ファイル名ではなく、フォントファイルの中に書かれた名前です。登録していないファミリーは、システムのフォントに戻ります。

コンポーネントが自分で表示する文言（Loading、Close、Copy、Next など）は英語です。`App::messages` で差し替えます。キーには `basic.` の接頭辞が付き、キットの README に一覧があります。

```rust,no_run
use kivact::i18n::Messages;
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn main() -> kivact::Result<()> {
    let messages = Messages::new()
        .with("basic.pagination.next", "Weiter")
        .with("basic.pagination.previous", "Zurück");
    App::new("Basic")
        .theme(basic::theme(kivact::style::ThemeMode::Light))
        .messages(messages)
        .run(|| kivact::ui::pagination(5usize, signal(0usize)))
}
```

## 自分のスタイルでキットの色を使う

キットの色は `kivact_ui_basic` の定数です。`basic::BACKGROUND`、`FOREGROUND`、`CARD`、`CARD_FOREGROUND`、`POPOVER`、`PRIMARY`、`PRIMARY_FOREGROUND`、`SECONDARY`、`MUTED`、`MUTED_FOREGROUND`、`ACCENT`、`DESTRUCTIVE`、`BORDER`、`INPUT`、`RING`、`CHART_1` から `CHART_5`、そして `SIDEBAR_*` の色があります。それぞれがテーマの変数です。色を受け取るスタイルのメソッドは、どれでもこれを受け取れます。描かれるときの、そのノードで有効なテーマから値が引かれるので、テーマの切り替えに追従します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn banner() -> impl Element {
    text("Heads up").style(
        style()
            .padding(12)
            .background(basic::MUTED)
            .color(basic::MUTED_FOREGROUND)
            .border(1, basic::BORDER),
    )
}
```

テーマの色から新しい色を計算したいとき（半透明の primary など）は、スタイルのクロージャの中でテーマの変数を読みます。テーマが変わるたびに、クロージャは再実行されます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::use_theme;
use kivact_ui_basic as basic;

#[component]
fn tinted() -> impl Element {
    text("Tinted").style(|| {
        let primary = use_theme().get().get(&basic::PRIMARY_VAR);
        style().padding(12).background(primary.with_alpha(0.1))
    })
}
```

## 色を変える

色はそれぞれ変数なので、キットが返すテーマに `Theme::set` を使って変えます。変数を設定すると、それを読むすべてのコンポーネントとスタイルが追従します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode, rgb};
use kivact_ui_basic as basic;

fn blue_theme(mode: ThemeMode) -> Theme {
    basic::theme(mode)
        .set(&basic::PRIMARY_VAR, rgb(0x2563eb))
        .set(&basic::PRIMARY_FOREGROUND_VAR, rgb(0xffffff))
        .set(&basic::RING_VAR, rgb(0x93c5fd))
}

fn main() -> kivact::Result<()> {
    App::new("Blue")
        .theme(blue_theme(ThemeMode::Light))
        .run(|| button("Save"))
}
```

気をつけることがいくつかあります。

- 変数は shadcn/ui の CSS の変数に対応します。`BACKGROUND`、`FOREGROUND`、`CARD`、`POPOVER`、`PRIMARY`、`SECONDARY`、`MUTED`、`ACCENT`、`DESTRUCTIVE`、`BORDER`、`INPUT`、`RING`、`CHART_1` から `CHART_5`、`SIDEBAR_*` の色、そのほかのキットの色です。それぞれ、`set` と `get` のための `_VAR` の static を持ちます。
- アプリがライトとダークを切り替えるなら、それぞれのモードの値を別々に設定します。`blue_theme` が渡されたモードに対してテーマを作るように、モードごとにテーマを作ってください。
- 変えた色は標準コンポーネントにも届きます。recipe が、有効なテーマの変数を読むからです。
- キットは、自分の色から `kivact::style::color` の中核の色の変数（`ACCENT`、`TEXT`、`BORDER`、`FOCUS`、`DANGER` など）を設定するので、キットの外のコンポーネントもキットに従います。中核の色を 1 つだけ変えたいときは、`basic::theme` のあとで、その中核の変数に `set` します（例: `.set(&color::FOCUS_VAR, rgb(0x2563eb))`）。

## 1 つの標準コンポーネントの描き方を調整する

recipe は、部品の状態をスタイルに変えるルールです。すべてのボタンの見た目を変えるには、テーマに `ButtonTheme` の part を入れます。その `root` のクロージャは、キットが描くスタイルを受け取って、変えたスタイルを返します。そのため、キットのそれ以外の働き（variant、サイズ、ホバーやフォーカスの見た目）は残ります。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode};
use kivact::ui::theme::ButtonTheme;
use kivact_ui_basic as basic;

fn rounded_buttons(mode: ThemeMode) -> Theme {
    basic::theme(mode).with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(8)))
}
```

1 つのインスタンスだけを変えたいときは、テーマには触れず `.style(..)` を渡します。呼出し側のスタイルは、標準コンポーネントでもキット自身のビルダーでも、常にキットの既定値より優先されます。

## アイコンを使う

`basic::ICON_NAMES` にある Lucide のアイコンは、約 125 の名前（`check`、`search`、`settings`、`trash-2` など）すべてがテーマのアイコンの集合に入っています。中核の `icon(name)` で描きます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn toolbar() -> impl Element {
    assert!(basic::ICON_NAMES.contains(&"settings"));
    row((
        button("Download").icon(icon("download")),
        button("Settings").icon(icon("settings")).hide_label(true),
        kivact::ui::badge("New").icon(icon("sparkles")),
    ))
    .style(style().gap(8))
}
```

- アイコンは 16 px で、現在の文字色で描かれます。ボタンのアイコンの枠のような小さな箱では、収まるように縮みます。手で大きさを変えるには、サイズを指定します（`icon("info").style(style().size(24, 24))`）。
- 1 つのビューを受け取る 1 つのスロット（`alert(..).icon(..)`、`badge(..).icon(..)`、`button(..).icon(..)`）には、`icon(name)` を渡します。項目がレコードのとき（`ListItem`、`MenuEntry`）は、名前を文字列で渡します（`MenuEntry::item("save", "Save").with_icon("save")`）。
- アイコンは装飾です。アイコンだけのボタンは、アクセシブルな名前をラベルから受け取るので、`button("Settings").icon(..).hide_label(true)` と書いてラベルを残してください。
- 集合にない名前は何も描きません。デバッグビルドでは、見つからない名前を呼出し元の場所に報告するので、綴りの間違いは見過ごされません。
- 自分のアイコンを追加するには、SVG のバイト列を `with_icon` でテーマに入れます。ファイル内の線や塗りの色は、文字色に置き換えられます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

static STAR: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#000' stroke-width='2'><path d='M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1L3.2 9.5l6.1-.9z'/></svg>";

fn main() -> kivact::Result<()> {
    let theme = basic::theme(ThemeMode::Light).with_icon("star", STAR);
    App::new("Icons")
        .theme(theme)
        .run(|| button("Favorite").icon(icon("star")))
}
```

## 関連

- [オーバーレイ](overlays.md): このテーマでのダイアログ、シート、ポップオーバー、メニュー、トースト。
- [ナビゲーション](navigation.md)、[データを表示する](data-display.md)、[フォームを作る](forms.md): キットのコンポーネントの各分野。
- すべてのコンポーネントとそのソース: リポジトリのクローンで `cargo run -p kivact-ui-basic-catalog` を実行します。
- API の全体: `cargo doc -p kivact-ui-basic --open`。
