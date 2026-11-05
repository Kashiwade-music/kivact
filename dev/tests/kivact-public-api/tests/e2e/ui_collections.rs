//! Role: Verify semantic static collections through the public UI builders.
use crate::support::mount;
use kivact::prelude::*;
use kivact::style::TextAlign;
use kivact::view::A11yAction;

#[test]
fn static_table_exposes_rows_and_cells() {
    let (screen, span) = mount(|| {
        let span = signal(2usize);
        (
            span,
            table((
                table_header((table_row((
                    table_head("Name").name("Name"),
                    table_head("Amount").name("Amount"),
                )),)),
                table_body((table_row((
                    table_cell("Invoice").name("Invoice"),
                    table_cell("42").name("Total").align(TextAlign::End),
                )),)),
                table_footer((table_row((table_cell("Summary")
                    .name("Summary")
                    .col_span(span),)),)),
            ))
            .name("Invoices"),
        )
    });
    let table = screen.within(screen.get_by_role(Role::Table, "Invoices"));
    assert_eq!(table.all_by_role(Role::RowGroup, "").len(), 3);
    assert_eq!(table.all_by_role(Role::Row, "").len(), 3);
    table.get_by_role(Role::ColumnHeader, "Name");
    table.get_by_role(Role::Cell, "Invoice");
    assert_eq!(
        table.get_by_role(Role::Cell, "Summary").column_span(),
        Some(2)
    );
    span.set(0);
    screen.run_until_idle();
    assert_eq!(
        table.get_by_role(Role::Cell, "Summary").column_span(),
        Some(1)
    );
    assert!(screen.query_by_role(Role::Grid, "Invoices").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn item_list_exposes_list_semantics() {
    let (screen, presses) = mount(|| {
        let presses = signal(0);
        (
            presses,
            item_list((
                item_row("First")
                    .media(text("1"))
                    .description("Details")
                    .actions(button("Inspect")),
                item_row("Second")
                    .header(text("Header"))
                    .footer(text("Footer"))
                    .on_press(move || presses.update(|n| *n += 1)),
            ))
            .name("Items"),
        )
    });
    let list = screen.within(screen.get_by_role(Role::List, "Items"));
    list.get_by_role(Role::ListItem, "First");
    list.get_by_role(Role::Button, "Inspect");
    let second = list.get_by_role(Role::ListItem, "Second");
    second.focus();
    second.press_key(Key::Enter);
    second.press_key(Key::Space);
    second.a11y(A11yAction::Activate);
    assert_eq!(presses.get_untracked(), 3);
    assert!(screen.query_by_role(Role::ListBox, "Items").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
