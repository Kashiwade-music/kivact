# Creative で編集パネルを作る

編集ツールのパネルと操作部品を作るキットです。 この章では入力した名前を確定する画面を作ります。まずリポジトリのルートでカタログを起動し、入力、ボタン、テーマの見た目を確認してください。

```text
cargo run -p kivact-ui-creative-catalog
```

次に [雛形](../../../../templates/basic/README.md) に従ってアプリを作り、`kivact-ui-creative` の path 依存を追加します。この crate の path は checkout 内の `crates/kivact-ui-creative` です。依存は `default-features = false` で指定し、GPU はアプリ側の `kivact/gpu` feature で選びます。`src/main.rs` を次に置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn panel() -> impl IntoView {
    let name = signal(String::new());
    let saved = signal(String::new());
    column((
        kivact::ui::text_input("名前", name),
        kivact::ui::button("確定").on_press(move || saved.set(name.get())),
        text(move || format!("確定した名前: {}", saved.get())),
    )).style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Creative で編集パネルを作る").theme(ui::theme(kivact::style::ThemeMode::Dark)).run(panel)
}
```

自分の package で `cargo run` を実行し、名前を入力して「確定」を押します。表示が入力した名前になることを確認してください。入力するだけでは確定値は変わりません。`text_input("名前", name)` は編集中の状態、`on_press` は確定する操作をつなぎます。

次に「確定」のラベルを「保存」に変え、もう一度起動してください。Tab で入力欄とボタンへ移り、Enter/Space でボタンを操作します。入力名と操作名は支援技術にも伝わるため、用途の分かる文字を選びます。

公開 API の一覧は `cargo doc -p kivact-ui-creative --no-deps` で生成してください。

編集するデータが増えたら、用途ごとの部品に項目列を読取専用の `IntoValue<Vec<T>>` として渡します。`timeline` は時間単位の `Clip`、`dope_sheet` は `Keyframe`、`graph_editor` は時間と値の `Curve`、`node_editor` は `NodeData` と `NodeLink` の変更を要求します。lane と item は一覧の index と別の安定した ID を持ちます。値の保存と undo の履歴はアプリが所有します。`on_edit` の `EditorEvent` で Begin・絶対値の Request・Commit/Cancel を受け、要求を反映または拒否します。Begin で項目を保存し、Cancel で復元します。選択・view の状態は editor ごとの `…State`（`TimeState` など）の binding にまとめて構築関数に渡し、単一の編集値も構築関数の引数の binding で渡します。具体例は [Creative の手順](../../how-to-guide/creative.md)を参照してください。

アプリが描くプレビューや scope は `viewport_surface` で `Pixels` を表示します。Kivact は絵の表示と拡大・pan・入力を受け持ちます。色・Bezier・gradient、見出しのセル、検索の複数列までの具体例は [Creative の手順](../../how-to-guide/creative.md)へ進んでください。
