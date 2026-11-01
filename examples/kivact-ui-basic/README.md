# kivact-ui-basic catalog

[kivact-ui-basic](../../crates/kivact-ui-basic/README.md)（shadcn/ui の Lyra preset 風の公式部品）を、ドキュメントの形で見せるカタログです。上のバーにテーマの切替、左に検索つきの部品一覧、右に部品ごとのページがあります。ページには説明、使い方、例ごとの実行結果とそのコードが並びます。

リポジトリの root から実行します。

```sh
cargo run -p kivact-ui-basic-catalog
cargo run -p kivact-ui-basic-catalog --no-default-features
```

既定では GPU を使い、`--no-default-features` で CPU だけを build できます。

## コードは実行しているもの

各ページは `src/docs/<slug>.rs` の 1 ファイルです。例は `// example: <id>` と `// end example` の行で囲んだ関数で、ページは `include_str!` で自分自身のソースを読み、囲んだ範囲を（字下げを外して）コードとして表示します。画面のコードと実行結果が食い違うことはありません。コードは `kivact-ui-basic` の `code_block` で表示し、色分けは同じ kit の `rust_lines`（小さな Rust の字句解析）が行います。コードは読み取り専用の text box なので、一部を pointer や Shift+矢印で選び Ctrl+C でコピーできます。全体のコピーのボタンは `kivact::platform::copy_text` でクリップボードへ書き込みます。

`Page` と `Example` の型は `src/model.rs`、ページの一覧（`PAGES`）は `src/docs.rs` にあります。ページを足す手順は次のとおりです。

1. `src/docs/` の既存のページ（例えば `button.rs`）を写して、`page()` の `slug`、`name`、`description`、`usage`、`source` を直す。
2. 例ごとに `Example::new(id, title, description, default)` を `examples` に足し、`// example: <id>` と `// end example` の間に `fn default() -> impl IntoView` の形の関数を書く（`View` に変えなくてよい）。`id` はページの中で重複させない。
3. 部品の一覧に載せるときは、`src/docs.rs` の `mod` と `PAGES`（各ページの `page()` を並べる）に追加する（未実装の部品には `準備中` を表示するだけのページがあるので、そのファイルを書き換える）。

## フォント

フォントはこのアプリのものです。`assets/fonts/` に Inter、Geist Mono、Noto Sans JP を置いてあり、`SOURCE.md`、`OFL.txt`、`SHA256SUMS.txt` が出所、ライセンス、バイト列を記録しています。

## 試験

`tests/catalog.rs` は実 window で、一覧のすべての項目を開いてタイトル、エラー、レイアウト、例の実行結果とコードの存在を確かめます。ほかにテーマの切替（開いたページと入力した値が残ること）、検索による絞り込み、キーボードでの移動、コピーのボタンを確かめます。window を開くので、`dev/tools/window-lock.sh` を通して実行します。

```sh
dev/tools/window-lock.sh -- cargo test -p kivact-ui-basic-catalog
```

`KIVACT_CATALOG_EVIDENCE=<ディレクトリ>` を設定すると、ページごとの画面を PNG で保存します。
