//! Role: Table consumers share keyboard movement, sorting and row selection.
use kivact::prelude::*;
use kivact::test::RenderOptions;

#[test]
fn table_stage3_keyboard_sort_and_selection() {
    let (screen, selected) = kivact::test::render_with_handle(
        RenderOptions {
            theme: kivact_ui_game::theme(kivact::style::ThemeMode::Light),

            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let selected = signal(Vec::<String>::new());
            let view = kivact::ui::data_table(
                vec![("1", "Beta"), ("2", "Alpha")],
                |row: &(&'static str, &'static str)| row.0.to_owned(),
                vec![
                    kivact::ui::DataColumn::new("Name", |row: &(&str, &str)| row.1.to_owned())
                        .width(160.0)
                        .compare(|a: &(&str, &str), b: &(&str, &str)| a.1.cmp(b.1)),
                ],
            )
            .selection(selected);
            (selected, view)
        },
    );
    screen.get_by_role(Role::ColumnHeader, "Name").focus();
    screen.press_key(Key::Enter);
    let alpha = screen.get_by_role(Role::GridCell, "Alpha");
    let beta = screen.get_by_role(Role::GridCell, "Beta");
    assert!(
        alpha.rect().expect("table cell laid out").y < beta.rect().expect("table cell laid out").y
    );
    alpha.focus();
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::GridCell, "Beta").is_focused());
    screen.press_key(Key::ArrowUp);
    screen.press_key(Key::Space);
    assert_eq!(selected.get_untracked(), vec!["2".to_owned()]);
    screen.get_by_role(Role::ColumnHeader, "Name").focus();
    screen.press_key(Key::Enter);
    assert!(
        screen
            .get_by_role(Role::GridCell, "Beta")
            .rect()
            .expect("table cell laid out")
            .y
            < screen
                .get_by_role(Role::GridCell, "Alpha")
                .rect()
                .unwrap()
                .y
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
