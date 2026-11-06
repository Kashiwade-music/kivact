//! Role: Verify typed table filters commit transactionally in popovers.
use kivact::interaction::FilterValue;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::MirrorMode;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::ui::DataColumn;
use kivact::ui::FilterField;
use kivact::ui::FilterUi;
use kivact::ui::data_table;
use kivact::view::Key;
use kivact::view::Role;
use std::collections::HashMap;

type Score = (&'static str, u8);

fn column() -> DataColumn<Score> {
    DataColumn::new("Score", |row: &Score| row.0.to_owned())
        .compare(|a, b| a.1.cmp(&b.1))
        .filter_field(
            FilterField::Number {
                min: Some(0.0),
                max: Some(50.0),
            },
            |row, value| matches!(value, FilterValue::Number(minimum) if f64::from(row.1) >= *minimum),
        )
}

#[test]
fn filter_popover_discards_unapplied_values_and_can_reset_a_typed_query() {
    let (screen, (filters, sort)) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).mirror(MirrorMode::Off),
        || {
            let filters = signal(HashMap::new());
            let sort = signal(None);
            (
                (filters, sort),
                data_table(
                    vec![("Ten", 10), ("Thirty", 30)],
                    |row| row.1,
                    vec![column()],
                )
                .typed_column_filters(filters)
                .sort(sort),
            )
        },
    );
    let trigger = screen.get_by_role(Role::Button, "Filter Score");
    trigger.focus();
    screen.press_key(Key::Enter);
    screen
        .get_by_role(Role::TextBox, "Filter Score")
        .type_text("20");
    assert!(filters.get_untracked().is_empty());
    assert!(screen.query_by_role(Role::Row, "Ten").is_some());
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Button, "Apply").is_none());
    assert!(filters.get_untracked().is_empty());
    assert!(sort.get_untracked().is_none());

    trigger.click();
    let input = screen.get_by_role(Role::TextBox, "Filter Score");
    assert_eq!(input.value().as_deref(), Some(""));
    input.type_text("20");
    screen.get_by_role(Role::Button, "Apply").click();
    assert_eq!(
        filters.get_untracked().get(&0),
        Some(&FilterValue::Number(20.0))
    );
    assert!(screen.query_by_role(Role::Row, "Ten").is_none());
    assert!(screen.query_by_role(Role::Row, "Thirty").is_some());
    assert!(sort.get_untracked().is_none());

    trigger.click();
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Filter Score")
            .value()
            .as_deref(),
        Some("20")
    );
    screen.get_by_role(Role::Button, "Reset").click();
    assert!(filters.get_untracked().is_empty());
    assert!(screen.query_by_role(Role::Row, "Ten").is_some());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn compact_inline_filter_commits_valid_values_without_a_popup() {
    let (screen, filters) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).mirror(MirrorMode::Off),
        || {
            let filters = signal(HashMap::new());
            (
                filters,
                data_table(
                    vec![("Ten", 10), ("Thirty", 30)],
                    |row| row.1,
                    vec![column()],
                )
                .typed_column_filters(filters)
                .filter_ui(FilterUi::InlineCompact),
            )
        },
    );
    assert!(screen.query_by_role(Role::Button, "Filter Score").is_none());
    screen
        .get_by_role(Role::TextBox, "Filter Score")
        .type_text("21");
    assert_eq!(
        filters.get_untracked().get(&0),
        Some(&FilterValue::Number(21.0))
    );
    assert!(screen.query_by_role(Role::Row, "Ten").is_none());
    assert!(screen.query_by_role(Role::Row, "Thirty").is_some());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
