# List

Keep a keyed todo list with checkboxes, removal and a remaining count.

```sh
cargo run -p kivact-example-list
```

text input と "Add" で `{ id, title, done }` を追加し、行ごとに `kivact::ui::checkbox` と "Remove" を置きます。`for_each` が `id` を key に行を保ち、末尾の "N left" は `memo` で導きます。行の checkbox は `bind!(todo.done)` で、行の `Signal<Todo>` の `done` field への双方向の `Signal<bool>` を作って `checkbox` に渡します。checkbox を押すと一覧の `done` が書き換わります。依存は `kivact` だけです。試験は `cargo test -p kivact-example-list` です。
