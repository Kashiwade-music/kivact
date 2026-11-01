//! Role: Custom cells and column resizing of the generic table.

use super::*;

#[test]
fn custom_table_cells_stay_reactive_and_columns_resize_via_public_input() {
    let (screen, (rows, widths)) = kivact::test::render_with_handle(crate::options(), || {
        let rows = signal(vec![line(1, ["Layer", "Off"])]);
        let widths = signal(vec![150.0, 120.0]);
        (
            (rows, widths),
            lines(&["Name", "Enabled"], rows, signal(Vec::new()))
                .resizable(widths)
                .cell_controls(move |item, col| {
                    if col == 0 {
                        return text(move || item.get().cells[0].clone()).into_view();
                    }
                    let id = item.get_untracked().id;
                    kivact::ui::switch(
                        "Layer enabled",
                        binding(
                            (move || item.get().cells[1] == "On").into_value(),
                            move |on| {
                                rows.update(|v| {
                                    if let Some(row) = v.iter_mut().find(|r| r.id == id) {
                                        row.cells[1] = if on { "On" } else { "Off" }.into();
                                    }
                                })
                            },
                        ),
                    )
                    .into_view()
                }),
        )
    });
    let enabled = screen.get_by_role(Role::Switch, "Layer enabled");
    enabled.click();
    assert_eq!(rows.get_untracked()[0].cells[1], "On");
    assert_eq!(enabled.is_checked(), Some(true));
    let resize = screen.get_by_role(Role::Splitter, "Resize Name");
    resize.a11y_with_value(A11yAction::SetValue, "180");
    assert_eq!(widths.get_untracked()[0], 180.0);
    assert_eq!(
        screen
            .get_by_role(Role::ColumnHeader, "Name")
            .rect()
            .expect("header")
            .width,
        180.0
    );
    resize.press_key(Key::ArrowRight);
    assert_eq!(widths.get_untracked()[0], 190.0);
    resize.press_key(Key::ArrowLeft);
    assert_eq!(widths.get_untracked()[0], 180.0);
    resize.press_key(Key::ArrowUp);
    resize.press_key(Key::ArrowDown);
    assert_eq!(widths.get_untracked()[0], 180.0);
    resize.press_key(Key::Home);
    assert_eq!(widths.get_untracked()[0], 18.0);
    resize.press_key(Key::End);
    assert_eq!(widths.get_untracked()[0], 2048.0);
    resize.a11y_with_value(A11yAction::SetValue, "181");
    assert_eq!(widths.get_untracked()[0], 181.0);
    resize.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(widths.get_untracked()[0], 181.0);
    let r = resize.rect().expect("resize");
    resize.drag(
        Point::new(r.width / 2.0, r.height / 2.0),
        Point::new(r.width / 2.0 + 30.0, r.height / 2.0),
    );
    assert_eq!(widths.get_untracked()[0], 211.0);
    clean(&screen);
}

#[test]
fn shared_columns_sort_and_match_only_where_they_say_and_draw_their_cell_views() {
    let (screen, (rows, query)) = kivact::test::render_with_handle(crate::options(), || {
        let rows = signal(vec![
            line(1, ["Beta", "2"]),
            line(2, ["Alpha", "10"]),
            line(3, ["Gamma", "1"]),
        ]);
        let query = signal(String::new());
        let columns = vec![
            // Neither sorts nor matches: the shared column's default.
            kivact::ui::DataColumn::new("Name", |line: &Line| line.cells[0].clone()),
            kivact::ui::DataColumn::new("Frames", |line: &Line| line.cells[1].clone())
                .sortable()
                .filter(|line, query| line.cells[1] == query)
                .cell(|line| text(format!("{} f", line.cells[1]))),
        ];
        (
            (rows, query),
            kivact::ui::data_table(rows, |line: &Line| line.id, columns)
                .filter(query)
                .toolbar(|_| ()),
        )
    });
    let order = |screen: &Screen| {
        let mut names = ["Alpha", "Beta", "Gamma"]
            .into_iter()
            .filter_map(|name| {
                let row = screen.query_by_role(Role::Row, name)?;
                Some((row.rect().expect("a laid out row").y, name))
            })
            .collect::<Vec<_>>();
        names.sort_by(|a, b| a.0.total_cmp(&b.0));
        names.into_iter().map(|(_, name)| name).collect::<Vec<_>>()
    };
    screen.get_by_role(Role::ColumnHeader, "Name").click();
    assert_eq!(
        order(&screen),
        ["Beta", "Alpha", "Gamma"],
        "Name does not sort"
    );
    screen.get_by_role(Role::ColumnHeader, "Frames").click();
    assert_eq!(
        order(&screen),
        ["Gamma", "Beta", "Alpha"],
        "sortable() orders numeric texts as numbers"
    );
    // The cell view replaces the text, which still names the cell, and it
    // follows its row.
    assert!(screen.get_by_text("10 f").exists());
    rows.update(|rows| rows[1].cells[1] = "12".into());
    screen.run_until_idle();
    assert!(screen.get_by_text("12 f").exists());
    query.set("Alpha".into());
    screen.run_until_idle();
    assert!(
        order(&screen).is_empty(),
        "Name takes no part in the filter"
    );
    query.set("12".into());
    screen.run_until_idle();
    assert_eq!(order(&screen), ["Alpha"]);
    clean(&screen);
}
