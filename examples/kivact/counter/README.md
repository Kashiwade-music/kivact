# Counter

Increment one signal from a button and show a memo derived from it.

```sh
cargo run -p kivact-example-counter
```

`signal` の値を button で増やし、`memo` の `doubled` を別の text に表示します。`src/main.rs` を setup から view の順に読んでください。依存は `kivact` だけで、button は標準部品の `kivact::ui::button`（見た目はテーマが決めます）です。試験は `cargo test -p kivact-example-counter` です。
