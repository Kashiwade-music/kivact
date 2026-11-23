# テーマと密度を設定する

[English](../../../en/game/how-to-guide/theme-and-density.md)

この手引きでは、Game の見た目を導入し、ライトかダークかを選び、コンパクトな密度に切り替え、色とフォントを変え、自分のコンポーネントでキットの色を使う方法を説明します。

## テーマを導入する

`game::theme(mode)` は普通の Kivact の `Theme` を返します。これを `App::theme` に渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game::{self as game, Density, ThemeOptions};

fn app() -> impl Element {
    column((text("Console"), button("Run").variant(kivact::ui::Slant)))
        .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Console")
        .theme(game::theme_with(ThemeMode::Dark, ThemeOptions { density: Density::Compact, ..Default::default() }))
        .run(app)
}
```

このテーマは `kivact::ui` のすべての標準部品（ボタン、入力欄、チェックボックス、スイッチ、リスト、タブ、ラジオグループ、スライダー、表、メニュー、ダイアログ、トースト）と、キット独自の部品の見た目を決めます。キットのアイコンも入るので、`icon("shield-check")` でアイコンを描けます。`ThemeMode::Light` と `ThemeMode::Dark` のどちらを使うかは自分で選びます。テーマは OS の設定を読みません。

## 密度を選ぶ

`ThemeOptions { density, ..Default::default() }` はキットのオプションで、`game::theme_with(mode, options)` に渡します。変わるのはコンソールの寸法だけです。

| | `Density::Comfortable`（既定） | `Density::Compact` |
| --- | --- | --- |
| コントロールの高さ | 34 px | 28 px |
| 表の行の高さ | 42 px | 32 px |
| パネルの余白 | 20 px | 14 px |

色とフォントファミリーは変わりません。密度の高い道具には `Compact`、プレイヤーが離れて読むメニューには `Comfortable` を使います。

## 実行中にモードと密度を切り替える

選択を signal に持たせ、テーマを作るクロージャを `provide_theme` に渡します。signal が変わると、すべてのスタイルが解決し直されます。何も作り直されないので、入力中の文字、スクロールの位置、フォーカスは保たれます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::provide_theme;
use kivact_ui_game::{self as game, Density, ThemeOptions};

fn app() -> impl IntoView {
    let dark = signal(true);
    let compact = signal(false);
    let theme = move || {
        let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
        let density = if compact.get() { Density::Compact } else { Density::Comfortable };
        game::theme_with(mode, ThemeOptions { density, ..Default::default() })
    };
    provide_theme(theme, move || {
        column((
            switch("Dark", dark),
            switch("Compact", compact),
            text_input(signal(String::new())).label("Callsign"),
        ))
        .style(
            style()
                .gap(12)
                .padding(24)
                .height(pct(100))
                .background(game::color::CANVAS)
                .color(kivact::style::color::TEXT),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Settings").run(app)
}
```

`provide_theme` はノードを作らず、何も描かないので、サブツリーの最上位のノードにページの色を指定します。`game::color::CANVAS` はキットのページ背景で、テーマが切り替わると一緒に変わります。

## 色とフォントを変える

キット固有の値はテーマの変数です。色は `kivact_ui_game::color` にある `ThemeVar<Color>` の static（例：`CYAN_VAR`）、フォントファミリーは `kivact_ui_game::fonts` にある `ThemeVar<FontFamily>` の static です。見出し、ボタン、数字に使う `DISPLAY_FAMILY_VAR`、コードと識別子に使う `MONO_FAMILY_VAR` があります。同じ名前の `const`（`CYAN`）は、スタイルの中で変数を読みます。変数は `Theme::set` で変えます。`Theme::set` はテーマを返すので、続けて何個でも呼べます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{FontFamily, ThemeMode, rgb};
use kivact_ui_game::{self as game, Density, ThemeOptions};

fn console_theme() -> Theme {
    game::theme_with(ThemeMode::Dark, ThemeOptions { density: Density::Compact, ..Default::default() })
        // The kit's own variables: an accent color and the display and monospaced families.
        .set(&game::color::CYAN_VAR, rgb(0xffb300))
        .set(&game::fonts::DISPLAY_FAMILY_VAR, FontFamily::from("Barlow Condensed"))
        .set(&game::fonts::MONO_FAMILY_VAR, FontFamily::from("JetBrains Mono"))
}

/// The font files are yours to embed, for example
/// `include_bytes!("../assets/BarlowCondensed-Bold.ttf")`.
fn application(display: &'static [u8], mono: &'static [u8]) -> App {
    App::new("Console")
        .theme(console_theme().with_fonts([display, mono]))
}
# fn main() {}
```

- `DISPLAY_FAMILY_VAR` はボタン、タブ、見出し、大きな数字に使われます。`MONO_FAMILY_VAR` は `game::mono`、KPI のキャプション、表のキャプションに使われます。どちらも既定では `sans-serif` と `monospace` です。
- フォントはアプリのものです。フォントファイルは `Theme::with_fonts` に渡し、ファイル名ではなくフォントの中に書かれたファミリー名を使います。
- `ThemeOptions` には `display_font` と `mono_font` のフィールドもあり、`theme_with` でテーマを作るときに 2 つのファミリーを設定します。
- 変数を変える前に、`theme_with` で密度を選びます。

標準の部品が使う、キットの基本の色（アクセント、成功、警告、危険、境界線）は `kivact::style::color` のテーマの変数です。標準の部品はこれらを読むためです。アクセントをすべての場所で変えるには、必要なキットの変数と一緒に `color::ACCENT_VAR` も設定します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ThemeMode, color, rgb};
use kivact_ui_game as game;

fn orange_theme() -> Theme {
    game::theme(ThemeMode::Dark).set(&color::ACCENT_VAR, rgb(0xff6a00))
}
# fn main() {}
```

## 自分のコンポーネントでキットの色を使う

色を受け取るスタイルの引数には、`game::color::NAVY` などの `game::color::*` の定数を渡せます。値はノードが描かれるテーマから読まれるので、クロージャを書かなくてもテーマの切り替えに追従します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

#[component]
fn warning_strip(message: Value<String>) -> impl Element {
    row((game::mono("WARN"), text(message))).style(
        style()
            .gap(10)
            .padding(8)
            .background(game::color::NAVY)
            .color(game::color::YELLOW)
            .border_left(3, game::color::CYAN),
    )
}
# fn main() {}
```

標準の `kivact::style::color::SURFACE`、`TEXT`、`TEXT_MUTED`、`BORDER`、`ACCENT`、`DANGER` はどのテーマでも動くので、他のキットでも使いたいコンポーネントにはこちらが向いています。

## キットの文言を差し替える

部品が自分で描く文言（ダイアログの閉じる、コマンドパレットのヒント、表の「Progress」など）は英語です。これらは `App::messages` で差し替えます。キーは各部品のリファレンスに載っています。

## 関連項目

- [リストとツリーで項目を見せる](item-lists-and-selection.md)
- [メーターと状態を見せる](meters-and-status.md)
- すべてのトークンと部品：`cargo doc -p kivact-ui-game --open`
