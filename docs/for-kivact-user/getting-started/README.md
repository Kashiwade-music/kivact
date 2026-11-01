# Kivact チュートリアル

Rust の `fn`、tuple、`String`、クロージャを学び始めた人が、小さな画面を順に作るための入門です。必要な crate は `kivact` だけです。engine や Arvane の内部構成を覚える必要はありません。

| 順番 | 記事 | 完成する画面 |
|---|---|---|
| 1 | [最初のアプリ](quickstart.md) | 押すと数が増えるカウンター |
| 2 | [部品とレイアウト](components-layout.md) | 関数に分けた縦横の配置 |
| 3 | [状態と反応性](state.md) | 元の数と計算結果が変わる表示 |
| 4 | [入力とイベント](input-events.md) | 入力した名前を確定する画面 |
| 5 | [条件分岐と一覧](conditional-lists.md) | 表示を切り替え、行を並べ替える画面 |
| 6 | [見た目とアクセシビリティ](styling-accessibility.md) | 色と意味のある操作名を付けた画面 |
| 7 | [非同期処理](async.md) | 処理中、成功、失敗の表示 |
| 8 | [画面のテスト](testing.md) | 操作後の表示を確かめる試験 |

React・SolidJS を使っていた方は、[React・SolidJSからKivactへ](react-solid.md)で概念の対応とRustでの違いを確認できます。

まず最初のアプリを起動してください。コード全体を置き換える章では、カウンター package の `src/main.rs` を使えます。各段階の独立した実行 package もあります。

```text
cargo run -p kivact-example-counter
cargo run -p kivact-example-form
cargo run -p kivact-example-list
cargo run -p kivact-example-async
cargo run -p kivact-example-component
```

入門では、ボタンや入力欄などの標準部品（`kivact::ui`。見た目はテーマが決めます）と、配置や属性の土台になる `node` を使います。信号のハンドルは `Copy`、所有する `String` などを `get()` すると値を複製します。図形の描画や独自入力を実装する前に、名前、役割、focus と操作を付けて動かしてください。

外観の付いた部品は [公式 UI ガイド](../../for-kivact-ui-user/README.md) にあります。完全な API 一覧は `cargo doc -p kivact --no-deps` で生成する rustdoc です。

入門の後は [目的別ガイド](../how-to-guide/README.md)へ進んでください。独自の外観を持つ部品は、[部品作者向けガイド](../../for-component-author/README.md)の interaction と primitive を使って作れます。文字入力も通常の node に `use_text_edit` の Part を付ける構成です。

このプロジェクトは初期開発版です。API は予告なく破壊的に変わります。
