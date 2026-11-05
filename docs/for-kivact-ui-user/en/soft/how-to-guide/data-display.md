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

## Static rows and lists

Use the canonical `item_row(title)` and `item_list(children)`. Row content stays in `.title_content`, `.description_content`, `.media`, `.actions` and `.marker` slots. `.title_lines(Some(n))` and `.description_lines(Some(n))` limit actual plain text; explicit `None` is unlimited. Soft defaults to one line. `.disabled` and `.selected` use the standard node modifiers. `.on_press` activates the row; `.delegate_to(target)` forwards activation and focus to an actual child control without another tab stop.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn workspace() -> impl IntoView {
    let notifications=signal(false);
    let target=node_ref();
    kivact::ui::item_list((
        kivact::ui::item_row("Design").description("Active project"),
        kivact::ui::item_row("Notifications")
            .actions(kivact::ui::checkbox("Enable notifications",notifications).node_ref(target))
            .delegate_to(target),
    ))
    .header(text("Workspace"))
    .marker(kivact::ui::ItemListMarker::Decimal)
    .start(3u32)
    .variant(soft::ListLook::default().density(soft::ItemDensity::Compact)
        .dividers(true).edge(soft::ListEdge::Flush))
}
```

Use `ItemDensity`, `ItemAlign`, `ItemLayout` and `ListEdge` as constant `.variant(..)` values. `ItemLook` carries reactive density/alignment/layout/highlight inputs; `ListLook` carries reactive list density/dividers/edge inputs. All four edge choices remain available. List markers are standard `ItemListMarker::{None,Disc,Circle,Decimal}`; `.start(..)` sets decimal numbering. Standalone rows can override `.role(..)` when they are a button or a descriptive group.

For keyed reactive content, enumerate the dataset once, key by the item, and read each row's index from that same value. Fixed direct children receive their positions automatically; reactive rows use `.position(index, count)`. Existing keyed rows retain their identity and native `pos_in_set`/`set_size` semantics:

```rust,no_run
use kivact::prelude::*;

fn projects() -> impl IntoView {
    let items=signal(vec!["Design".to_string(),"Documentation".to_string()]);
    let rows=memo(move || items.get().into_iter().enumerate().collect::<Vec<_>>());
    let count=memo(move || rows.with(Vec::len));
    kivact::ui::item_list((for_each(rows,|(_,item)|item.clone(),move |row| {
        kivact::ui::item_row(move || row.get().1)
            .position(move || row.get().0,count)
    }),))
    .marker(kivact::ui::ItemListMarker::Decimal)
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
