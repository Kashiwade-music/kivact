//! Role: Verify keyboard selection and date-range staging through the public Soft controls.
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact::signal;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, render_with_handle};
use kivact::view::{Key, Role};

fn options() -> RenderOptions {
    RenderOptions {
        theme: kivact_ui_soft::theme(ThemeMode::Light),
        ..RenderOptions::from_env()
    }
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
    screen.press_key(Key::PageUp);
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some(date(18)));
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
    screen.press_key(Key::Escape);
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
        (selected, kivact_ui_soft::date_input("Due", selected))
    });
    screen
        .get_by_role(Role::TextBox, "Due")
        .type_text("2026-10-14");
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some(date(14)));
    screen.press_key(Key::ArrowDown);
    screen.get_by_role(Role::GridCell, "2026-10-15");
    assert!(screen.get_by_role(Role::TextBox, "Due").is_focused());
    screen.get_by_role(Role::Button, "Choose date").click();
    screen.get_by_role(Role::Button, "Choose date").click();
    screen.get_by_role(Role::GridCell, "2026-10-15").click();
    assert_eq!(selected.get_untracked(), Some(date(15)));
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
