# Display data: tables, charts, badges, avatars and code

[日本語](../../../jp/basic/how-to-guide/data-display.md)

This guide shows how to present information that the user reads rather than edits: a small static table, a sortable and filterable data table, a chart, status badges, avatars and a block of code. It assumes you have done the [tutorial](../getting-started/README.md).

The kit never owns your data. Collections come in as plain `Vec`s (or a signal of one, so the view follows changes), and what the user does (a copy request, a selection) comes back through a signal or a callback that you handle.

## Show a small static table

`kivact::ui::table` is for content that is laid out by hand. Build it from a header, a body and optionally a footer and caption:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn invoices() -> impl IntoView {
    kivact::ui::table((
        kivact::ui::table_header((kivact::ui::table_row((
            kivact::ui::table_head("Invoice").width(110.0),
            kivact::ui::table_head("Status"),
            kivact::ui::table_head("Amount").align(kivact::style::TextAlign::End),
        )),)),
        kivact::ui::table_body((
            kivact::ui::table_row((
                kivact::ui::table_cell("INV001").width(110.0),
                kivact::ui::table_cell("Paid"),
                kivact::ui::table_cell("$250.00").align(kivact::style::TextAlign::End),
            )),
            kivact::ui::table_row((
                kivact::ui::table_cell("INV002").width(110.0),
                kivact::ui::table_cell("Pending"),
                kivact::ui::table_cell("$150.00").align(kivact::style::TextAlign::End),
            )),
        )),
        kivact::ui::table_footer((kivact::ui::table_row((
            kivact::ui::table_cell("Total").width(110.0),
            kivact::ui::table_cell(""),
            kivact::ui::table_cell("$400.00").align(kivact::style::TextAlign::End),
        )),)),
        text("A list of recent invoices.").style(
            style()
                .margin_top(16)
                .color(basic::ColorName::MutedForeground)
                .text_align(kivact::style::TextAlign::Center),
        ),
    ))
    .style(style().width(460))
}
```

Give the same `.width(..)` to the head and cell of a column that should have a fixed width; the others share the rest. A row can be made selectable: `kivact::ui::table_row(..).selected(move || chosen.get() == index).on_press(move || chosen.set(index))`.

## Show many rows with sorting, filtering, selection and paging

For real data use the standard `data_table(rows, key, columns)`. The theme draws it in the Lyra look. Each `DataColumn` has a title and a function that makes the cell text; optional modifiers add sorting (`.compare(..)`), filtering (`.filter(..)`) and right alignment (`.numeric(true)`):

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[derive(Clone, PartialEq)]
struct Payment {
    id: u32,
    email: &'static str,
    amount: f64,
}

fn payments() -> impl IntoView {
    let data = vec![
        Payment {
            id: 1,
            email: "ken99@example.com",
            amount: 316.0,
        },
        Payment {
            id: 2,
            email: "abe45@example.com",
            amount: 242.0,
        },
        Payment {
            id: 3,
            email: "silas22@example.com",
            amount: 874.0,
        },
    ];
    let query = signal(String::new());
    let selected = signal(Vec::<u32>::new());
    let sort = signal(None);
    let page = signal(0_usize);
    let columns = vec![
        DataColumn::new("Email", |p: &Payment| p.email.to_string())
            .compare(|a, b| a.email.cmp(b.email))
            .filter(|p: &Payment, q: &str| p.email.contains(q)),
        DataColumn::new("Amount", |p: &Payment| format!("${:.2}", p.amount))
            .compare(|a, b| a.amount.total_cmp(&b.amount))
            .numeric(true),
    ];
    column((
        text_input("Filter emails", query).placeholder("Filter emails..."),
        data_table(data, |p: &Payment| p.id, columns)
            .row_label(|p: &Payment| p.email.to_string())
            .filter(query)
            .selection(selected)
            .sort(sort)
            .page(page, 5),
        basic::muted(move || format!("{} selected", selected.get().len())),
    ))
    .style(style().width(560).gap(8))
}
```

- The row type needs `Clone + PartialEq`, because the table compares rows to update only what changed. The key function (here `p.id`) gives every row a stable identity for selection and focus. Selection holds keys, not row numbers.
- `selection`, `sort`, `filter` and `page` are all optional signals you own. The table writes them when the user clicks a header, a check box or a page button, and you can write them too (to clear the selection, say). `sort` holds `Option<(column index, direction)>`.
- `.row_label(..)` gives each row's accessible name, which is what a screen reader says for its check box.
- `.toolbar(|context| view)` replaces the area above the table, for a filter box together with a "Columns" menu as in shadcn's data table. `.row_actions(width, |row| view)` adds a trailing column of controls (a "..." menu per row). The catalog's Data Table page has both worked out.
- The table's own texts (`Filter...`, `No results found.`, `Page 1 of 3`, ...) come from messages under `data_table.*` that you can replace; see [Theme and tokens](theme-and-tokens.md#choose-the-fonts).

## Draw a chart

