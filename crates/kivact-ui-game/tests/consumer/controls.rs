//! Role: Buttons, checks, switches, disclosures and the table through the public API.

use kivact::prelude::*;
use kivact::view::A11yAction;

#[test]
fn consumer_buttons_checks_mixed_states_and_modifiers() {
    let (screen, (count, checked, mixed)) =
        kivact::test::render_with_handle(crate::support::options(true), || {
            let count = signal(0);
            let checked = signal(false);
            let mixed = signal(true);
            (
                (count, checked, mixed),
                column((
                    kivact::ui::button("Approve")
                        .variant(kivact_ui_game::Slant)
                        .test_id("approve")
                        .style(style().height(46))
                        .on_press(move || count.update(|v| *v += 1)),
                    kivact::ui::button("Disabled")
                        .variant(kivact_ui_game::Slant)
                        .disabled(true),
                    kivact::ui::checkbox("Permissions", checked).indeterminate(mixed),
                    kivact::ui::switch("Network", checked),
                ))
                .style(style().gap(20).padding(20)),
            )
        });
    let button = screen.get_by_test_id("approve");
    assert_eq!(button.accessibility_bounds().map(|r| r.height), Some(46.));
    button.click();
    button.press_key(Key::Space);
    assert_eq!(count.get_untracked(), 2);
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    let check = screen.get_by_role(Role::CheckBox, "Permissions");
    assert!(check.is_indeterminate());
    check.a11y(A11yAction::Toggle);
    assert!(checked.get_untracked());
    mixed.set(false);
    screen.run_until_idle();
    assert!(!check.is_indeterminate());
    screen
        .get_by_role(Role::Switch, "Network")
        .press_key(Key::Space);
    assert!(!checked.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_checkbox_switch_and_disclosure_follow_the_shared_toggle_rules() {
    let (screen, (check, switch, open)) =
        kivact::test::render_with_handle(crate::support::options(true), || {
            let check = signal(false);
            let switch = signal(false);
            let open = signal(false);
            (
                (check, switch, open),
                column((
                    kivact::ui::checkbox("Notify", check),
                    kivact::ui::switch("Network", switch),
                    kivact::ui::disclosure("Settings", open, || text("Inspector content")),
                    kivact::ui::checkbox("Locked", kivact::signal(false)).disabled(true),
                ))
                .style(style().padding(20).gap(16)),
            )
        });
    let check_box = screen.get_by_role(Role::CheckBox, "Notify");
    let switch_node = screen.get_by_role(Role::Switch, "Network");
    check_box.press_key(Key::Enter);
    switch_node.press_key(Key::Enter);
    assert!(!check.get_untracked() && !switch.get_untracked(), "Enter");
    for node in [&check_box, &switch_node] {
        node.press_key(Key::Space);
        assert_eq!(node.value(), None, "role and checked, no On/Off text");
    }
    assert!(check.get_untracked() && switch.get_untracked());
    check_box.a11y(A11yAction::Activate);
    switch_node.a11y(A11yAction::Toggle);
    assert!(!check.get_untracked() && !switch.get_untracked());
    let locked = screen.get_by_role(Role::CheckBox, "Locked");
    locked.force_click();
    locked.a11y(A11yAction::Toggle);
    assert_eq!(locked.is_checked(), Some(false));

    let settings = screen.get_by_role(Role::Button, "Settings");
    assert_eq!(settings.is_expanded(), Some(false));
    settings.press_key(Key::Enter);
    assert!(open.get_untracked());
    settings.press_key(Key::Space);
    assert!(!open.get_untracked());
    settings.a11y(A11yAction::Toggle);
    assert_eq!(settings.is_expanded(), Some(true));
    settings.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_table_preserves_selection_and_updates_cells() {
    let (screen, (rows, selected, query)) =
        kivact::test::render_with_handle(crate::support::options(true), || {
            let rows = signal(vec![("a", "Zulu".to_owned()), ("b", "Alpha".to_owned())]);
            let selected = signal(Vec::new());
            let query = signal(String::new());
            (
                (rows, selected, query),
                column((
                    kivact::ui::text_input("Filter", query),
                    kivact::ui::data_table(
                        rows,
                        |row: &(&'static str, String)| row.0,
                        vec![
                            kivact::ui::DataColumn::new("NAME", |row: &(&str, String)| {
                                row.1.clone()
                            })
                            .width(300.)
                            .compare(|a: &(&str, String), b: &(&str, String)| a.1.cmp(&b.1))
                            .filter(|row: &(&str, String), query: &str| row.1.contains(query)),
                        ],
                    )
                    .selection(selected)
                    .row_label(|row: &(&'static str, String)| row.0.to_owned())
                    .footer(|_| text("Footer renderer"))
                    .row_actions(100.0, |_| text("Row renderer"))
                    .toolbar(|_| text("Bulk renderer"))
                    .filter(query),
                ))
                .style(style().padding(20).gap(12)),
            )
        });
    screen.get_by_role(Role::CheckBox, "Select a").click();
    assert_eq!(selected.get_untracked(), vec!["a"]);
    screen.get_by_text("Footer renderer");
    screen.get_by_text("Bulk renderer");
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Select all")
            .is_indeterminate()
    );
    screen
        .get_by_role(Role::ColumnHeader, "NAME")
        .a11y(A11yAction::Activate);
    query.set("Alpha".into());
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Row, "a").is_none());
    assert_eq!(selected.get_untracked(), vec!["a"]);
    screen.get_by_role(Role::CheckBox, "Select all").click();
    assert_eq!(selected.get_untracked().len(), 2);
    rows.update(|v| v[1].1 = "Changed".into());
    query.set(String::new());
    screen.run_until_idle();
    screen.get_by_text("Changed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
