//! Role: Calendar grid parts share day focus, selection, disabled days and month paging.

use crate::support::mount_with;
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange, Locale, Weekday};
use kivact::interaction::{CalendarGrid, CalendarPolicy, CalendarSelection, use_calendar_grid};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen};
use kivact::view::{Modifiers, dynamic};

fn day(y: i32, m: u8, d: u8) -> CalendarDate {
    CalendarDate::new(y, m, d)
}

/// The shown months side by side, then the paging buttons. The days of the
/// neighbouring months are drawn in the first and last week only when
/// `outside` is true (two months side by side would repeat them).
fn drawn(
    calendar: CalendarGrid,
    first: Signal<CalendarMonth>,
    months: usize,
    outside: bool,
) -> Node {
    let panels = dynamic(first, move |first| {
        let week_start = calendar.week_start().get_untracked();
        row((0..months)
            .map(|index| {
                let month = first.add_months(index as i32);
                let weeks = month.weeks(week_start, 0).into_iter().map(|week| {
                    row(week
                        .into_iter()
                        .filter(|date| outside || date.month_of() == month)
                        .map(|date| {
                            node()
                                .part(calendar.cell(month, date))
                                .style(style().size(40, 24))
                                .child(text(date.day().to_string()))
                        })
                        .collect::<Vec<_>>())
                    .part(calendar.row())
                });
                column(weeks.collect::<Vec<_>>())
                    .part(calendar.grid())
                    .name(format!("{}-{:02}", month.year(), month.month()))
            })
            .collect::<Vec<_>>())
    });
    column((
        panels,
        row((
            node()
                .part(calendar.previous_year())
                .name("Previous year")
                .child(text("<<")),
            node()
                .part(calendar.previous_month())
                .name("Previous month")
                .child(text("<")),
            node()
                .part(calendar.next_month())
                .name("Next month")
                .child(text(">")),
            node()
                .part(calendar.next_year())
                .name("Next year")
                .child(text(">>")),
        )),
    ))
}

struct Single {
    value: Signal<Option<CalendarDate>>,
    month: Signal<CalendarMonth>,
    commits: Signal<Vec<CalendarDate>>,
}

fn single(
    selected: Option<CalendarDate>,
    months: usize,
    outside: bool,
    configure: impl FnOnce(CalendarPolicy) -> CalendarPolicy + 'static,
) -> (Screen, Single) {
    mount_with(
        RenderOptions {
            locale: Some(Locale::new("en-US")),
            ..Default::default()
        },
        move || {
            let value = signal(selected);
            let month = signal(selected.unwrap_or(day(2026, 10, 1)).month_of());
            let commits = signal(Vec::<CalendarDate>::new());
            let policy = configure(
                CalendarPolicy::new()
                    .months(months)
                    .week_start(Weekday::Monday)
                    .on_commit(move |date| commits.update(|all| all.push(date))),
            );
            let calendar =
                use_calendar_grid(month, CalendarSelection::Single(value.into()), policy);
            (
                Single {
                    value,
                    month,
                    commits,
                },
                drawn(calendar, month, months, outside),
            )
        },
    )
}

fn cell<'a>(screen: &'a Screen, date: &str) -> kivact::test::Element<'a> {
    screen.get_by_role(Role::GridCell, date)
}

