# Pop でフォームを作る

標準部品 `kivact::ui` にフォームの外観を設定するキットです。 この章では入力した名前を確定する画面を作ります。まずリポジトリのルートでカタログを起動し、入力、ボタン、テーマの見た目を確認してください。

```text
cargo run -p kivact-ui-pop-catalog
```

次に [雛形](../../../../templates/basic/README.md) に従ってアプリを作り、`kivact-ui-pop` の path 依存を追加します。この crate の path は checkout 内の `crates/kivact-ui-pop` です。依存は `default-features = false` で指定し、GPU はアプリ側の `kivact/gpu` feature で選びます。`src/main.rs` を次に置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as ui;

fn panel() -> impl IntoView {
    let name = signal(String::new());
    let saved = signal(String::new());
    column((
        kivact::ui::text_input("名前",name),
        kivact::ui::button("確定").on_press(move || saved.set(name.get())),
        text(move || format!("確定した名前: {}", saved.get())),
    )).style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Pop でフォームを作る").theme(ui::theme(kivact::style::ThemeMode::Light)).run(panel)
}
```

自分の package で `cargo run` を実行し、名前を入力して「確定」を押します。表示が入力した名前になることを確認してください。入力するだけでは確定値は変わりません。`text_input("名前", name)` の `name` は編集中の状態、`on_press` は確定する操作をつなぎます。

次に「確定」のラベルを「保存」に変え、もう一度起動してください。Tab で入力欄とボタンへ移り、Enter/Space でボタンを操作します。入力名と操作名は支援技術にも伝わるため、用途の分かる文字を選びます。

公開 API の一覧は `cargo doc -p kivact-ui-pop --no-deps` で生成してください。

項目列を扱う標準の `tag_input` と table は、アプリの一覧を読取専用で受けます。タグの追加・削除の要求をアプリが反映し、選択などの view 状態は binding で渡します。具体例は [Pop の手順](../../how-to-guide/pop.md)を参照してください。
