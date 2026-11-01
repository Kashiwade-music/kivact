# Kivact

> Kivact is a native GUI framework for Rust, built from plain functions and fine-grained signals. It is in early development: APIs change without notice, and pull requests are not accepted for now. The documentation is written in Japanese.

Kivact は、普通の Rust 関数で native GUI を作る初期開発版のフレームワークです。部品の setup は一度実行され、signal の変更は値を読む表示へ反映されます。まず `kivact` だけで画面を作れます。button・checkbox・switch・text_input などの標準部品は `kivact::ui` にあり、見た目はテーマが決めます。

```rust,no_run
use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("count: {}", count.get())),
        button("増やす").on_press(move || count.update(|value| *value += 1)),
    )).style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}
```

リポジトリのルートで `cargo run -p kivact-example-counter` を実行してください。[入門](docs/for-kivact-user/getting-started/README.md) は Rust 初学者向けにカウンター、入力、状態、一覧、非同期へ進みます。

React・SolidJSの経験がある方は[対応ガイド](docs/for-kivact-user/getting-started/react-solid.md)から概念の違いを確認できます。

標準部品の見た目をまとめて差し替えるテーマと分野固有の部品は、公式 kit の `kivact-ui-basic`、`kivact-ui-pop`、`kivact-ui-game`、`kivact-ui-creative` です。[公式 UI ガイド](docs/for-kivact-ui-user/README.md) に各キットの入門と作業別手順があります。各キットと外部部品は `kivact` の公開 API を使います。

[目的別ガイド](docs/for-kivact-user/how-to-guide/README.md)はフォーム、一覧・表、overlay、描画、非同期の platform 操作などを扱います。自社のデザインシステムや独自の入力部品は、[部品作者向けガイド](docs/for-component-author/README.md)の interaction と primitive から作れます。

```text
cargo run -p kivact-example-form
cargo run -p kivact-example-list
cargo run -p kivact-example-async
cargo run -p kivact-example-component
cargo run -p kivact-ui-basic-catalog
cargo run -p kivact-ui-pop-catalog
cargo run -p kivact-ui-game-catalog
cargo run -p kivact-ui-creative-catalog
```

[サンプル一覧](examples/README.md) からアプリの組合せも読めます。workspace の実行アプリは `--no-default-features` で CPU だけを build できます。通常は GPU を使い、利用できなければ CPU へ移ります。GPU を含む binary を実行時に CPU へ固定する場合は `ARVANE_RENDERER=cpu` を設定します。部品ライブラリは renderer を選びません。

公開 API の一覧は `cargo doc -p kivact -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative --no-deps` で生成する rustdoc を参照してください。[開発者ガイド](docs/for-kivact-developer/README.md) に設計・規則・検証、[CONTRIBUTING](CONTRIBUTING.md) に変更の手順があります。

初期開発版のため、API と構成は予告なく破壊的に変わります。互換 alias や以前の起動コマンドは提供しません。Linux の実 desktop・IME・screen reader と macOS の実画面には未検証の部分があります。GIF/WebP は最初の frame だけを表示し、実験的 primitive には制限があります。詳しい未解決事項は [設計](docs/for-kivact-developer/design.md) にあります。

| feature | 用途 |
|---|---|
| `gpu`（既定） | GPU 描画、利用できなければ CPU |
| `test` | 実 window の公開 Testing Library |
| `gpu-interop`（実験、既定 off） | 外部 GPU texture の共有 |
| `embed`（実験、既定 off） | Windows・Linux X11 のホスト window に App を開く。[wrapper の手順と制限](docs/for-kivact-user/how-to-guide/embed.md) |

## ライセンスと貢献

Copyright (c) 2026 Kashiwade。[MIT](LICENSE-MIT) または [Apache-2.0](LICENSE-APACHE) のどちらかを選んで利用できます。同梱する font・icon などの license は [THIRD-PARTY-NOTICES](THIRD-PARTY-NOTICES.md) にあります。

当面は作者だけが開発し、pull request は受け付けません。不具合の報告は issue へ、脆弱性は [SECURITY](SECURITY.md) の手順で連絡してください。
