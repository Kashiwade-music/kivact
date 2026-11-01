# テーマと密度を設定する

[English](../../../en/game/how-to-guide/theme-and-density.md)

Game の見た目を導入し、ライトとダーク、コンパクトな密度を選び、色とフォントを変え、自分のコンポーネントでキットの色を使うための手引きです。

## テーマを導入する

`game::theme(mode)` は、普通の Kivact の `Theme` を返します。`App::theme` に渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

fn app() -> impl IntoView {
    column((text("Console"), button("Run").variant(game::Slant)))
        .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Console")
        .theme(game::theme(ThemeMode::Dark).density(Density::Compact))
        .run(app)
}
```

このテーマは、`kivact::ui` のすべての標準コンポーネント（ボタン、入力欄、チェックボックス、スイッチ、リスト、タブ、ラジオグループ、スライダー、テーブル、メニュー、ダイアログ、トースト）とキット固有のコンポーネントを描き替えます。キットのアイコンも組み込まれるので、`icon("shield-check")` で描けます。`ThemeMode::Light` か `ThemeMode::Dark` はアプリが選びます。テーマが OS の設定を読むことはありません。

## 密度を選ぶ

`.density(..)` はキット唯一のオプションです。トレイト `ThemeOptions` のメソッドなので、スコープに入れておく必要があります（`use` 行の `ThemeOptions as _`）。変わるのはコンソールの寸法だけです。

| | `Density::Comfortable`（既定） | `Density::Compact` |
| --- | --- | --- |
| コントロールの高さ | 34 px | 28 px |
| テーブルの行の高さ | 42 px | 32 px |
| パネルの余白 | 20 px | 14 px |

色、フォントファミリー、そのほかのトークンはそのままなので、トークンを変えたあとに `.density(..)` を適用してもかまいません。情報密度の高いツールには `Compact`、プレイヤーが離れて読むメニューには `Comfortable` を使います。

## 実行中にモードと密度を切り替える

選択をシグナルに持たせ、`provide_theme` にテーマを作るクロージャを渡します。シグナルが変わるたびにすべてのスタイルが解決し直され、ビューの作り直しは起きないので、入力中の文字、スクロール位置、フォーカスはそのまま残ります。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::view::provide_theme;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

fn app() -> impl IntoView {
    let dark = signal(true);
    let compact = signal(false);
    let theme = move || {
        let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
        let density = if compact.get() { Density::Compact } else { Density::Comfortable };
        game::theme(mode).density(density)
    };
    provide_theme(theme, move || {
        column((
            switch("Dark", dark),
            switch("Compact", compact),
            text_input("Callsign", signal(String::new())),
        ))
        .style(
            style()
                .gap(12)
                .padding(24)
                .height(pct(100))
                .background(game::Color::Canvas)
                .color(ColorToken::Text),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Settings").run(app)
}
```

`provide_theme` はノードを作らず、何も塗りません。そのため、サブツリーの最上位ノードにページの色を指定します。`game::Color::Canvas` はキットのページ背景で、テーマの切り替えに追従します。

## 色とフォントを変える

キット固有の値は `game::Tokens` にあります。公開フィールドだけを持つ普通の構造体で、フォントファミリーの `display_family` と `mono_family`、色の `yellow`、`cyan`、`navy`、`canvas` など、寸法の `control_height`、`row_height`、`padding` を含みます。トークンをコピーして、フィールドを書き換え、`Theme::with` で入れ直します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::theme::Typography;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

fn console_theme() -> Theme {
    let theme = game::theme(ThemeMode::Dark).density(Density::Compact);

    // The kit's own tokens: display and monospaced families, an accent color.
    let mut tokens = (*theme.get::<game::Tokens>()).clone();
    tokens.display_family = "Barlow Condensed".into();
    tokens.mono_family = "JetBrains Mono".into();
    tokens.yellow = rgb(0xffb300);

    // Kivact's shared type settings: the font of ordinary text.
    let text = Typography {
        family: Some("Noto Sans JP".into()),
        ..theme.text().clone()
    };
    theme.with(tokens).with_text(text)
}

/// The font files are yours to embed, for example
/// `include_bytes!("../assets/BarlowCondensed-Bold.ttf")`.
fn application(display: &'static [u8], mono: &'static [u8], text: &'static [u8]) -> App {
    App::new("Console")
        .font(display)
        .font(mono)
        .font(text)
        .theme(console_theme())
}
# fn main() {}
```

- `display_family` はボタン、タブ、見出し、大きな数字に使われ、`mono_family` は `game::mono`、表のキャプションなど機械的なテキストに使われます。既定はそれぞれ `sans-serif` と `monospace` です。
- フォントはアプリが持ちます。ファイルを `App::font` で登録し、ファミリー名には、ファイル名ではなく、フォントの内部に書かれた名前を使います。
- 密度をトークンの編集より優先させたいなら、編集のあとで `.density(..)` を呼びます。密度は寸法のフィールド（`control_height`、`row_height`、`padding`）を置き換えるためです。

標準部品が使うキットの色（アクセント、成功、警告、危険、枠線）は、トークンではなくテーマのパレット（`theme.colors()` と `.with_colors(..)`）にあります。標準コンポーネントがパレットを読むためです。トークンには、キットが上乗せする値が入っています。アクセントをあらゆる場所で変えたいときは、気になるトークンに加えて `colors.accent` も変更します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game as game;

fn orange_theme() -> Theme {
    let theme = game::theme(ThemeMode::Dark);
    let mut colors = theme.colors().clone();
    colors.accent = rgb(0xff6a00);
    theme.with_colors(colors)
}
# fn main() {}
```

## 自分のコンポーネントでキットの色を使う

色を受け取るスタイルの引数に `game::Color::..` を渡せます。値はノードが描かれるテーマから引かれるので、クロージャを書かなくてもテーマの切り替えに追従します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn warning_strip(message: &'static str) -> Node {
    row((game::mono("WARN"), text(message))).style(
        style()
            .gap(10)
            .padding(8)
            .background(game::Color::Navy)
            .color(game::Color::Yellow)
            .border_left(3, game::Color::Cyan),
    )
}
# fn main() {}
```

標準の `ColorToken::Surface`、`Text`、`TextMuted`、`Border`、`Accent`、`Danger` はどのテーマでも動くので、他のキットでも使いたいコンポーネントにはこちらが向いています。`Tokens` のフィールドを直接読むのは、そこから新しい色を計算したいときだけにします。

## キットの文言を差し替える

コンポーネントが自分で描く文言（ダイアログの Close、コマンドパレットのヒント、表の "Progress" など）は英語です。`App::messages` で差し替えます。キーは各コンポーネントの rustdoc に載っています。

## 関連

- [一覧とツリーでアイテムを見せる](item-lists-and-selection.md)
- [メーターと状態を見せる](meters-and-status.md)
- すべてのトークンとコンポーネント: `cargo doc -p kivact-ui-game --open`
