# Kivact

[English](README.md)

Kivact は Rust のネイティブ GUI フレームワークです。コンポーネントは一度だけ実行される普通の関数で、細粒度のシグナルが、値を読んでいる画面の部分だけを更新します。React や SolidJS の経験がある開発者になじみのある考え方で、アプリの Rust のドメインロジックと同じコードベースに UI を置けます。

> **状態: 初期開発版。** API と crate の構成は予告なく変わり、リリースはまだありません。不具合の報告は issue で受け付けます。pull request は当面受け付けません。

```rust,no_run
use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("回数: {}", count.get())),
        button("増やす").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}
```

## 特徴

- **普通の関数とシグナル。** `signal`・`memo`・`effect` と key 付きの一覧。仮想 DOM はなく、コンポーネント全体を再実行しません。
- **標準コンポーネント。** button・入力欄・select・dialog・menu・tabs・表・toast などを `kivact::ui` に持ち、見た目は有効なテーマが決めます。
- **既定でアクセシブル。** どのコンポーネントも役割・名前・状態を OS の支援技術へ出し、キーボードで操作できます。
- **実ウィンドウのテスト。** `kivact::test` は `cargo test` から実際のウィンドウを開き、Testing Library のように役割と名前で要素を操作します。
- **文字と入力。** CJK・結合文字・双方向テキスト・IME の変換を扱います。
- **描画。** HTML Canvas に近い語彙の Canvas、アプリ自身の renderer が描いた frame の表示、ウィンドウなしの画像の書き出しを持ちます。
- **UI キット。** 4 つの公式キットが標準コンポーネントの見た目を差し替え、分野別のコンポーネントを足します。キットは公開 API だけで作られています。

## はじめに

Kivact はまだ crates.io に公開していません。Git リポジトリに依存します。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

ツールチェーンは [`rust-toolchain.toml`](rust-toolchain.toml) で固定しています。まず [チュートリアル](docs/for-kivact-user/jp/getting-started/README.md) から始めてください。

サンプルは、リポジトリを clone してルートで実行します。

```text
cargo run -p kivact-example-counter
cargo run -p kivact-example-form
cargo run -p kivact-example-list
cargo run -p kivact-example-async
cargo run -p kivact-example-component
cargo run -p kivact-ui-basic-catalog
```

## ドキュメント

| 対象 | 日本語 | English |
|---|---|---|
| アプリの開発者 | [Kivact ガイド](docs/for-kivact-user/jp/README.md) | [Kivact guide](docs/for-kivact-user/en/README.md) |
| UI キットの利用者 | [UI キット ガイド](docs/for-kivact-ui-user/jp/README.md) | [UI kit guide](docs/for-kivact-ui-user/en/README.md) |
| Kivact の開発者 | [製品戦略](docs/for-kivact-developer/strategy.md) | — |

API リファレンスは rustdoc です: `cargo doc -p kivact --open`

## Crate

| Crate | 用途 |
|---|---|
| `kivact` | フレームワーク本体。アプリとコンポーネントのライブラリはこの crate だけに依存します。 |
| `kivact-ui-basic` | 中立的な汎用キット（shadcn/ui の「Lyra」preset 風）。 |
| `kivact-ui-pop` | 落ち着いた配色の業務アプリ向けキット。 |
| `kivact-ui-game` | ゲーム風の操作パネルと HUD。 |
| `kivact-ui-creative` | 制作ツール向け: タイムライン、ノードグラフ、ドック、インスペクター。 |
| `kivact-gpu-share` | GPU texture 共有のエンジン側（実験的）。 |

[`crates/`](crates) にあるその他の crate は `kivact` の内部実装（反応の核、engine、レイアウト、テキスト、ネイティブホストの Arvane）で、直接使うものではありません。[`architecture/`](architecture) には生成した依存関係の図があります。

## `kivact` の Cargo feature

| Feature | 既定 | 説明 |
|---|---|---|
| `gpu` | 有効 | GPU で描画し、使えなければ CPU に切り替えます。無効にすると CPU だけで build します。`ARVANE_RENDERER=cpu` で実行時に CPU へ固定できます。 |
| `test` | 無効 | 実ウィンドウのテストライブラリ `kivact::test`。 |
| `embed` | 無効 | 実験的。ホストのウィンドウの中にアプリを開きます（オーディオプラグインの editor など）。 |
| `gpu-interop` | 無効 | 実験的。アプリ自身の renderer の GPU texture を、読み戻さずに表示します。 |

## 対応プラットフォーム

| プラットフォーム | 状況 |
|---|---|
| Windows | 主対象。 |
| Linux（Wayland、X11） | 主対象。デスクトップ環境・IME・スクリーンリーダーの確認は一部です。 |
| macOS | 未対応。 |
| Android、iOS | 設計では考慮していますが、未対応です。 |

## リポジトリの構成

| パス | 内容 |
|---|---|
| `crates/` | フレームワーク、ネイティブホスト、UI キット |
| `examples/` | 小さなアプリとキットのカタログ |
| `templates/basic/` | テスト付きの最小のアプリ |
| `docs/` | 利用者ガイドと開発者向けの製品戦略 |
| `dev/` | ワークスペースの検査と公開 API のテスト |
| `architecture/` | 生成した crate と module の依存関係の図 |

## 開発への参加

当面は作者だけが開発します。[CONTRIBUTING](CONTRIBUTING.md) を参照してください。脆弱性は [SECURITY](SECURITY.md) の手順で非公開に連絡してください。

## ライセンス

Copyright (c) 2026 Kashiwade。[MIT](LICENSE-MIT) または [Apache-2.0](LICENSE-APACHE) のいずれかを選んで利用できます。同梱するフォントとアイコンはそれぞれのライセンスに従います。[THIRD-PARTY-NOTICES](THIRD-PARTY-NOTICES.md) を参照してください。
