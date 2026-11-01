# List

Keep a keyed todo list with checkboxes, removal and a remaining count.

```sh
cargo run -p kivact-example-list
```

text input と "Add" で `{ id, title, done }` を追加し、行ごとに `kivact::ui::checkbox` と "Remove" を置きます。`for_each` が `id` を key に行を保ち、末尾の "N left" は `memo` で導きます。行の checkbox は `binding` で一覧の signal に結びます。依存は `kivact` だけです。試験は `cargo test -p kivact-example-list` です。
