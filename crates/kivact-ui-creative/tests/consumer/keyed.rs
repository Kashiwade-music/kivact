//! Role: Tests for keyed choices, dropdowns and tabs of the Creative kit.

use super::*;
use kivact::ui::ListItem;

fn options(items: &[(&str, &str)]) -> Vec<ListItem<String>> {
    items
        .iter()
        .map(|(key, label)| ListItem::new((*key).to_owned(), *label))
        .collect()
}

#[test]
fn keyed_document_tabs_leave_order_and_closing_to_the_caller() {
    let (screen, (items, selected, reorders, closes)) =
        kivact::test::render_with_handle(crate::options(), || {
            let items = signal(options(&[("a", "Alpha"), ("b", "Beta"), ("c", "Gamma")]));
            let selected = signal("a".to_owned());
            let reorders = signal(Vec::<(String, String)>::new());
            let closes = signal(Vec::<String>::new());
            (
                (items, selected, reorders, closes),
                column((ui::document_tabs("Documents", items, selected)
                    .on_reorder(move |pair: (String, String)| {
                        reorders.update(|all| all.push(pair.clone()));
                        items.update(|items| {
                            let a = items.iter().position(|item| *item.key() == pair.0);
                            let b = items.iter().position(|item| *item.key() == pair.1);
                            if let (Some(a), Some(b)) = (a, b) {
                                items.swap(a, b);
                            }
                        });
                    })
                    .on_close(move |key: String| {
                        closes.update(|all| all.push(key.clone()));
                        items.update(|items| items.retain(|item| *item.key() != key));
                    }),)),
            )
        });
    let alpha = screen.get_by_role(Role::Tab, "Alpha");
    alpha.focus();
    let rect = alpha.rect().expect("alpha tab");
    alpha.drag(
        rect.center(),
        Point::new(rect.center().x + 65.0, rect.center().y),
    );
    assert_eq!(
        reorders.get_untracked(),
        vec![("a".to_owned(), "b".to_owned())]
    );
    assert_eq!(selected.get_untracked(), "a");
    items.update(|items| items[1] = ListItem::new(items[1].key().clone(), "Renamed Alpha"));
    screen.run_until_idle();
    screen
        .get_by_role(Role::Tab, "Renamed Alpha")
        .press_key(Key::ArrowLeft);
    assert_eq!(
        selected.get_untracked(),
        "b",
        "the tabs follow the caller's order"
    );
    items.update(|items| items[0] = items[0].clone().disabled(true));
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Tab, "Beta").is_disabled());
    screen
        .get_by_role(Role::Tab, "Renamed Alpha")
        .press_key(Key::Home);
    assert_eq!(selected.get_untracked(), "a");
    screen
        .get_by_role(Role::Tab, "Gamma")
        .get_by_role(Role::Button, "Close tab")
        .a11y(A11yAction::Activate);
    assert_eq!(closes.get_untracked(), vec!["c".to_owned()]);
    assert!(screen.query_by_role(Role::Tab, "Gamma").is_none());
    assert_eq!(reorders.get_untracked().len(), 1);
    clean(&screen);
}

#[test]
fn creative_select_bounds_a_long_list_and_scrolls_the_last_row_into_view() {
    let screen = kivact::test::render_with(crate::options(), || {
        let rows: Vec<ListItem<u32>> = (0..30)
            .map(|i| ListItem::new(i, format!("Item {i:02}")))
            .collect();
        kivact::ui::select("Pick", rows, signal(None::<u32>))
            .placeholder("")
            .name("Pick")
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Pick");
    trigger.press_key_with(
        Key::ArrowDown,
        kivact::view::Modifiers {
            alt: true,
            ..Default::default()
        },
    );
    assert_eq!(trigger.is_expanded(), Some(true), "Alt+Down opens");
    let list = screen
        .get_by_role(Role::ListBox, "Pick")
        .rect()
        .expect("laid out");
    assert!(list.height < 30.0 * 20.0, "the list is bounded: {list:?}");
    screen.press_key(Key::End);
    let last = screen.focused().expect("a row has the focus").rect();
    let last = last.expect("laid out");
    assert!(
        last.y + last.height <= list.y + list.height + 0.5,
        "End scrolls the last row into the list: {last:?} in {list:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
