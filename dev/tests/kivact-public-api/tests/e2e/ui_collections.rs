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

#[test]
fn item_list_positions_and_delegation_use_the_actual_children() {
    let (screen, presses) = mount(|| {
        let presses = signal(0);
        let target = node_ref();
        (
            presses,
            item_list((
                item_row("Delegate").delegate_to(target).actions(
                    button("Inspect target").node_ref(target).on_press(move || {
                        presses.update(|count| *count += 1);
                    }),
                ),
                item_row("Explicit position").position(4usize, 8usize),
            ))
            .header(text("Documents"))
            .marker(kivact::ui::ItemListMarker::Decimal)
            .start(3),
        )
    });
    let list = screen.get_by_role(Role::List, "");
    let labels = list.labelled_by();
    assert_eq!(labels.len(), 1);
    screen
        .within(labels[0])
        .get_by_role(Role::Text, "Documents");
    let first = screen.get_by_role(Role::ListItem, "Delegate");
    assert_eq!(first.pos_in_set(), Some(1));
    assert_eq!(first.set_size(), Some(2));
    let explicit = screen.get_by_role(Role::ListItem, "Explicit position");
    assert_eq!(explicit.pos_in_set(), Some(5));
    assert_eq!(explicit.set_size(), Some(8));
    first.click();
    assert_eq!(presses.get_untracked(), 1);
    assert!(
        screen
            .get_by_role(Role::Button, "Inspect target")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