#[test]
fn days_are_cells_named_by_date_and_a_click_chooses_and_commits() {
    let (screen, state) = single(None, 1, true, |policy| policy.disabled(|d| d.day() == 15));
    screen.get_by_role(Role::Grid, "2026-10");
    // The neighbouring months fill the first and last week.
    assert!(screen.query_by_role(Role::GridCell, "2026-09-28").is_some());
    cell(&screen, "2026-10-14").click();
    assert_eq!(state.value.get_untracked(), Some(day(2026, 10, 14)));
    assert!(cell(&screen, "2026-10-14").is_selected());
    assert!(!cell(&screen, "2026-10-13").is_selected());
    // The same day commits again: a caller closes a popover on it.
    cell(&screen, "2026-10-14").click();
    assert_eq!(
        state.commits.get_untracked(),
        vec![day(2026, 10, 14), day(2026, 10, 14)]
    );
    // A disabled day is published disabled and cannot be chosen.
    assert!(cell(&screen, "2026-10-15").is_disabled());
    cell(&screen, "2026-10-15").force_click();
    assert_eq!(state.value.get_untracked(), Some(day(2026, 10, 14)));
    // A day of a neighbouring month can be clicked and shows its month.
    cell(&screen, "2026-09-30").click();
    assert_eq!(state.value.get_untracked(), Some(day(2026, 9, 30)));
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2026, 9));
    screen.get_by_role(Role::Grid, "2026-09");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_selected_day_holds_the_tab_stop_and_arrows_home_and_end_move_it() {
    let (screen, state) = single(Some(day(2026, 10, 14)), 1, true, |policy| policy);
    // Only the selected day is in the tab order.
    screen.press_tab(false);
    assert!(cell(&screen, "2026-10-14").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(cell(&screen, "2026-10-15").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(cell(&screen, "2026-10-22").is_focused());
    screen.press_key(Key::ArrowUp);
    screen.press_key(Key::ArrowLeft);
    assert!(cell(&screen, "2026-10-14").is_focused());
    // The week starts on Monday: 12 to 18 October.
    screen.press_key(Key::Home);
    assert!(cell(&screen, "2026-10-12").is_focused());
    screen.press_key(Key::End);
    assert!(cell(&screen, "2026-10-18").is_focused());
    // Moving the focus does not choose; Enter does.
    assert_eq!(state.value.get_untracked(), Some(day(2026, 10, 14)));
    screen.press_key(Key::Enter);
    assert_eq!(state.value.get_untracked(), Some(day(2026, 10, 18)));
    screen.press_key(Key::ArrowLeft);
    screen.press_key(Key::Space);
    assert_eq!(state.value.get_untracked(), Some(day(2026, 10, 17)));
    assert_eq!(
        state.commits.get_untracked(),
        vec![day(2026, 10, 18), day(2026, 10, 17)]
    );
    // Control with an arrow is not the calendar's.
    screen.press_key_with(
        Key::End,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(cell(&screen, "2026-10-17").is_focused());
    // Tab leaves the grid; there is one stop in it.
    screen.press_tab(false);
    assert!(!cell(&screen, "2026-10-17").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A range calendar over October 2026 starting from `initial`.
fn range(initial: Option<DateRange>) -> (Screen, Signal<Option<DateRange>>) {
    mount_with(
        RenderOptions {
            locale: Some(Locale::new("en-US")),
            ..Default::default()
        },
        move || {
            let value = signal(initial);
            let month = signal(day(2026, 10, 1).month_of());
            let calendar = use_calendar_grid(
                month,
                CalendarSelection::Range(value.into()),
                CalendarPolicy::new().week_start(Weekday::Monday),
            );
            (value, drawn(calendar, month, 1, true))
        },
    )
}

#[test]
fn a_write_to_the_range_from_elsewhere_starts_the_next_choice_over() {
    let (screen, value) = range(None);
    cell(&screen, "2026-10-05").click();
    assert_eq!(
        value.get_untracked(),
        Some(DateRange::new(day(2026, 10, 5), day(2026, 10, 5)))
    );
    value.set(Some(DateRange::new(day(2026, 10, 20), day(2026, 10, 22))));
    screen.run_until_idle();
    cell(&screen, "2026-10-08").click();
    assert_eq!(
        value.get_untracked(),
        Some(DateRange::new(day(2026, 10, 8), day(2026, 10, 8))),
        "the stale first choice is not completed"
    );
    cell(&screen, "2026-10-09").click();
    assert_eq!(
        value.get_untracked(),
        Some(DateRange::new(day(2026, 10, 8), day(2026, 10, 9)))
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn escape_during_a_range_choice_puts_back_the_range_from_before_it() {
    let before = DateRange::new(day(2026, 10, 1), day(2026, 10, 3));
    let (screen, value) = range(Some(before));
    cell(&screen, "2026-10-14").click();
    assert_eq!(
        value.get_untracked(),
        Some(DateRange::new(day(2026, 10, 14), day(2026, 10, 14)))
    );
    cell(&screen, "2026-10-14").focus();
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), Some(before));
    cell(&screen, "2026-10-20").click();
    assert_eq!(
        value.get_untracked(),
        Some(DateRange::new(day(2026, 10, 20), day(2026, 10, 20))),
        "the next choice starts a range again"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
