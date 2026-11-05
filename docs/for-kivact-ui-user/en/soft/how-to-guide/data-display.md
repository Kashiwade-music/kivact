# Data display

[日本語](../../../jp/soft/how-to-guide/data-display.md)

Use a metadata list for named attributes of one record. Give the section a heading and let each value use the view that matches its data.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn build_details() -> impl IntoView {
    column((
        soft::heading(2, "Build details"),
        soft::metadata_list(vec![
            soft::metadata_item("Status", soft::text("Passed")),
            soft::metadata_item("Revision", soft::code("ae72f04")),
        ]),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
}
```

Use a standard data table when comparing multiple records by column or when users need sorting. Keep the original values available to the table so the display formatting does not determine numeric or chronological order.

Keyboard shortcuts use standard `kbd("mod+shift+p")` or typed `kbd_combo(keys).modifiers(modifiers)`. Soft styles each cap through `kbd_key` and the row through `kbd`; no kit-specific parser is required. `mod` follows the platform command key. Key glyphs, arrow aliases, Escape, Return, and a literal `plus` cap retain localized full spoken names; customize `kbd.key.*` and `kbd.separator` in the application message catalog.
