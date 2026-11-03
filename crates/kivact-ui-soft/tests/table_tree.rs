//! Role: Exercise a hierarchy through the standard table in the Soft theme.
use kivact::interaction::SortOrder;
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, WindowMode, render_with_handle};
use kivact::view::CheckedState;

#[test]
fn tree_rows_keep_table_roles_and_expand_before_paging() {
    let (screen, (expanded, page)) = render_with_handle(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Light),
            window: WindowMode::Headless,
            ..RenderOptions::default()
        },
        || {
            let rows = vec![
                (1, None, "Work"),
                (2, Some(1), "Design"),
                (3, Some(2), "Assets"),
                (4, Some(1), "Build"),
                (5, None, "Other"),
            ];
            let expanded = signal(vec![1]);
            let page = signal(0);
            let sort = signal(Some((0, SortOrder::Descending)));
            let name = DataColumn::new("Name", |row: &(i32, Option<i32>, &str)| row.2.to_owned())
                .compare(|a, b| a.2.cmp(b.2));
            let table = data_table(rows, |row| row.0, vec![name])
                .tree(|row| row.1, expanded)
                .tree_expand_all(true)
                .page(page, 4)
                .sort(sort);
            ((expanded, page), table)
        },
    );
    let work = screen.get_by_role(Role::Row, "Work");
    let design = screen.get_by_role(Role::Row, "Design");
    let build = screen.get_by_role(Role::Row, "Build");
    assert_eq!(work.level(), Some(1));
    assert_eq!(work.is_expanded(), Some(true));
    assert_eq!(design.level(), Some(2));
    assert_eq!(design.is_expanded(), Some(false));
    assert_eq!(build.is_expanded(), None);
    assert!(design.rect().unwrap().y < build.rect().unwrap().y);
    assert_eq!(
        screen
            .get_by_role(Role::Button, "Expand all rows")
            .checked(),
        Some(CheckedState::Indeterminate)
    );
    screen.get_by_role(Role::Button, "Expand all rows").click();
    assert_eq!(expanded.get_untracked(), [1, 2]);
    assert!(screen.query_by_role(Role::Row, "Other").is_none());
    let assets = screen.get_by_role(Role::Row, "Assets");
    assert_eq!(assets.level(), Some(3));
    let design_cell = design.get_by_role(Role::GridCell, "Design");
    let assets_cell = assets.get_by_role(Role::GridCell, "Assets");
    let design_text = design_cell
        .get_by_role(Role::Text, "Design")
        .rect()
        .unwrap();
    let assets_text = assets_cell
        .get_by_role(Role::Text, "Assets")
        .rect()
        .unwrap();
    assert!((assets_text.x - design_text.x - 16.0).abs() < 1.0);
    page.set(1);
    screen.run_until_idle();
    screen.get_by_role(Role::Row, "Other");
    screen
        .get_by_role(Role::Button, "Collapse all rows")
        .click();
    assert!(expanded.get_untracked().is_empty());
    assert!(screen.query_by_role(Role::Row, "Design").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
