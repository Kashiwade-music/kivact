# HTTP で JSON を読む

[English](../../en/how-to-guide/http-json.md)

`kivact` の opt-in feature `tokio` を有効にし、`reqwest` の `json` と `rustls`、`serde_json` を依存に追加します。

```toml
[dependencies]
kivact = { version = "0.1.0", features = ["tokio"] }
reqwest = { version = "0.13.5", default-features = false, features = ["rustls", "json"] }
serde_json = "1"
```

`resource` は Solid の `createResource` に相当します。`id` が変わると前の要求をキャンセルします。`async_view` は読み込み中、結果、または Retry ボタン付きのエラーを表示します。`?` は HTTP と JSON のエラーを `UiError` に変換します。

```no_run
use kivact::prelude::*;
use kivact::task::on_tokio;

fn user_page() -> impl IntoView {
    let id = signal(1_u32);
    let user = resource(id, |id| async move {
        on_tokio(async move {
            let user = reqwest::get(format!("https://jsonplaceholder.typicode.com/users/{id}"))
                .await?
                .error_for_status()?
                .json::<serde_json::Value>()
                .await?;
            Ok::<_, UiError>(user)
        }).await
    });
    column((
        button("Next user").on_press(move || id.update(|id| *id += 1)),
        async_view(user, |user| text(user["name"].as_str().unwrap_or("Unknown").to_owned())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Users").run(user_page)
}
```

`on_tokio` は App が持つ一つの runtime で Send future を動かし、完了時に UI を起こします。signal は呼び出し前に読み、普通のデータを渡してください。待機する future が drop されると task を abort します。source の変更、owner の破棄でも同じです。App の終了時には runtime も停止します。`.loading(|| text("Loading"))` と `.fallback(|error, retry| button(error.to_string()).on_press(move || retry.run()))` で表示を変更できます。action も同じ `async_view` に渡せます。実行前は空で、Retry は最後の入力で再実行します。
