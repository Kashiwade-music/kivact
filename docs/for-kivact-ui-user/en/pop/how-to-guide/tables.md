# Show rows in a table

[日本語](../../../jp/pop/how-to-guide/tables.md)

Use this guide to show a list of records as a table with search, sorting,
selection and paging, in the Pop look. The table is the standard
`kivact::ui::data_table`; the Pop theme draws it as a rounded surface with 40
px headers, 52 px rows and 160 px columns that scroll sideways when they do
not fit. The kit adds no table type of its own.

## Show the rows

Your rows stay in your signal. The table needs the rows, a function that
returns each row's unique key, and the columns.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

#[derive(Clone, PartialEq)]
struct Order {
    id: u32,
    customer: &'static str,
    status: &'static str,
    total: f64,
}

fn orders() -> impl IntoView {
    let rows = signal(vec![
        Order { id: 1, customer: "Haruka Sato", status: "Paid", total: 120.0 },
        Order { id: 2, customer: "Ren Mori", status: "Pending", total: 48.5 },
        Order { id: 3, customer: "Aoi Ito", status: "Paid", total: 310.0 },
    ]);
    data_table(
        rows,
        |order: &Order| order.id,
        vec![
            DataColumn::new("Customer", |order: &Order| order.customer.to_owned()).sortable(),
            DataColumn::new("Status", |order: &Order| order.status.to_owned()).cell(
                |order: &Order| {
                    let hue = if order.status == "Paid" {
                        pop::Hue::Mint
                    } else {
                        pop::Hue::Yellow
                    };
                    pop::badge(order.status).hue(hue)
                },
            ),
            DataColumn::new("Total", |order: &Order| format!("${:.2}", order.total))
                .compare(|a, b| a.total.total_cmp(&b.total))
                .numeric(true),
        ],
    )
    .style(style().margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Orders")
        .theme(pop::theme(ThemeMode::Light))
        .run(orders)
}
```

Run it and click a header: rows sort ascending, then descending. A header
sorts when its column has `.sortable()` (by its text, numerically when both
cells are numbers) or `.compare(..)` (by your own order). A column with
neither keeps the order of your signal.

**Rust note.** A row type needs `Clone + PartialEq`, and the key type needs
`Clone + Eq + Hash`; a `u32` id or a `String` slug works. `|order: &Order|`
spells out the row type so the closures infer.

`.cell(|row| view)` replaces the cell's content, here with a badge; the
column's text still names the cell for screen readers and for filtering.

## Add search, selection and paging

Each extra is turned on by giving the table the state it should keep. That
state is yours, so you can read or set it from anywhere.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::SortOrder;
use kivact_ui_pop as pop;

#[derive(Clone, PartialEq)]
struct Order {
    id: u32,
    customer: String,
    total: f64,
}

fn orders(rows: Signal<Vec<Order>>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    let query = signal(String::new());
    let page = signal(0_usize);
    let sort = signal(Some((1_usize, SortOrder::Descending)));
    column((
        data_table(
            rows,
            |order: &Order| order.id,
            vec![
                DataColumn::new("Customer", |order: &Order| order.customer.clone())
                    .sortable()
                    .filter(|order, query| {
                        order.customer.to_lowercase().contains(&query.to_lowercase())
                    }),
                DataColumn::new("Total", |order: &Order| format!("${:.2}", order.total))
                    .compare(|a, b| a.total.total_cmp(&b.total))
                    .numeric(true),
            ],
        )
        .selection(selected)
        .filter(query)
        .page(page, 10)
        .sort(sort)
        .row_label(|order| format!("Order {}", order.id)),
        text(move || format!("{} selected", selected.with(Vec::len))),
    ))
    .style(style().gap(12).margin(24))
}
# fn main() {}
```

- `.selection(signal)` adds a check box to every row and a "select this page"
  box in the header, and keeps the keys of the checked rows. Checking a page
  does not touch selected rows on other pages. A summary ("2 of 3 row(s)
  selected.") appears under the table.
- `.filter(signal)` shows a filter field above the table. A row stays when
  the text matches through any column's `.filter(|row, query| bool)`. A column
  without `.filter` is not searched, and the key is never searched.
- `.page(signal, size)` shows `size` rows per page with Previous and Next.
  The page number starts at 0. A new sort or filter text goes back to the
  first page, and a page number past the end shows the last page without
  rewriting your signal.
- `.sort(signal)` is optional: use it to start sorted or to read the sort. It
  holds `Some((column index, direction))`; the index counts columns from 0.
- `.row_label(..)` names each row and its check box for screen readers.
  Without it the first column's text is used.

## Put buttons next to the search

`.toolbar(|context| view)` replaces the area above the table, which normally
holds only the filter field. The `context` has the table's live state: the
selection and the filter as bindings, and `context.model` with the counts.
You must then draw the filter field yourself if you still want one:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

#[derive(Clone, PartialEq)]
struct Order {
    id: u32,
    customer: String,
}

fn orders(rows: Signal<Vec<Order>>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    let query = signal(String::new());
    data_table(
        rows,
        |order: &Order| order.id,
        vec![DataColumn::new("Customer", |order: &Order| order.customer.clone()).sortable()],
    )
    .selection(selected)
    .filter(query)
    .toolbar(move |_context| {
        row((
            text_input("Filter orders", query)
                .placeholder("Search customers")
                .style(style().grow(1)),
            button("Delete selected")
                .variant(pop::ButtonTone::Soft)
                .disabled(move || selected.with(Vec::is_empty))
                .on_press(move || {
                    let doomed = selected.get();
                    rows.update(|rows| rows.retain(|order| !doomed.contains(&order.id)));
                    selected.set(Vec::new());
                }),
        ))
        .style(style().gap(8))
    })
}
# fn main() {}
```

Deleting is your code: the table never changes the rows. `.footer(|context|
view)` replaces the area below the table (the selection summary and the pager)
in the same way.

## More per-row controls

- `.row_actions(width, |row| view)` adds a last column of controls such as a
  menu button. The header has no title and is named "Actions".
- `.resizable(signal_of_widths)` adds resize handles to the headers and keeps
  the widths (a `Vec<f64>`, one per column) in your signal.
- `.row_details(|row| view)` puts a view under each row, and
  `.cell_controls(..)` puts controls inside cells; both stay mounted while
  rows are sorted and paged, so their state (a half-typed edit) is kept.
- `.row_color(|row| Option<Color>)` tints a row, for example an overdue one.

## Texts

The table's own texts (the filter placeholder "Filter…", "No results found.",
"Previous", "Next", "Page {page} of {pages}") are English. Replace them with
`App::messages`; the keys are `data_table.filter`, `data_table.empty`,
`data_table.previous`, `data_table.next`, `data_table.page` and so on. They are
listed in the rustdoc of `kivact::ui::data_table`.

## See also

- [Show figures, status and records on a business screen](status-and-figures.md)
  for lists with a few rows
- [Set up the Pop theme and tokens](theme-and-tokens.md)
- Table, column and context API: `kivact::ui::DataTable`, `DataColumn` and
  `DataTableContext` in `cargo doc -p kivact --open`
