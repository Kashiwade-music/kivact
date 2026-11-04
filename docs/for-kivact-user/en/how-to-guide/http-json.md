# Read JSON over HTTP

Enable `kivact`'s opt-in `tokio` feature. Add `reqwest` with `json` and `rustls`, and `serde_json`:

```toml
[dependencies]
kivact = { version = "0.1.0", features = ["tokio"] }
reqwest = { version = "0.13.5", default-features = false, features = ["rustls", "json"] }
serde_json = "1"
```

`resource` is Solid's `createResource`: changing `id` cancels the previous request. `async_view` shows loading, the result, or an error with Retry. `?` converts HTTP and JSON errors to `UiError`.

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

`on_tokio` runs the Send future on one runtime owned by the App, then wakes the UI. Read signals before entering it; move plain data into the future. Dropping the awaiting future aborts the task, including on source changes and owner disposal. App shutdown stops its runtime. Customize presentation with `.loading(|| text("Loading"))` and `.fallback(|error, retry| button(error.to_string()).on_press(move || retry.run()))`. An action works with the same `async_view`; its idle view is empty and Retry repeats its latest input.
