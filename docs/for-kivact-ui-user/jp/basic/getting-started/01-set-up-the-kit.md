# 1. キットを準備する

[English](../../../en/basic/getting-started/01-set-up-the-kit.md)

このステップでは、新しいプロジェクトに `kivact-ui-basic` を追加し、キットのテーマをアプリに適用して、見出しだけのウィンドウを開きます。

## プロジェクトを作る

バイナリのプロジェクトを作り、依存を追加します。Kivact はまだ crates.io にないため、どちらの crate も Git リポジトリから取得します。

```text
cargo new settings
cd settings
```

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
kivact-ui-basic = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

Kivact は Rust 1.98.1、edition 2024 でビルドします（リポジトリの `rust-toolchain.toml` で固定しています）。`dev-dependencies` の行は `test` feature を有効にするもので、ステップ 3 のテストで使います。

## 最初の画面を書く

`src/main.rs` を次のプログラムに置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact::style::{Theme, ThemeMode};
use kivact_ui_basic as basic;

/// キットのテーマに、文字の family として Inter を指定したもの。
fn theme(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let text = Typography {
        family: Some("Inter, 'Noto Sans JP', sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

/// アプリに登録するフォントファイル。今は空で、ウィンドウはシステムのフォントを使います。
fn fonts() -> Vec<&'static [u8]> {
    Vec::new()
}

fn settings() -> impl IntoView {
    column((
        basic::h3("設定"),
        basic::muted("プロフィールと通知を管理します。"),
    ))
    .style(
        style()
            .size(pct(100), pct(100))
            .padding(24)
            .gap(8)
            .background(basic::Color::Background)
            .color(basic::Color::Foreground),
    )
}

fn main() -> kivact::Result<()> {
    let mut app = App::new("設定").theme(theme(ThemeMode::Light));
    for font in fonts() {
        app = app.font(font);
    }
    app.run(settings)
}
```

`cargo run` で実行します。白いウィンドウに「設定」という見出しと、その下に灰色の説明文が表示されます。

## コードの意味

- `basic::theme(mode)` は Kivact の `Theme` を返します。キットの配色、余白、12 px の文字、そして標準コントロール（button、switch、入力欄、select、dialog など）の描き方（recipe）をまとめたものです。これを `App::theme` に渡すだけで、アプリ内のすべての標準コンポーネントが Lyra の見た目になります。
- `basic::h3` と `basic::muted` はキットのタイポグラフィ用の関数です。どちらも view の部品である `Node` を返すので、好きな場所に置けます。
- `basic::Color` はキットの配色の名前（`Background`、`Foreground` など）です。これを使ったスタイルは、描画するときに有効なテーマから色を読みます。ステップ 3 でテーマを切り替えたときに色が追従するのはそのためです。ルートの view に背景色と文字色を塗っているのは、CSS で `body` に色を指定するのと同じで、ウィンドウ全体をテーマの色にするためです。
- `fonts()` はアプリが使うフォントファイルを返す場所です。キットはフォントを同梱しません。`theme` 関数の `Typography` は文字の family に `Inter` を指定し、代わりの候補として日本語用の `Noto Sans JP` と `sans-serif` を並べています。Inter を登録するまではシステムのフォントが使われますが、このチュートリアルはそのまま最後まで進められます。

## ダークの配色を試す

`main` の `ThemeMode::Light` を `ThemeMode::Dark` に変えて、もう一度実行します。背景がほぼ黒になり、文字が白くなります。確認したら `Light` に戻してください。実行中にモードを切り替える方法は、ステップ 3 で扱います。

## 任意: Inter を登録する

キットは Inter（本文）と Geist Mono（コードブロック）に合わせた寸法になっています。どちらも SIL OFL のフォントです。Inter を使うには、`Inter[opsz,wght].ttf` をダウンロードしてプロジェクトの `assets/fonts/` に置き、`fonts` からそのバイト列を返します。

```text
fn fonts() -> Vec<&'static [u8]> {
    vec![include_bytes!("../assets/fonts/Inter[opsz,wght].ttf") as &[u8]]
}
```

family 名はフォントファイルの中に書かれている名前で、ファイル名ではありません。以降のステップはこの設定に依存しません。

## 次へ

[2. 設定フォームを作る](02-build-the-settings-form.md) では、この画面にカードと入力欄とスイッチを置きます。
