# Kivact アプリ開発者ガイド

[English](../en/README.md)

Kivact は Rust 向けのネイティブ GUI フレームワークです。コンポーネントは一度だけ実行される普通の関数で、細粒度のシグナルが、読み取った部分だけをウィンドウ上で更新します。React や SolidJS を知っていれば、考え方になじみやすいはずです。アプリのドメインロジックも同じ Rust のコードベースに置けます。

> **開発初期です。** Kivact にはまだリリースがなく、API は予告なく変わります。安定したビルドが必要な場合は、コミットを固定してください。

対象プラットフォームは Windows と Linux (Wayland / X11) です。macOS は動作確認していません。

```rust,no_run
use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("count: {}", count.get())),
        button("Increment").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}
```

## インストール

Kivact はまだ crates.io にありません。アプリは Git リポジトリに依存します。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

リポジトリは `rust-toolchain.toml` でツールチェーンを Rust 1.98.1 (edition 2024) に固定しています。このバージョン以降を使ってください。

既定では GPU で描画し、使える GPU がなければ自動で CPU 描画に切り替わります。GPU 対応なしでビルドするには、既定の feature を無効にします (`default-features = false`)。実行時に CPU 描画を強制するには、環境変数 `ARVANE_RENDERER=cpu` を設定します。

## どこから読むか

| セクション | 内容 |
| --- | --- |
| [getting-started/](getting-started/README.md) | チュートリアルです。小さな ToDo アプリを 1 ページずつ作ります。まずここから始めてください。 |
| [how-to-guide/](how-to-guide/README.md) | フォームの検証やダイアログの表示など、目的別のガイドです。 |
| [UI キット](../../for-kivact-ui-user/jp/README.md) | 公式の 5 つのコンポーネントキット (`kivact-ui-basic`、`kivact-ui-pop`、`kivact-ui-game`、`kivact-ui-creative`、`kivact-ui-soft`) とそのテーマです。 |
| API リファレンス | Kivact に依存するプロジェクト、またはリポジトリのクローンで `cargo doc -p kivact --open` を実行します。 |
