# Build the settings form

[日本語](../../../jp/soft/getting-started/02-build-the-settings-form.md)

Keep the display name in a signal. The standard input reads and updates that signal, and Soft supplies its appearance.

```rust,no_run
use kivact::prelude::*;
fn settings() -> impl IntoView {
    column((
        text_input("Display name", signal(String::new())),
        button("Save").variant(ButtonVariant::Primary),
    )).style(style().gap(12))
}
```
