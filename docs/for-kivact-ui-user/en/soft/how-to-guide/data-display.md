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

## Chat and source blocks

Use `kivact::ui::{chat_thread, chat_message, chat_composer, typing_indicator, code_block}` with the Soft theme. `chat_thread(for_each(items, key, render))` replaces the message list and layout; its `empty`, `loading_top`, `streaming`, `gap`, `align`, `composer`, `viewport_ref`, `history_revision`, and `scroll_button` inputs retain those controls. Pass the actual dataset revision to `history_revision` when prepending history.

Set the message author with `chat_message(author)`, its participant with `sender`, and its body with `content`. The `avatar`, `metadata`, `time`, `status`, `footer`, `icon`, `group`, and `bubble_width` inputs replace the former bubble, metadata, and system wrappers. Use `ChatSender::System` for system messages. Compose a collapsible composer drawer with standard `disclosure` inside the `drawer` slot; standalone editor and send actions use `text_area` and `button`.

Soft `ChatDensity`, `BubbleVariant`, `SystemMessageVariant`, and `ComposerElevation` are standard `.variant(...)` values. `ChatAppearance` carries their reactive values together. Source `CodeBlockSize`, `CodeBlockWidth`, and `CodeBlockContainer` also use `.variant(...)`; `CodeAppearance` carries reactive values. All behavioral source options remain on `ui::code_block`, including copy feedback, line numbers, highlighted lines, diff, wrapping, height, controlled expansion, collapse threshold, and end content.

Language tokenizers remain in Soft. Pass a built-in `CodeLanguageInput::Named` or a custom `CodeLanguage` to `code_highlight` in the standard `highlight` line callback. Supply the current `Tokens.colors.syntax` or an explicit `SyntaxColors` as the callback's content source to preserve reactive palette overrides. `syntax_theme` still supplies that palette to nested content. Streaming text, tool calls, tokens, and application-owned dictation remain Soft domains.

## Tables

Build static tables with `kivact::ui::{table, table_header, table_body, table_footer, table_row, table_head, table_cell}`. The Soft theme styles their actual recipes. `TableCell::align(TextAlign)` replaces `TableAlign`; `width` and reactive `col_span` remain on the standard cell. Pass `TableRowAppearance::new(dividers, hover, striped)` or `TableCellAppearance::new(density, vertical_align, divider)` with `.variant(...)`; each input can be reactive. `ItemDensity` replaces the former `TableDensity` alias. Selected rows use `.with_node(|node| node.selected(selected))`.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn summary() -> impl IntoView {
    kivact::ui::table((
        kivact::ui::table_header((kivact::ui::table_row((
            kivact::ui::table_head(text("Project")),
            kivact::ui::table_head(text("Status")),
        )),)),
        kivact::ui::table_body((kivact::ui::table_row((
            kivact::ui::table_cell(text("Design")).variant(
                soft::TableCellAppearance::new(soft::ItemDensity::Compact,
                    kivact::style::Align::Center, true)),
            kivact::ui::table_cell(soft::table_row_status("Passed", soft::RowStatusTone::Success)),
        )).variant(soft::TableRowAppearance::new(true, true, false)),)),
    ))
}
```

`table_selection_toolbar(selection)` retains its `clear_label`, `selection_label` and `actions` inputs and derives visibility and count from the caller's selection binding. Its clear button and toolbar use the canonical controls. `table_row_status(label, RowStatusTone)` retains all four status colors on a canonical badge with the `RowStatusDot` appearance.

Use standard `pagination(page_count, page)` for a zero-based DataTable page binding; `.summary(...)` can display “Page n of m”. Replace `table_row_index(index)` with `text(move || index.get().to_string())` styled with `TextAlign::End` and the theme's secondary text color. Interactive tables continue to use `data_table` and `DataColumn`: sorting, selection, filters, column widths and order, sticky columns, paging, row status, tree/group rows, empty content, row actions, toolbar and footer slots keep their existing owners and APIs.
