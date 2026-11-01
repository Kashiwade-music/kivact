# Async

Load a record by id with a resource and render loading, error and ready.

```sh
cargo run -p kivact-example-async
```

数値の id を source にした `resource` が、`sleep` を通信の代わりに待って読み込みます。"Previous" と "Next" で id を変えると再取得され、id 0 は `UiError::msg("no user 0")` で失敗します。`dynamic` が `AsyncState` に応じて読込中、エラー、完了を描き分けます。依存は `kivact` だけです。試験は `cargo test -p kivact-example-async` です。
