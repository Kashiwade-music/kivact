# 1. 最初のアプリ

この章では `kivact` だけを使うカウンターを起動し、自分で表示を変えます。Rust の環境を用意し、リポジトリのルートで `cargo --version` を確認してください。Rust の版は `rust-toolchain.toml` にあります。初回の依存取得とコンパイルには時間がかかります。

```text
cargo run -p kivact-example-counter
```

`Increment` を押し、数字が増えることを確認します。次に [main.rs](../../../examples/kivact/counter/src/main.rs) の表示を変更し、もう一度起動してください。必要な依存は `kivact` だけです。

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

`fn counter() -> impl IntoView` は画面を返す普通の Rust 関数です。`column((...))` は異なる型の二つの子を tuple にまとめ、縦に並べます。`signal(0)` は書き換えられる状態です。表示へ渡す `move || ...` は値を読み、変更時に表示を更新します。`move` は必要な値をクロージャへ取り込みます。signal のハンドルは `Copy` なので表示と操作の両方へ渡せます。

`button("増やす")` は `kivact::ui` の標準部品です。`kivact::prelude::*` から使えます。ボタンの文字は支援技術が読む名前にもなり、役割とキーボードの選択も標準で付いています。`on_press` はクリック、Enter、Space、支援技術の Activate 操作から呼ばれます。色や角丸などの見た目はテーマが決めます。`.variant(ButtonVariant::Secondary)` や `.style(...)` で変えられます。

`main` の `kivact::Result<()>` は起動の失敗を返します。`App::run` に渡す関数がルート画面を作ります。表示文を変更して再起動し、次に増分を `2` に変えて結果を確かめてください。

次は [部品とレイアウト](components-layout.md) です。自分の package を作る手順は [雛形](../../../templates/basic/README.md)、kit の部品は [公式 UI ガイド](../../for-kivact-ui-user/README.md) にあります。
