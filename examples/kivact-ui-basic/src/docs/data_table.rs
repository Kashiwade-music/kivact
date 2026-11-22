//! Catalog page of Data Table: `kivact::ui::data_table` in the Lyra theme.
use kivact::ui::Paging;
use kivact::ui::{DataColumn, data_table};

use catalog_shell::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("data-table", "Data Table", "Components", include_str!("data_table.rs"))
        .description("The standard kivact::ui::data_table drawn in the Lyra theme. Rows can be of any type and are identified by a key. Filtering, selection, sorting and paging appear only when the app passes their state as signals. Parts such as the Columns menu are built in the toolbar slot.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::Paging;\nuse kivact_ui_basic as basic;")
        .align(PreviewAlign::Start)
        .example(Example::new(
            "payments",
            "Payments",
            "The shadcn example, combining selection, sort, page, row_actions, a filter field and a Columns menu built in the toolbar slot (it switches the visible columns with the same signal).",
            payments,
        ))
        .example(Example::new(
            "simple",
            "Simple",
            "Pass only selection and sort for a small table with no filter and no paging.",
            simple,
        ))
}

// example: payments
#[derive(Clone, PartialEq)]
struct Payment {
    id: u32,
    status: &'static str,
    email: &'static str,
    amount: f64,
}

fn payment_rows() -> Vec<Payment> {
    let payment = |id, status, email, amount| Payment {
        id,
        status,
        email,
        amount,
    };
    vec![
        payment(1, "success", "ken99@example.com", 316.0),
        payment(2, "success", "abe45@example.com", 242.0),
        payment(3, "processing", "monserrat44@example.com", 837.0),
        payment(4, "failed", "silas22@example.com", 874.0),
        payment(5, "failed", "carmella@example.com", 721.0),
        payment(6, "pending", "dora71@example.com", 99.0),
        payment(7, "success", "zed@example.com", 15.0),
    ]
}

// xtask-allow(state): independent: sort is written by the column headers through the data table's sort binding, which also resets page to 0; page is then moved only by the pager, so every sort can be shown on every page
fn payments() -> impl IntoView {
    let data = payment_rows();
    let query = signal(String::new());
    let selected = signal(Vec::<u32>::new());
    let sort = signal(None);
    let page = signal(0_usize);
    let visible = signal(vec![true; 3]);
    let copied = signal(String::new());
    let columns = vec![
        DataColumn::new("Status", |p: &Payment| p.status.to_string())
            .visible(move || visible.with(|v| v[0])),
        DataColumn::new("Email", |p: &Payment| p.email.to_string())
            .compare(|a, b| a.email.cmp(b.email))
            .filter(|p: &Payment, q: &str| p.email.contains(q))
            .visible(move || visible.with(|v| v[1])),
        DataColumn::new("Amount", |p: &Payment| format!("${:.2}", p.amount))
            .compare(|a, b| a.amount.total_cmp(&b.amount))
            .numeric(true)
            .width(110.0)
            .visible(move || visible.with(|v| v[2])),
    ];
    let table = data_table(data, |p: &Payment| p.id, columns)
        .row_label(|p: &Payment| p.email.to_string())
        .filter(query)
        .selection(selected)
        .sort(sort)
        .page(page, 5)
        .toolbar(move |context| {
            let query = context.filter.expect("the table has a filter");
            let first_page = match context.paging {
                Paging::Local { page, .. } => Some(page),
                Paging::Remote { page, .. } => Some(page),
                Paging::Off => None,
            };
            let filter = text_input(binding(query, move |text: String| {
                query.set(text);
                if let Some(page) = first_page {
                    page.set(0);
                }
            }))
            .label("Filter emails...")
            .placeholder("Filter emails...")
            .style(style().max_width(384).width(pct(100)));
            let titles = ["Status", "Email", "Amount"];
            let entries = memo(move || {
                let shown = visible.get();
                titles
                    .iter()
                    .enumerate()
                    .map(|(i, title)| MenuEntry::checkbox(i, *title, shown[i]).keep_open(true))
                    .collect::<Vec<_>>()
            });
            let columns_menu = menu("Columns", entries)
                .on_activate(move |index: usize| {
                    visible.update(|shown| shown[index] = !shown[index]);
                })
                .attach(
                    button("Columns")
                        .variant(ButtonVariant::Outline)
                        .trailing(icon("chevron-down")),
                );
            row((filter, node().style(style().flex_grow(1)), columns_menu))
                .style(style().align_items(Align::Center).gap(8).padding_y(16))
        })
        .row_actions(40.0, move |row: Value<Payment>| {
            let id = row.with(|p| p.id);
            let entries = vec![MenuEntry::item(id, "Copy payment ID")];
            menu("Actions", entries)
                .on_activate(move |id: u32| {
                    copied.set(format!("Copied payment {id}"));
                })
                .attach(
                    button("Open menu")
                        .hide_label(true)
                        .icon(icon("ellipsis"))
                        .variant(ButtonVariant::Ghost),
                )
        });
    column((table, basic::muted(move || copied.get()))).style(style().width(560))
}
// end example

// example: simple
fn simple() -> impl IntoView {
    #[derive(Clone, PartialEq)]
    struct Member {
        name: &'static str,
        role: &'static str,
    }
    let member = |name, role| Member { name, role };
    let data = vec![
        member("Ada Lovelace", "Engineer"),
        member("Grace Hopper", "Admiral"),
        member("Alan Turing", "Mathematician"),
        member("Edsger Dijkstra", "Professor"),
    ];
    let selected = signal(Vec::<&'static str>::new());
    let sort = signal(None);
    let columns = vec![
        DataColumn::new("Name", |m: &Member| m.name.to_string()).compare(|a, b| a.name.cmp(b.name)),
        DataColumn::new("Role", |m: &Member| m.role.to_string()),
    ];
    data_table(data, |m: &Member| m.name, columns)
        .selection(selected)
        .sort(sort)
        .style(style().width(460))
}
// end example
