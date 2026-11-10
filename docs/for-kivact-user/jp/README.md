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

既定では CPU で frame を描き、window が GPU を必要としたとき（ユーザーの shader、GPU export、GPU texture の共有）に GPU device を起動します。CPU の frame が表示周期の半分を続けて超える場合も起動します。使える GPU がなければ CPU のままです。実行時に CPU 描画を強制するには、環境変数 `ARVANE_RENDERER=cpu` を設定します。`ARVANE_RENDERER=gpu` は window の起動時に device を起動します。

### 小さいバイナリを配布する

facade は既定で 2 つの feature を有効にします。`gpu`（GPU レンダラー）と `complex-scripts`（空白で単語を区切らない言語のための ICU の分かち書き・行分割の辞書。タイ語、ラオ語、クメール語、ミャンマー語、中国語・日本語の分かち書きを含み、約 3 MB）です。その分かち書きが要らないアプリは `complex-scripts` を外し、ほかはそのまま使えます。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", default-features = false, features = ["gpu"] }
```

`gpu` も一覧から外せば、GPU レンダラーのないビルドになります。`kivact-bench` は `--no-default-features` のバイナリで `gpu` だけを有効にし、小さくなったビルドを測ります。`templates/basic/Cargo.toml` の `[profile.release]`（fat LTO、codegen-units 1、描画・shaping・layout の crate だけ `opt-level = 3` でほかは `"s"`、symbol の strip）がもう半分です。バイナリが小さくなり、hot path が速くなり、release のビルドは長くなります。

## どこから読むか

| セクション | 内容 |
| --- | --- |
| [getting-started/](getting-started/README.md) | チュートリアルです。小さな ToDo アプリを 1 ページずつ作ります。まずここから始めてください。 |
| [how-to-guide/](how-to-guide/README.md) | フォームの検証やダイアログの表示など、目的別のガイドです。 |
| [UI キット](../../for-kivact-ui-user/jp/README.md) | 公式の 5 つのコンポーネントキット (`kivact-ui-basic`、`kivact-ui-pop`、`kivact-ui-game`、`kivact-ui-creative`、`kivact-ui-soft`) とそのテーマです。 |
| API リファレンス | Kivact に依存するプロジェクト、またはリポジトリのクローンで `cargo doc -p kivact --open` を実行します。 |
