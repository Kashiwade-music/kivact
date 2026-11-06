# 1. Creative キットを設定する

[English](../../../en/creative/getting-started/01-set-up-the-kit.md)

このステップでは、プロジェクトに `kivact-ui-creative` を追加し、キットのテーマを適用して、インスペクターの最初の部品を表示します。数値フィールドが 1 つ入ったパネルです。

## 1. 依存関係を追加する

`cargo new layer-inspector` でプロジェクトを作り、`Cargo.toml` に Kivact と Creative キットを追加します。Kivact はまだ crates.io にないため、どちらも Git リポジトリから取得します。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
kivact-ui-creative = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

`dev-dependencies` の行は、ステップ 3 で使うテスト用の機能を有効にします。対応するツールチェーンは Rust 1.98.1（edition 2024）です。同じ環境にしたい場合は、Kivact のリポジトリの `rust-toolchain.toml` をコピーしてください。

## 2. パネルを表示する

`src/main.rs` を次の内容に置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

fn inspector() -> impl IntoView {
    let x = signal(120.0);
    ui::panel(
        "インスペクター",
        column((ui::property_row(
            "X",
            ui::number("X", x)
                .range(kivact::interaction::ScalarRange::new(-1000.0, 1000.0).expect("valid range"))
                .step(1.0)
                .precision(1)
                .unit(" px")
                .label_visible(false),
        ),)).role(kivact::view::Role::List)
        .style(style().gap(6).padding(10)),
    )
    .style(style().width(280))
}

fn main() -> kivact::Result<()> {
    App::new("レイヤーインスペクター")
        .theme(ui::theme(ThemeMode::Dark))
        .run(inspector)
}
```

`cargo run` で実行します。暗い色のウィンドウに、角の丸い **インスペクター** パネルが表示されます。パネルには 1 行あり、ラベル **X** と、`120.0 px` を表示するフィールドが並びます。

## 書いたコードの説明

- `ui::theme(ThemeMode::Dark)` は、Kivact の通常の `Theme` です。`App::theme` で渡すとウィンドウ全体に適用され、キットの部品も、`kivact::ui` の標準部品（ボタン、テキスト入力、メニューなど）も Creative の見た目になります。フォントファイルは不要で、文字にはシステムの `sans-serif` と `monospace` を使います（自分のフォントを使うには、[Creative キットのテーマ・フォント・アイコンを設定する](../how-to-guide/theme-and-icons.md#自分のフォントを使う)を参照してください）。
- `signal(120.0)` は、フィールドが編集する値です。`Copy` できるハンドルなので（SolidJS のシグナルと同じ感覚です）、複製せずに `x` を複数の場所へ渡せます。
- `ui::number("X", x)` は数値フィールドです。第 1 引数はアクセシビリティ上の名前で、`.label_visible(false)` を呼ばない限りフィールドの中にも描かれます（ここでは行のラベルがあるので隠しています）。第 2 引数の `x` が編集する値です。フィールドは自分用のコピーを持たず、書き込みはすべてあなたのシグナルへ直接届きます。
- `ui::property_row("X", field)` はインスペクターの行です。右寄せのラベルと、その右の値の部品で構成されます。
- `ui::panel(title, content)` は、タイトルバー付きの角の丸い「島」です。キットの部品はどれもビルダーなので、`.style(..)` のような通常の修飾子がそのまま使えます。

## 試してみる

フィールドを左右にドラッグしてみてください。フィールドをクリックして `2 * 8` と入力し、Enter を押すこともできます。計算式が使え、結果は指定した範囲に収められます。フィールドでできることは、次のページで詳しく見ます。

## 次へ

[2. 値を編集する](02-edit-values.md)
