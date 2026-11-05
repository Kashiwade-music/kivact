# ステップ 1: キットを導入する

[English](../../../en/pop/getting-started/01-set-up-the-kit.md)

このステップでは、プロジェクトに Pop キットを追加し、カード、見出し、テキスト欄、ボタンを Pop の見た目で表示するウィンドウを開きます。

## プロジェクトを作る

```text
cargo new new-account
cd new-account
```

Kivact とキットはまだ crates.io にありません。`Cargo.toml` に Git 依存として追加します。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
kivact-ui-pop = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

`dev-dependencies` の行は、ステップ 3 で使うテスト用の機能を有効にします。

## キットとは何か

キットは**テーマ**です。標準コンポーネント（`text_input`、`button`、`radio_group`、`tabs` など）は `kivact` クレートが提供し、テーマが描き方を決めるまでは素朴な見た目です。`kivact-ui-pop` はそのテーマと、カード `panel`、日付入力、バッジ、アラートといった固有のコンポーネントを提供します。コアのチュートリアルと同じ `button(..)` や `text_input(..)` を書き、キットが見た目だけを変えます。

## テーマを入れる

`src/main.rs` を次の内容に置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn sign_up() -> impl IntoView {
    let name = signal(String::new());
    kivact::ui::card((
        text("新規アカウント").style(style().font_size(20).weight(600)),
        text_input("名前", name),
        button("アカウントを作成"),
    ))
    .style(style().width(360).margin(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("New account")
        .theme(pop::theme(ThemeMode::Light))
        .run(sign_up)
}
```

実行します。

```text
cargo run
```

明るい背景の上に、細い枠線と角丸の白いカード、見出し、角丸のテキスト欄、青いボタンが表示されます。欄をクリックして入力すると、青いフォーカスリングが出ます。

新しく出てきたものは 3 つです。

- `pop::theme(ThemeMode::Light)` が Pop テーマを作り、`App::theme(..)` がウィンドウ全体に適用します。結果は普通の Kivact の `Theme` です。ダークの配色が欲しいときは `ThemeMode::Dark` を使います。
- `kivact::ui::card(..)` は Pop のコンポーネントで、枠線と角丸を持つカードです。子要素を縦に並べます。
- `use kivact_ui_pop as pop;` は短い別名にすぎません。キットのコンポーネントは `pop::名前`、標準コンポーネントは修飾なしで書きます。

状態はこれまでどおり `signal` に持ちます。キットがデータを持つことはありません。`name` はあなたのもので、`text_input` はそれを読み書きするだけです。

## フォント

Pop は Kivact が同梱するフォントでそのまま動くので、このチュートリアルでは何も足す必要はありません。キット付属のカタログは、元のデザインに合わせて Inter、Barlow、Noto Sans JP を `App::font` で登録しています。フォントはアプリが持つもので、同じことをしたい場合は [Pop のテーマとトークンを設定する](../how-to-guide/theme-and-tokens.md) を参照してください。

次へ: [ステップ 2: フォームを組み立てる](02-build-the-form.md)
