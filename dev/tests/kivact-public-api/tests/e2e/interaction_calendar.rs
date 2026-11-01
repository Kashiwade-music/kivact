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

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Default::default()
    }
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

#[test]
fn moving_past_a_month_edge_pages_the_month_and_keeps_the_focus() {
    let (screen, state) = single(Some(day(2026, 10, 31)), 1, false, |policy| policy);
    screen.press_tab(false);
    assert!(cell(&screen, "2026-10-31").is_focused());
    screen.press_key(Key::ArrowRight);
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2026, 11));
    screen.get_by_role(Role::Grid, "2026-11");
    assert!(cell(&screen, "2026-11-01").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2026, 10));
    assert!(cell(&screen, "2026-10-31").is_focused());
    // PageDown keeps the day, or clamps to the end of a shorter month.
    screen.press_key(Key::PageDown);
    assert!(cell(&screen, "2026-11-30").is_focused());
    screen.press_key(Key::PageUp);
    assert!(cell(&screen, "2026-10-30").is_focused());
    // Shift with the page keys moves a year.
    screen.press_key_with(Key::PageDown, shift());
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2027, 10));
    assert!(cell(&screen, "2027-10-30").is_focused());
    screen.press_key_with(Key::PageUp, shift());
    assert!(cell(&screen, "2026-10-30").is_focused());
    // February of a leap year.
    screen.press_key_with(Key::PageUp, shift());
    assert!(cell(&screen, "2025-10-30").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn moves_skip_disabled_days_and_a_disabled_day_takes_no_focus() {
    let (screen, state) = single(Some(day(2026, 10, 16)), 1, false, |policy| {
        policy.disabled(|d| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
    });
    screen.press_tab(false);
    assert!(cell(&screen, "2026-10-16").is_focused());
    // Friday, then the weekend is skipped.
    screen.press_key(Key::ArrowRight);
    assert!(cell(&screen, "2026-10-19").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(cell(&screen, "2026-10-16").is_focused());
    // End of the week lands on the last enabled day, not the Sunday.
    screen.press_key(Key::End);
    assert!(cell(&screen, "2026-10-16").is_focused());
    screen.press_key(Key::Home);
    assert!(cell(&screen, "2026-10-12").is_focused());
    // A day a week down that is disabled moves on in the direction of the key.
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(cell(&screen, "2026-10-26").is_focused());
    // A disabled day does not take a press or Enter.
    cell(&screen, "2026-10-17").force_click();
    assert_eq!(state.value.get_untracked(), Some(day(2026, 10, 16)));
    assert!(state.commits.get_untracked().is_empty());
    assert!(cell(&screen, "2026-10-17").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn without_a_selection_the_initial_focus_or_the_first_enabled_day_holds_the_tab_stop() {
    let (screen, _) = single(None, 1, false, |policy| {
        policy.initial_focus(day(2026, 10, 20))
    });
    screen.press_tab(false);
    assert!(cell(&screen, "2026-10-20").is_focused());
    drop(screen);
    // An initial focus that is not shown, or disabled, is not used.
    let (screen, _) = single(None, 1, false, |policy| {
        policy
            .initial_focus(day(2026, 12, 20))
            .disabled(|d| d.day() < 3)
    });
    screen.press_tab(false);
    assert!(cell(&screen, "2026-10-03").is_focused());
}

#[test]
fn month_buttons_page_the_shown_months_and_the_tab_stop_falls_to_the_first_enabled_day() {
    let (screen, state) = single(Some(day(2026, 10, 14)), 1, false, |policy| policy);
    screen.get_by_role(Role::Button, "Next month").click();
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2026, 11));
    screen.get_by_role(Role::Grid, "2026-11");
    assert_eq!(
        state.value.get_untracked(),
        Some(day(2026, 10, 14)),
        "paging does not choose"
    );
    // The selected day is not shown: the grid still has a stop, behind the
    // three buttons before "Next month".
    for _ in 0..3 {
        screen.press_tab(true);
    }
    assert!(cell(&screen, "2026-11-01").is_focused());
    screen.get_by_role(Role::Button, "Previous year").click();
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2025, 11));
    screen.get_by_role(Role::Button, "Next year").click();
    screen.get_by_role(Role::Button, "Next year").click();
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2027, 11));
    screen.get_by_role(Role::Button, "Previous month").click();
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2027, 10));
    // The month binding is the caller's: writing it pages the grid.
    state.month.set(CalendarMonth::new(2026, 10));
    screen.run_until_idle();
    assert!(cell(&screen, "2026-10-14").is_selected());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn several_months_page_when_the_focus_leaves_them_and_not_before() {
    let (screen, state) = single(Some(day(2026, 10, 14)), 2, false, |policy| policy);
    screen.get_by_role(Role::Grid, "2026-10");
    screen.get_by_role(Role::Grid, "2026-11");
    screen.press_tab(false);
    assert!(cell(&screen, "2026-10-14").is_focused());
    screen.press_key(Key::PageDown);
    assert!(cell(&screen, "2026-11-14").is_focused());
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2026, 10));
    screen.press_key(Key::PageDown);
    assert_eq!(
        state.month.get_untracked(),
        CalendarMonth::new(2026, 11),
        "the last shown month"
    );
    assert!(cell(&screen, "2026-12-14").is_focused());
    screen.press_key(Key::PageUp);
    screen.press_key(Key::PageUp);
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2026, 10));
    assert!(cell(&screen, "2026-10-14").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn selecting_on_move_chooses_the_day_the_arrow_reaches_without_a_commit() {
    let (screen, state) = single(Some(day(2026, 10, 14)), 1, false, |policy| {
        policy.select_on_move(true)
    });
    screen.press_tab(false);
    screen.press_key(Key::ArrowRight);
    assert_eq!(state.value.get_untracked(), Some(day(2026, 10, 15)));
    assert!(cell(&screen, "2026-10-15").is_selected());
    screen.press_key(Key::PageDown);
    assert_eq!(state.value.get_untracked(), Some(day(2026, 11, 15)));
    assert_eq!(state.month.get_untracked(), CalendarMonth::new(2026, 11));
    assert!(state.commits.get_untracked().is_empty());
    cell(&screen, "2026-11-16").click();
    assert_eq!(state.commits.get_untracked(), vec![day(2026, 11, 16)]);
}

#[test]
fn a_range_is_two_choices_in_either_order_and_every_day_in_it_is_selected() {
    let (screen, (range, month, commits)) = mount_with(
        RenderOptions {
            locale: Some(Locale::new("en-US")),
            ..Default::default()
        },
        || {
            let range = signal(None::<DateRange>);
            let month = signal(CalendarMonth::new(2026, 10));
            let commits = signal(Vec::<CalendarDate>::new());
            let calendar = use_calendar_grid(
                month,
                CalendarSelection::Range(range.into()),
                CalendarPolicy::new()
                    .week_start(Weekday::Monday)
                    .disabled(|d| d.day() == 7)
                    .on_commit(move |date| commits.update(|all| all.push(date))),
            );
            ((range, month, commits), drawn(calendar, month, 1, false))
        },
    );
    assert!(
        screen
            .get_by_role(Role::Grid, "2026-10")
            .is_multiselectable()
    );
    cell(&screen, "2026-10-09").click();
    assert_eq!(
        range.get_untracked(),
        Some(DateRange::new(day(2026, 10, 9), day(2026, 10, 9)))
    );
    assert!(
        commits.get_untracked().is_empty(),
        "the second choice completes it"
    );
    cell(&screen, "2026-10-05").click();
    assert_eq!(
        range.get_untracked(),
        Some(DateRange::new(day(2026, 10, 5), day(2026, 10, 9)))
    );
    assert_eq!(commits.get_untracked(), vec![day(2026, 10, 5)]);
    for selected in ["05", "06", "07", "08", "09"] {
        assert!(
            cell(&screen, &format!("2026-10-{selected}")).is_selected(),
            "{selected}"
        );
    }
    assert!(!cell(&screen, "2026-10-04").is_selected());
    assert!(!cell(&screen, "2026-10-10").is_selected());
    // A disabled day inside the range is part of it, and cannot end one.
    cell(&screen, "2026-10-20").click();
    cell(&screen, "2026-10-07").force_click();
    assert_eq!(
        range.get_untracked(),
        Some(DateRange::new(day(2026, 10, 20), day(2026, 10, 20)))
    );
    cell(&screen, "2026-10-13").click();
    assert_eq!(
        range.get_untracked(),
        Some(DateRange::new(day(2026, 10, 13), day(2026, 10, 20)))
    );
    assert_eq!(month.get_untracked(), CalendarMonth::new(2026, 10));
    // The range's start holds the tab stop.
    screen.press_tab(false);
    assert!(cell(&screen, "2026-10-13").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_week_starts_on_the_locale_unless_the_policy_says_otherwise() {
    let (screen, ()) = mount_with(
        RenderOptions {
            locale: Some(Locale::new("ja-JP")),
            ..Default::default()
        },
        || {
            let month = signal(CalendarMonth::new(2026, 10));
            let calendar = use_calendar_grid(
                month,
                CalendarSelection::Single(signal(None).into()),
                CalendarPolicy::new(),
            );
            ((), drawn(calendar, month, 1, true))
        },
    );
    // Sunday first in Japan: 4 October starts the second week.
    let x = |name: &str| cell(&screen, name).rect().expect("laid out").x;
    assert!(x("2026-10-04") < x("2026-10-05"));
    assert!(x("2026-10-04") < x("2026-10-10"));
    assert_eq!(x("2026-10-04"), x("2026-10-11"));
    // Home goes to that Sunday.
    // A chosen day holds the tab stop.
    cell(&screen, "2026-10-08").click();
    screen.press_tab(false);
    assert!(cell(&screen, "2026-10-08").is_focused());
    screen.press_key(Key::Home);
    assert!(cell(&screen, "2026-10-04").is_focused());
    screen.press_key(Key::End);
    assert!(cell(&screen, "2026-10-10").is_focused());
}
