# Soft を設定する

[English](../../../en/soft/getting-started/01-set-up-the-kit.md)

同じ Git リビジョンから `kivact` と `kivact-ui-soft` を依存関係に追加し、ウィンドウを開く前にテーマを設定します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(kivact_ui_soft::theme(ThemeMode::Light))
        .run(|| kivact_ui_soft::heading(1, "Settings"))
}
```
