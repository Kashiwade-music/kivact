//! Role: APG grid behavior with a kit's theme installed.

use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::ui::RowStatus;
use kivact::view::Modifiers;
use kivact_public_api_tests::support::mount_with;

#[test]
fn accessory_controls_share_one_tab_stop_and_remain_keyboard_operable() {
    let (screen, selected) = mount_with(
        RenderOptions::from_env().theme(kivact_ui_soft::theme(kivact::style::ThemeMode::Light)),
        || {
            let selected = signal(Vec::<i32>::new());
            let expanded = signal(Vec::<i32>::new());
            (
                selected,
                column((
                    data_table(
                        vec![1, 2],
                        |row| *row,
                        vec![DataColumn::new("Value", |value: &i32| value.to_string())],
                    )
                    .selection(selected)
                    .row_index(1)
                    .row_status(|_| Some(RowStatus::new("Ready", Tone::Neutral)))
                    .row_details(|_| text("Details"))
                    .expanded_rows(expanded)
                    .row_actions(80.0, |_| button("Run")),
                    node()
                        .role(Role::Button)
                        .name("After")
                        .focusable(true)
                        .style(style().size(80, 20)),
                )),
            )
        },
    );
    let grid = screen.get_by_role(Role::Grid, "Data table");
    assert_eq!(grid.row_count(), Some(3));
    assert_eq!(grid.column_count(), Some(6));
    assert_eq!(
        screen
            .get_by_role(Role::ColumnHeader, "Actions")
            .column_index(),
        Some(6)
    );
    assert_eq!(screen.get_by_role(Role::Row, "1").row_index(), Some(2));
    screen.press_tab(false);
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    screen.press_tab(true);
    screen.get_by_role(Role::CheckBox, "Select 1").focus();
    screen.press_key(Key::Escape);
    screen.press_key(Key::Home);
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::Enter);
    assert!(screen.get_by_role(Role::CheckBox, "Select 1").is_focused());
    screen.press_key(Key::Space);
    assert_eq!(selected.get_untracked(), vec![1]);
    screen.press_key(Key::Escape);
    screen.press_key(Key::End);
    screen.press_key(Key::F(2));
    assert!(
        screen
            .all_by_role(Role::Button, "Run")
            .into_iter()
            .any(|button| button.is_focused())
    );
    screen.press_key(Key::Escape);
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn group_disclosures_participate_in_grid_arrow_navigation() {
    let (screen, ()) = mount_with(
        RenderOptions::from_env().theme(kivact_ui_soft::theme(kivact::style::ThemeMode::Light)),
        || {
            let collapsed = signal(Vec::<String>::new());
            (
                (),
                column((
                    data_table(
                        vec![1, 2],
                        |row| *row,
                        vec![DataColumn::new("Value", |value: &i32| value.to_string())],
                    )
                    .group_by(|row| if *row == 1 { "A" } else { "B" }, collapsed),
                    node()
                        .role(Role::Button)
                        .name("After")
                        .focusable(true)
                        .style(style().size(80, 20)),
                )),
            )
        },
    );
    assert_eq!(
        screen.get_by_role(Role::Grid, "Data table").row_count(),
        Some(5)
    );
    screen.get_by_role(Role::GridCell, "1").focus();
    screen.press_key(Key::ArrowUp);
    assert!(screen.get_by_role(Role::GridCell, "A").is_focused());
    screen.press_key(Key::F(2));
    assert!(
        screen
            .get_by_role(Role::Button, "Collapse group A")
            .is_focused()
    );
    screen.press_key(Key::Space);
    screen.press_key(Key::Escape);
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::GridCell, "B").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::GridCell, "2").is_focused());
    screen.press_key(Key::ArrowUp);
    assert!(screen.get_by_role(Role::GridCell, "B").is_focused());
    screen.press_key_with(
        Key::Home,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(screen.get_by_role(Role::ColumnHeader, "Value").is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
