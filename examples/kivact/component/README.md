# Component

Compose a reusable card with props, children and a close callback.

```sh
cargo run -p kivact-example-component
```

`card(title, on_close, children)` は、題名（`impl IntoText`）、閉じる通知（`impl Fn() + Copy + 'static`）、子（`impl Fn() -> V`）を受ける部品です。本体は `show` で折りたたみ、`app()` が別の子を渡して 2 回使います。"Close" は親の signal を更新して card を取り除きます。依存は `kivact` だけです。試験は `cargo test -p kivact-example-component` です。
