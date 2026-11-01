# Kivact のサンプル

まず [Kivact だけを使う最小アプリ](kivact/README.md) で状態・入力・一覧・非同期処理を学びます。各ディレクトリは独立した Cargo package で、1つのアプリだけを起動します。

装飾済みの部品を試すときは、利用する kit のカタログを選びます。

| kit | カタログ | 実行 |
|---|---|---|
| Basic | [shadcn/ui Lyra 風の文書型カタログ](kivact-ui-basic/README.md) | `cargo run -p kivact-ui-basic-catalog` |
| Pop | [落ち着いた配色の業務アプリ UI](kivact-ui-pop/README.md) | `cargo run -p kivact-ui-pop-catalog` |
| Game | [ゲーム UI](kivact-ui-game/README.md) | `cargo run -p kivact-ui-game-catalog` |
| Creative | [制作ツール UI](kivact-ui-creative/README.md) | `cargo run -p kivact-ui-creative-catalog` |

学習の順番は [チュートリアル](../docs/for-kivact-user/jp/getting-started/README.md)、キットの導入は [UI キット ガイド](../docs/for-kivact-ui-user/jp/README.md) を参照してください。
