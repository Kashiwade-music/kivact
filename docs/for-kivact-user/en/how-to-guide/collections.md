# Show static collections

[日本語](../../jp/how-to-guide/collections.md)

Use `ui::table` for a static table and `ui::data_table` when cells need grid navigation, sorting, or filtering. A static table exposes table, row-group, row, column-header, and cell roles. It does not add keyboard navigation or selection behavior.

```no_run
use kivact::prelude::*;
use kivact::style::TextAlign;

fn invoices() -> impl IntoView {
    table((
        table_header((table_row((table_head("Invoice"), table_head("Amount"))),)),
        table_body((table_row((
            table_cell("INV001"),
            table_cell("$250.00").align(TextAlign::End),
        )),)),
        table_footer((table_row((table_cell("One invoice").col_span(2usize),)),)),
    )).name("Invoices")
}
```

Cells share a row's available width. Set the same `.width(..)` for a column in every row when it needs a fixed width. `.col_span(..)` takes a constant or reactive column count, publishes the cell's column span, and gives a flexible cell that many shares of the row. Counts below one become one. Give the table an accessible name with `.name(..)`; a caption can be ordinary text beside the table.

`ui::item_list` and `ui::item_row` expose list and list-item roles. They are suitable for content with leading media, a title, a description, and trailing actions. Use `ui::list` for a selectable listbox.

```no_run
use kivact::prelude::*;

fn documents() -> impl IntoView {
    item_list((
        item_row("Quarterly report")
            .description("Updated today")
            .media(text("PDF"))
            .actions(button("Download")),
        item_row("Meeting notes").on_press(|| println!("Open notes")),
    )).name("Documents")
}
```

`.on_press(..)` makes the item focusable and supports click, Enter, Space, and assistive activation while retaining its list-item role. `.header(..)` and `.footer(..)` add content above and below the main row. `.title_content(..)` replaces the drawn title while preserving its accessible name; `.description_content(..)` replaces the description view. The description is connected through `described_by`. Use `.marker(view)` for a leading marker and `.media(view)` for an image or other leading content. `.delegate_to(target)` routes row activation to that real control while retaining the row's list-item semantics. Use `.title_lines(..)` and `.description_lines(..)` to limit text, and `item_list(..).marker(..).start(..)` for shared numbering.

For a hierarchy, `ui::tree(label, items, selected).expanded(expanded)` uses shared tree navigation and publishes expansion only for branches. `.item(..)` supplies row content, `.header(..)` precedes the rows, and `.reveal(gap, transition)` uses retained presence for branch reveals. The selected keys and focused row are separate states. The application owns the expansion and selection bindings.

Install `UiTokens` to adjust `table`, `item_list`, and `tree` metrics. Override `UiRecipes::table`, `item_list`, or `tree` to style individual parts; each builder accepts kit variants with `.variant(..)`. Density, dividers, stripes, media frames, and typography belong to recipes and styles. Caller styles take precedence over default recipes.
