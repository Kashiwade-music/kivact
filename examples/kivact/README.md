# Kivact の最小アプリ

直接依存は `kivact` だけです。リポジトリのルートから、学びたい操作に対応するアプリを1つ起動します。どれも React・SolidJS の開発者が最初に書く基本のプログラムです。

| アプリ | 学ぶこと | 実行 |
|---|---|---|
| [counter](counter/README.md) | `signal` と `memo`、標準の button | `cargo run -p kivact-example-counter` |
| [form](form/README.md) | text input、`memo` による検証、`show_some` | `cargo run -p kivact-example-form` |
| [list](list/README.md) | key つきの一覧、checkbox、追加と削除 | `cargo run -p kivact-example-list` |
| [async](async/README.md) | `resource` と `dynamic` による読込中・エラー・完了 | `cargo run -p kivact-example-async` |
| [component](component/README.md) | props、children、callback を持つ部品 | `cargo run -p kivact-example-component` |

各アプリの摩擦の計測は [FRICTION.md](FRICTION.md) にあります。

Rust 初学者は [入門チュートリアル](../../docs/for-kivact-user/getting-started/README.md) に沿って、実行結果を確かめながらコードを変えてください。小さなアプリを作り始める雛形は [templates/basic](../../templates/basic/README.md) にあります。
