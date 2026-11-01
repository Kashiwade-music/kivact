# 6. 見た目とアクセシビリティ

`.style(style()...)` で余白、大きさ、色などを指定します。レイアウトや部品の初期スタイルの上に、後から指定したスタイルが重なります。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let name = signal(String::new());
    column((
        text("設定"),
        text_input("表示名", name).placeholder("名前を入力"),
        button("保存").description("入力した表示名を保存します"),
    ))
    .style(
        style()
            .gap(12)
            .padding(16)
            .background(ColorToken::Surface)
            .color(ColorToken::Text),
    )
}

fn main() -> kivact::Result<()> {
    App::new("見た目の例").theme(Theme::dark()).run(app)
}
```

`gap(12)` は子の間隔、`padding(16)` は内側の余白です。長さの数値は論理ピクセルです。`ColorToken::Surface` や `ColorToken::Text` はテーマの色を参照するので、`Theme::dark()` を選んでも配色がつながります。固定色が必要なら `rgb(0x336699)` も使えます。

`App::theme` はアプリ起動時のテーマです。標準の `button("保存")` はテーマの色と大きさで描かれ、色の役割（`ColorToken::Accent` など）や `focus_ring`、`control_height` を変えたテーマを選ぶと、すべての標準部品の見た目が変わります。`button("保存")` と `text_input("表示名", name)` の文字列は、見た目だけでなく支援技術が使う名前にもなります。補足が必要なら `.description(...)` を付けます。キーボード操作やテストから見つけやすいよう、操作部品には意味の分かる名前を付けてください。

スタイルでは `width` / `height` / `size`、`margin`、`border`、`radius` なども指定できます。`border` は CSS の `box-sizing: border-box` と同じく、`padding` と並んで箱の内側に場所を取り、`width` / `height` は枠を含む大きさです。`hover(style()...)` や `focus(style()...)` で状態ごとの見た目も設定できます。状態の style で枠の幅を変えても中身は動きません（状態の style は描画だけを変えます）。

文字の書体は `style().font_family("monospace")` のように指定し、子孫へ継がれます。文字の色・大きさ・太さなどは CSS と同じく property ごとに継がれるので、`weight(600)` だけを指定した子も親の `color` を保ちます。画像は `image(include_bytes!("../assets/logo.svg")).name("ロゴ")` で表示し、`.name(...)` が支援技術の読む代替テキストになります（名前のない画像は装飾として扱われます）。

次は [非同期処理](async.md) で、待ち時間のある仕事を UI とつなぎます。


## 自分で動かして確認する

入門 package の `src/main.rs` を上のコードに置き換え、`cargo run -p kivact-example-counter` で起動します。`gap(12)` を `gap(24)` に変えて再起動し、間隔を確認します。Tab で「保存」へ移れることと、表示名の入力欄が名前付きであることも確認してください。
