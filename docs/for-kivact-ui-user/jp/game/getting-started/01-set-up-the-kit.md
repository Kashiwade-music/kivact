# ステップ 1: キットを導入する

[English](../../../en/game/getting-started/01-set-up-the-kit.md)

このステップでは、プロジェクトに Game キットを追加し、枠付きのパネル、キャプション、見出し、斜めのボタンを Game の見た目で表示するウィンドウを開きます。

## プロジェクトを作る

```text
cargo new loadout
cd loadout
```

Kivact とキットはまだ crates.io にありません。`Cargo.toml` に Git 依存として追加します。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
kivact-ui-game = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

`dev-dependencies` の行は、ステップ 3 で使うテスト用の機能を有効にします。

## キットとは何か

キットは**テーマ**です。標準コンポーネント（`button`、`list`、`tabs`、`text_input` など）は `kivact` クレートが提供し、テーマが描き方を決めるまでは素朴な見た目です。`kivact-ui-game` はそのテーマと、パネルの枠、メーター、バッジ、等幅のキャプションといった固有のコンポーネントを提供します。コアのチュートリアルと同じ `button(..)` を書き、キットが見た目だけを変えます。

## テーマを入れる

`src/main.rs` を次の内容に置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game::{self as game, Density, ThemeOptions};

#[component]
fn loadout() -> impl Element {
    game::hud(
        column((
            game::mono("SQUAD 04 / SLOT 2"),
            text("LOADOUT").style(style().font_size(24).font_weight(700)),
            button("Deploy").variant(kivact::ui::Slant),
        ))
        .style(style().gap(10).padding(20)),
    )
    .style(style().width(360).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Loadout")
        .theme(game::theme_with(ThemeMode::Dark, ThemeOptions { density: Density::Comfortable, ..Default::default() }))
        .run(loadout)
}
```

実行します。

```text
cargo run
```

暗い色のウィンドウに、枠線付きのパネルが現れます。2 つの角（左上と右下）には短い明るいカギ括弧があり、キャプションは等幅フォントで、出撃ボタンは右上に小さな黄色い四角が付いた平行四辺形に傾いています。ボタンにマウスを重ねると明るくなり、押すと 2 ピクセル沈みます。

新しく出てきたものは 4 つです。

- `game::theme(ThemeMode::Dark)` が Game テーマを作り、`App::theme(..)` がウィンドウ全体に適用します。結果は普通の Kivact の `Theme` です。`ThemeMode::Light` なら明るいコンソールの配色になります。
- `ThemeOptions { density: Density::Comfortable, ..Default::default() }` で寸法を選び、`game::theme_with` に渡します。Comfortable は高さ 34 px、パネル余白 20 px、Compact は高さ 28 px、余白 14 px です。
- `game::hud(..)` は Game のコンポーネントで、向かい合う 2 つの角にアクセント色のカギ括弧が付いた、枠線付きの面です。`game::mono(..)` は、ID やショートカット、機械的な値のための小さな等幅キャプションです。
- `.variant(kivact::ui::Slant)` は、キットが標準コンポーネントに見た目を足す方法です。ボタンはこれまでと同じ `button` のまま、描き方だけが変わります。

状態はこれまでどおりシグナルに持ちます。キットがデータを持つことはありません。

## フォント

Game はシステムのフォントで動くので、このチュートリアルでは何も足す必要はありません。テーマには 2 つのフォントファミリーの変数があります。見出し、ボタン、数字には `game::fonts::DISPLAY_FAMILY_VAR`、コードと識別子には `game::fonts::MONO_FAMILY_VAR` を使います。別のファミリーを名指すには、`ThemeOptions` の `display_font` と `mono_font` のフィールドを設定します。フォントはアプリのものです。詳しくは [テーマと密度を設定する](../how-to-guide/theme-and-density.md) を参照してください。

次へ: [ステップ 2: パネルを組み立てる](02-build-the-panel.md)
