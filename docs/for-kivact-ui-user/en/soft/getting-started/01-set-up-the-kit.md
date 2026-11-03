# Set up Soft

[日本語](../../../jp/soft/getting-started/01-set-up-the-kit.md)

Add `kivact` and `kivact-ui-soft` from the same Git revision. Set the app theme before opening the window.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(kivact_ui_soft::theme(ThemeMode::Light))
        .run(|| kivact_ui_soft::heading(1, "Settings"))
}
```
