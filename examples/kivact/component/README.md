# Component

Compose a reusable card with props, children and a close callback.

```sh
cargo run -p kivact-example-component
```

`#[component]` を付けた `card(title, content)` は、題名（`Value<String>`）と、開くたびに作り直す子（`ViewFn`）を位置引数で受け、任意の `.on_close(..)`（`Callback<()>`）を setter で受けます。戻り値は `impl IntoView` です。本体は `show(open, content)` で折りたたみ、`app()` が別の子を渡して 2 回使います。"Close" は親の signal を更新して card を取り除きます。依存は `kivact` だけです。試験は `cargo test -p kivact-example-component` です。