`basic::chart(config, rows)` draws bar, stacked bar, line, area, pie, donut, radar and radial charts. `ChartConfig` names the series and each `ChartRow` has a label and one value per series:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn visitors() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop"),
        basic::ChartSeries::new("mobile", "Mobile"),
    ]);
    let data = vec![
        basic::ChartRow::new("Jan", [186.0, 80.0]),
        basic::ChartRow::new("Feb", [305.0, 200.0]),
        basic::ChartRow::new("Mar", [237.0, 120.0]),
    ];
    kivact::ui::card(
        basic::chart(config, data)
            .kind(basic::ChartKind::Line)
            .height(220.0),
    )
    .header(
        column((
            text("Bar chart").style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
            text("January - March").style(
                style()
                    .line_height(1.625)
                    .color(basic::ColorName::MutedForeground),
            ),
        ))
        .style(style().gap(4).grow(1).min_width(0)),
    )
    .style(style().width(560))
}
```

- `.kind(..)` is one of `Bar` (default), `StackedBar`, `Line`, `Area`, `Pie`, `Donut`, `Radar` and `Radial`. For a pie, donut or radial chart, each series is a slice or ring sized by the sum of its values over all rows. A radar chart has one axis per row.
- Series use the theme's five chart colors in order, so a theme switch recolors them. Set `ChartSeries::new(..).color(rgb(0x2563eb))` to pin one.
- `.orientation(Orientation::Horizontal)` lays bars out sideways. `.value_format(|v| format!("{v:.0}k"))` changes how values read in the axis and tooltips. `.legend(false)`, `.grid(false)`, `.axis(false)` and `.tooltip(false)` remove parts; `.height(..)` sets the height.
- Pass a signal of rows (`signal(Vec<ChartRow>)`) as `data` to update the chart live.
- Hovering shows a tooltip with the values. The chart also has an accessible summary, from the messages `basic.chart.*`.

## Mark status with badges and people with avatars

A badge is a small label. An avatar shows an image, or the fallback text when it has none:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn people() -> impl IntoView {
    column((
        row((
            kivact::ui::badge("Active"),
            kivact::ui::badge("Draft").variant(basic::BadgeVariant::Secondary),
            kivact::ui::badge("Failed").variant(basic::BadgeVariant::Destructive),
            kivact::ui::badge("Verified")
                .variant(basic::BadgeVariant::Outline)
                .icon(icon("badge-check").style(style().size(12, 12))),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        kivact::ui::avatar_group((
            kivact::ui::avatar("AL").fallback_content(|| text("AL")),
            kivact::ui::avatar("GH").fallback_content(|| text("GH")),
            kivact::ui::avatar("AT")
                .fallback_content(|| text("AT"))
                .size(kivact::ui::ControlSize::Large),
            kivact::ui::avatar("+3").fallback_content(|| text("+3")),
        )),
    ))
    .style(style().gap(12))
}
```

- Badge variants are `Default`, `Secondary`, `Destructive`, `Outline`, `Ghost` and `Link`. A badge becomes a button when you give it `.on_press(..)`.
- `kivact::ui::avatar(name).src(Some(asset))` reads reactive images. Use `.fallback_content(|| text("CN"))` for exact fallback text; otherwise the component derives initials from the name. Sizes are `ControlSize::Small`, `Medium` and `Large`. `.status(Some(Tone::Success))` draws a semantic corner dot; `.status_content(view)` accepts custom corner content.
- `kivact::ui::avatar_group(children)` overlaps its avatars. A final avatar with explicit fallback content can display a literal overflow count; `.max(limit)` manages a reactive count.

## Show code with a copy button

`basic::code_block(lines)` draws highlighted lines in a monospaced font. `basic::rust_lines(source)` highlights Rust source and `basic::code_lines(text)` shows plain text. The copy button does not touch the clipboard itself; it calls `.on_copy(..)` and you decide what happens:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn snippet() -> impl IntoView {
    let source = "fn main() {\n    println!(\"Hello, Kivact!\");\n}";
    basic::code_block(basic::rust_lines(source))
        .title("main.rs")
        .line_numbers(true)
        .on_copy(move || {
            spawn(kivact::platform::copy_text(source));
        })
        .style(style().width(pct(100)))
}
```

- `.max_height(..)` makes a long block scroll, `.size(basic::CodeBlockSize::Sm)` is the compact size, and `.footer(|| ..)` adds a line below the block.
- For another language, build the lines by hand: a `CodeLine` is a list of `CodeSpan::new(text, CodeTone::Keyword)` and so on (tones: `Plain`, `Keyword`, `String`, `Number`, `Comment`, `Function`, `Type`, `Macro`, `Lifetime`, `Punctuation`, `Attribute`). The colors are the `code` field of the theme's tokens.
- The code font is `Geist Mono`; register it as described in [Theme and tokens](theme-and-tokens.md#choose-the-fonts). Without it a system monospace font is used.
- After a successful copy the button reads "Copied" for a moment. The words come from `basic.code_block.*` messages.

## See also

- [Forms](forms.md): inputs, pickers and validation.
- [Theme and tokens](theme-and-tokens.md): chart and code colors follow the theme's tokens.
- Rustdoc: `cargo doc -p kivact-ui-basic --open` for the chart, table and code block types, and `cargo doc -p kivact-ui --open` for `data_table`.

`kivact::ui::item_list(children)` groups standard `item_row(title)` rows. Set `basic::ItemVariant` and `basic::ItemMediaVariant` with `.variant(..)`; `basic::ItemSize::Sm` keeps a 32 px image instead of 40 px. For reactive density use `.variant(basic::ItemLook::default().size(size))`. Media, actions, header and footer remain ordinary view slots.

Standard cards use `kivact::ui::card(content)` with ordinary views in `.header(..)`, `.media(..)` and `.footer(..)`. `basic::CardSize::Sm` is a visual variant. Use `.variant(basic::CardLook::default().size(size).header_bordered(bordered))` for reactive spacing and a header rule; the header title, description and action are caller-composed text and row/column views.

Headings use `kivact::ui::heading(text, level)` and the theme keeps the original Basic typography. Progress supports `ProgressVariant::Bar`, `Ring` and `Gauge` through `.variant(..)`; all three use the Basic primary and muted palette. For a fixed media ratio, put `.aspect_ratio(width / height)` on the caller-owned node style. Statistics and description lists use the standard `stat` and `description_list` components.
