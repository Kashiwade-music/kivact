//! Role: Catalog page of Data Table: `kivact::ui::data_table` in the Lyra theme.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "data-table",
        name: "Data Table",
        description: "標準の kivact::ui::data_table を Lyra の theme で描いた表です。行は任意の型で、key で識別します。絞り込み・選択・並べ替え・ページは、それぞれの状態をアプリの signal で渡したときだけ付きます。Columns メニューのような部品は toolbar の slot で作ります。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact_ui_basic as basic;",
        source: include_str!("data_table.rs"),
        align: PreviewAlign::Start,
        examples: vec![
            Example::new(
                "payments",
                "Payments",
                "selection、sort、page、row_actions と、toolbar の slot で作った絞り込み欄と Columns メニュー（列の visible を同じ signal で切り替えます）を組み合わせた shadcn の例です。",
                payments,
            ),
            Example::new(
                "simple",
                "Simple",
                "selection と sort だけを渡すと、絞り込みとページ送りのない小さな表になります。",
                simple,
            ),
        ],
    }
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
            let first_page = context.page.map(|(page, _)| page);
            let filter = text_input(
                "Filter emails...",
                binding(query, move |text: String| {
                    query.set(text);
                    if let Some(page) = first_page {
                        page.set(0);
                    }
                }),
            )
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
            row((filter, node().style(style().grow(1)), columns_menu))
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
