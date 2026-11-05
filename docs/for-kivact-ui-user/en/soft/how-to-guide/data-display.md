# Data display

[日本語](../../../jp/soft/how-to-guide/data-display.md)

Use a metadata list for named attributes of one record. Give the section a heading and let each value use the view that matches its data.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn build_details() -> impl IntoView {
    column((
        kivact::ui::heading("Build details", 2),
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

Use standard `badge(label)` with `.tone(Tone::Info/Success/Warning/Danger)` for status colors. Categorical colors remain `.variant(soft::BadgeVariant::Blue)` and the other Neutral hue variants. `.icon(view)` and `.trailing_icon(view)` accept custom visuals; size an icon to 12 px for the original cap geometry. A badge can use `.on_press(...)` when it is an action.

Headings use `kivact::ui::heading(value, level)`. Apply `HeadingDisplay::D1`, `D2` or `D3` with `.variant(..)` to keep the semantic level while changing display typography. Generic node styles supply color, weight, line clamping, wrapping, alignment, capsize and strikethrough. A truncation tooltip composes `ui::tooltip(value).enabled(anchor.text_truncated()).attach(heading.node_ref(anchor))`; the caller owns its anchor. Progress keeps its Bar motion and tone options and also accepts `ProgressVariant::Ring` and `Gauge`.
