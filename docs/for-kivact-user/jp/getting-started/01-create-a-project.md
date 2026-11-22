# ステップ 1: プロジェクトを作る

[English](../../en/getting-started/01-create-a-project.md)

このステップでは、Cargo プロジェクトを作り、「Tasks」と表示するウィンドウを開きます。

## プロジェクトを作る

```text
cargo new tasks
cd tasks
```

Kivact はまだ crates.io にないので、`Cargo.toml` に Git 依存として追加します。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
```

開発中は、Kivact のレイアウトと描画のコードが最適化なしでは遅すぎます。そのため、`Cargo.toml` に次の設定も加えます。依存する crate はすべてデバッグビルドでも最適化され、自分のコードはデバッグしやすいままになります。

```toml
[profile.dev.package."*"]
opt-level = 3
```

Kivact には Rust 1.98.1 以降が必要です。新しいツールチェーンなら、`cargo new` は edition 2024 のプロジェクトを作るので、そのままで問題ありません。

## いちばん小さなウィンドウ

`src/main.rs` の内容を次のプログラムに置き換えます。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    text("Tasks")
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

`use kivact::prelude::*;` は、`App`、`text`、`column`、`signal`、`binding`、`callback` など、よく使う名前をまとめて取り込みます。以降のページでも、ファイルの先頭に必ず書きます。

`App::new("Tasks")` はアプリを表し、引数の文字列がウィンドウのタイトルになります。`.run(app)` はウィンドウを開き、閉じるまで動き続けます。`app` は一度だけ呼ばれ、返された view がウィンドウの中身になります。

`app` はルートの**ビュービルダー**です。`impl IntoView` を返す、普通の関数です。`impl IntoView` は「画面に表示できるもの」という意味で、具体的な型名を書く必要はありません。`text("Tasks")` はもっとも単純な view で、1 行の文字です。`app` は、このチュートリアルで唯一コンポーネントではない view 関数です。次のステップから、画面の各部分がコンポーネントになります。

`main` は `kivact::Result<()>` を返します。ウィンドウを開けないなどの起動時のエラーは、無視されずにプログラムの終了時に報告されます。

## 実行する

```text
cargo run
```

初回は Kivact をソースからビルドするため、時間がかかります。ビルドが終わると、「Tasks」というタイトルのウィンドウが開き、左上に「Tasks」と表示されます。ウィンドウを閉じるとプログラムが終わります。

次へ: [ステップ 2: コンポーネントとレイアウト](02-components-and-layout.md)
