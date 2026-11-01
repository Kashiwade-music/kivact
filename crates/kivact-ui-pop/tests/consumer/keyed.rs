//! Role: Stable application identities through Pop styled public builders.

use kivact::prelude::*;
use kivact::ui::ListItem;
use kivact::view::{A11yAction, Orientation};
use kivact_ui_pop as ui;

fn option(key: &str, label: &str) -> ListItem<String> {
    ListItem::new(key.to_owned(), label)
}

#[test]
fn tabs_and_segments_keep_selected_keys_while_rows_and_focus_change() {
    for pill in [false, true] {
        let role = if pill { Role::RadioButton } else { Role::Tab };
        let (screen, (options, selected, visible)) =
            kivact::test::render_with_handle(crate::options(), move || {
                let options = signal(vec![
                    option("a", "Alpha").icon("home"),
                    option("b", "Beta").description("4"),
                    option("g", "Gamma"),
                ]);
                // Tabs always have a selection; a radio group may have none.
                let tab = signal("b".to_owned());
                let segment = signal(Some("b".to_owned()));
                let selected = move || {
                    if pill {
                        segment.get_untracked()
                    } else {
                        Some(tab.get_untracked())
                    }
                };
                let visible = signal(true);
                (
                    (options, selected, visible),
                    column((
                        show(visible, move || {
                            if pill {
                                kivact::ui::radio_group("Section", options, segment)
                                    .orientation(Orientation::Horizontal)
                                    .variant(ui::Segmented)
                                    .into_view()
                            } else {
                                kivact::ui::tabs("Tabs", options, tab)
                                    .item(ui::tab_label)
                                    .into_view()
                            }
                        }),
                        kivact::ui::button("After"),
                    )),
                )
            });
        let beta = screen.get_by_role(role, "Beta");
        beta.focus();
        assert!(beta.is_selected());
        beta.get_by_text("4");
        options.update(|rows| rows[1] = ListItem::new(rows[1].key().clone(), "Beta renamed"));
        screen.run_until_idle();
        assert_eq!(
            beta.name(),
            "Beta renamed",
            "a label change keeps the mounted keyed row"
        );
        let creates = screen.stats().creates;
        options.update(|rows| rows.swap(0, 1));
        screen.run_until_idle();
        assert_eq!(
            screen.stats().creates,
            creates,
            "reordering retains member nodes"
        );
        assert!(beta.is_focused());
        options.update(|rows| rows[0] = rows[0].clone().disabled(true));
        screen.run_until_idle();
        assert!(beta.is_disabled());
        assert!(
            beta.is_selected(),
            "disabled selection still belongs to the caller"
        );
        assert!(screen.get_by_role(role, "Alpha").is_focused());
        assert_eq!(selected().as_deref(), Some("b"));
        options.set(vec![option("g", "Gamma")]);
        screen.run_until_idle();
        assert!(screen.query_by_role(role, "Beta renamed").is_none());
        assert!(screen.get_by_role(role, "Gamma").is_focused());
        assert_eq!(selected().as_deref(), Some("b"));
        options.set(vec![option("b", "Beta returned"), option("g", "Gamma")]);
        screen.run_until_idle();
        let returned = screen.get_by_role(role, "Beta returned");
        assert!(returned.is_selected());
        assert!(
            !returned.is_focused(),
            "selected identity and mounted active focus differ"
        );
        returned.a11y(A11yAction::Select);
        assert!(returned.is_focused());
        returned.press_key(Key::ArrowRight);
        assert_eq!(selected().as_deref(), Some("g"));
        assert!(screen.get_by_role(role, "Gamma").is_focused());
        visible.set(false);
        screen.run_until_idle();
        assert!(screen.query_by_role(role, "Gamma").is_none());
        assert_eq!(selected().as_deref(), Some("g"));
        visible.set(true);
        screen.run_until_idle();
        assert!(screen.get_by_role(role, "Gamma").is_selected());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn pop_select_bounds_a_long_list_and_scrolls_the_last_row_into_view() {
    let screen = kivact::test::render_with(crate::options(), || {
        let rows: Vec<ListItem<u32>> = (0..30)
            .map(|i| ListItem::new(i, format!("Item {i:02}")))
            .collect();
        kivact::ui::select("Pick", rows, signal(None::<u32>))
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
