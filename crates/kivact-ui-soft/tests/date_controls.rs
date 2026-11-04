//! Role: Verify keyboard selection and date-range staging in Soft controls.
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::view::Key;
use kivact::view::LiveRegion;
use kivact::view::Modifiers;
use kivact::view::Role;

fn options() -> RenderOptions {
    RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light))
}

fn date(day: u8) -> CalendarDate {
    CalendarDate::new(2026, 10, day)
}

#[test]
fn calendar_keys_move_focus_and_enter_selects() {
    let (screen, selected) = render_with_handle(options(), || {
        let selected = signal(Some(date(14)));
        (
            selected,
            kivact_ui_soft::calendar(selected)
                .week_start(Weekday::Monday)
                .autofocus(true),
        )
    });
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-14")
            .is_focused()
    );
    screen.press_key(Key::ArrowRight);
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-15")
            .is_focused()
    );
    screen.press_key(Key::Home);
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-12")
            .is_focused()
    );
    screen.press_key(Key::End);
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-18")
            .is_focused()
    );
    screen.press_key(Key::PageDown);
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-11-18")
            .is_focused()
    );
    assert_eq!(
        screen
            .get_by_role(Role::Status, "November 2026")
            .live_region(),
        LiveRegion::Polite
    );
    screen.press_key(Key::PageUp);
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some(date(18)));
    assert_eq!(
        screen
            .get_by_role(Role::Status, "Date selected: 2026-10-18")
            .live_region(),
        LiveRegion::Polite
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn range_stages_anchor_cancels_and_commits_reverse_with_span_constraints() {
    let previous = DateRange::new(date(1), date(4));
    let (screen, selected) = render_with_handle(options(), move || {
        let selected = signal(Some(previous));
        (
            selected,
            kivact_ui_soft::calendar_range(selected)
                .default_month(CalendarMonth::new(2026, 10))
                .range_span(3, Some(5)),
        )
    });
    screen.get_by_role(Role::GridCell, "2026-10-12").click();
    assert_eq!(selected.get_untracked(), Some(previous));
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-12")
            .is_focused()
    );
    screen.get_by_role(Role::GridCell, "2026-10-08").hover();
    assert_eq!(selected.get_untracked(), Some(previous));
    screen.get_by_role(Role::Status, "Start date selected: 2026-10-12");
    screen.press_key(Key::Escape);
    screen.get_by_role(Role::Status, "Range selection cancelled");
    assert_eq!(selected.get_untracked(), Some(previous));
    screen.get_by_role(Role::GridCell, "2026-10-12").click();
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-11")
            .is_disabled()
    );
    assert_eq!(selected.get_untracked(), Some(previous));
    screen.get_by_role(Role::GridCell, "2026-10-12").click();
    assert_eq!(selected.get_untracked(), Some(previous));
    screen.get_by_role(Role::GridCell, "2026-10-12").click();
    screen.get_by_role(Role::GridCell, "2026-10-08").click();
    assert_eq!(
        selected.get_untracked(),
        Some(DateRange::new(date(8), date(12)))
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn date_input_commits_draft_and_calendar_returns_focus() {
    let (screen, selected) = render_with_handle(options(), || {
        let selected = signal(None::<CalendarDate>);
        (
            selected,
            kivact_ui_soft::date_input("Due", selected)
                .presentation(kivact_ui_soft::DatePresentation::Desktop),
        )
    });
    screen
        .get_by_role(Role::TextBox, "Due")
        .type_text("2026-10-14");
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some(date(14)));
    screen.press_key_with(
        Key::ArrowDown,
        Modifiers {
            alt: true,
            ..Modifiers::default()
        },
    );
    screen.get_by_role(Role::GridCell, "2026-10-15");
    assert!(screen.get_by_role(Role::TextBox, "Due").is_focused());
    screen.get_by_role(Role::Button, "Choose date").click();
    screen.get_by_role(Role::Button, "Choose date").click();
    screen.get_by_role(Role::GridCell, "2026-10-15").click();
    assert_eq!(selected.get_untracked(), Some(date(15)));
    screen.get_by_role(Role::Status, "Date selected: 2026-10-15");
    assert!(screen.get_by_role(Role::TextBox, "Due").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn date_range_input_closes_and_returns_focus_after_second_choice() {
    let (screen, selected) = render_with_handle(options(), || {
        let selected = signal(None::<DateRange>);
        (
            selected,
            kivact_ui_soft::date_range_input("Window", selected)
                .calendar(|calendar| calendar.default_month(CalendarMonth::new(2026, 10))),
        )
    });
    screen.get_by_role(Role::Button, "Window").click();
    screen.get_by_role(Role::GridCell, "2026-10-12").click();
    assert!(selected.get_untracked().is_none());
    screen.get_by_role(Role::GridCell, "2026-10-08").click();
    assert_eq!(
        selected.get_untracked(),
        Some(DateRange::new(date(8), date(12)))
    );
    assert!(screen.get_by_role(Role::Button, "Window").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn range_presets_validate_endpoints_and_publish_only_on_activation() {
    let initial = DateRange::new(date(8), date(12));
    let (screen, selected) = render_with_handle(options(), move || {
        let selected = signal(Some(initial));
        use kivact_ui_soft::{DateRangePreset, date_range_input};
        (
            selected,
            date_range_input("Preset window", selected)
                .min(date(6))
                .max(date(24))
                .min_range_span(3)
                .max_range_span(5)
                .date_constraint(|day| day != date(22))
                .presets(vec![
                    DateRangePreset::new("Current range", move || initial),
                    DateRangePreset::new("Before minimum", || DateRange::new(date(3), date(7))),
                    DateRangePreset::new("After maximum", || DateRange::new(date(22), date(26))),
                    DateRangePreset::new("Too short", || DateRange::new(date(15), date(16))),
                    DateRangePreset::new("Too long", || DateRange::new(date(15), date(20))),
                    DateRangePreset::new("Disabled endpoint", || {
                        DateRange::new(date(19), date(22))
                    }),
                    DateRangePreset::new("New range", || DateRange::new(date(16), date(19))),
                ]),
        )
    });
    screen.get_by_role(Role::Button, "Preset window").click();
    let current = screen.get_by_role(Role::Button, "Current range");
    assert!(current.is_focused());
    assert!(current.is_selected());
    for label in [
        "Before minimum",
        "After maximum",
        "Too short",
        "Too long",
        "Disabled endpoint",
    ] {
        assert!(screen.get_by_role(Role::Button, label).is_disabled());
    }
    screen.get_by_role(Role::GridCell, "2026-10-14").click();
    assert_eq!(selected.get_untracked(), Some(initial));
    screen.get_by_role(Role::Button, "New range").click();
    assert_eq!(
        selected.get_untracked(),
        Some(DateRange::new(date(16), date(19)))
    );
    assert!(
        screen
            .get_by_role(Role::Button, "Preset window")
            .is_focused()
    );
    assert!(screen.query_by_role(Role::Button, "New range").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
