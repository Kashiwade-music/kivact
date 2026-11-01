# Form

Validate a sign-up form with memos and show messages with show_some.

```sh
cargo run -p kivact-example-form
```

`email` と `password` を `kivact::ui::text_input` に結び、欄ごとの検証メッセージを `memo` で導いて `show_some` で出し入れします。両方が有効になるまで送信 button は無効で、押すと "Signed up as <email>" を表示します。依存は `kivact` だけです。試験は `cargo test -p kivact-example-form` です。
