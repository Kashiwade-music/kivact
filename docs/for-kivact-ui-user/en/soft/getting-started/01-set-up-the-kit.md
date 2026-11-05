# Set up Soft

[日本語](../../../jp/soft/getting-started/01-set-up-the-kit.md)

Add `kivact` and `kivact-ui-soft` from the same Git revision. Set the app theme before opening the window so standard controls and Soft components share the same palette.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_soft as soft;

fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(soft::theme(ThemeMode::Light))
        .run(|| {
            column((
                kivact::ui::heading("Settings", 1),
                soft::text("Manage your profile and appearance."),
            ))
            .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
        })
}
```

If you supply Figtree, register it before opening the window. See the theme and tokens guide for how to style your own surfaces.
