//! Role: The calendar, the date input and the date range input.

use super::*;
use kivact::i18n::{CalendarDate, Locale, Messages, Weekday};

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day)
}

fn mount_in(
    locale: &str,
    setup: impl FnOnce() -> (Signal<CalendarDate>, ui::Calendar) + 'static,
) -> (Screen, Signal<CalendarDate>) {
    kivact::test::render_with_handle(
        RenderOptions {
            theme: ui::theme(kivact::style::ThemeMode::Light),
            locale: Some(Locale::new(locale)),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        setup,
    )
}

/// The weekday headings, left to right.
fn heading_order(screen: &Screen) -> Vec<&'static str> {
    let mut found = DAYS
        .iter()
        .map(|day| {
            (
                screen
                    .get_by_role(Role::Text, day)
                    .rect()
                    .expect("laid out")
                    .x,
                *day,
            )
        })
        .collect::<Vec<_>>();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, day)| day).collect()
}

/// The left edge of a day's cell.
fn column_x(screen: &Screen, name: &str) -> f32 {
    screen
        .get_by_role(Role::GridCell, name)
        .rect()
        .expect("laid out")
        .x
}

/// How many pixels of a day's cell lean red and how many lean blue: the
/// colors of its number on a Sunday and on a Saturday.
fn tint(screen: &Screen, name: &str) -> (usize, usize) {
    let rect = screen
        .get_by_role(Role::GridCell, name)
        .rect()
        .expect("laid out");
    let shot = screen.screenshot();
    let (mut red, mut blue) = (0, 0);
    for y in rect.y as u32..(rect.y + rect.height) as u32 {
        for x in rect.x as u32..(rect.x + rect.width) as u32 {
            let pixel = shot.pixel(x, y).expect("in the frame");
            let [r, g, b] = [16, 8, 0].map(|shift| ((pixel >> shift) & 0xff) as i32);
            if r > g + 60 && r > b + 60 {
                red += 1;
            }
            if b > r + 60 && b > g + 20 {
                blue += 1;
            }
        }
    }
    (red, blue)
}

fn select_all(input: &kivact::test::Element<'_>) {
    input.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
}

#[test]
fn the_week_starts_on_the_locale_s_first_day_unless_the_app_says_otherwise() {
    let make = || {
        let selected = signal(date(2026, 10, 14));
        (selected, ui::calendar(selected, date(2026, 10, 8)))
    };
    let (screen, _) = mount_in("ja-JP", make);
    assert_eq!(heading_order(&screen), DAYS);
    // October 2026 starts on a Thursday: the grid starts on the Sunday before.
    assert!(screen.query_by_role(Role::GridCell, "2026-09-27").is_some());
    assert!(screen.query_by_role(Role::GridCell, "2026-09-26").is_none());
    assert_eq!(
        column_x(&screen, "2026-09-27"),
        screen.get_by_role(Role::Text, "Sun").rect().unwrap().x
    );

    let (screen, _) = mount_in("en-GB", make);
    assert_eq!(
        heading_order(&screen),
        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    );
    assert!(screen.query_by_role(Role::GridCell, "2026-09-28").is_some());
    assert!(screen.query_by_role(Role::GridCell, "2026-09-27").is_none());
    assert_eq!(
        column_x(&screen, "2026-09-28"),
        screen.get_by_role(Role::Text, "Mon").rect().unwrap().x
    );

    let (screen, _) = mount_in("ja-JP", || {
        let selected = signal(date(2026, 10, 14));
        (
            selected,
            ui::calendar(selected, date(2026, 10, 8)).week_starts_on(Weekday::Saturday),
        )
    });
    assert_eq!(
        heading_order(&screen),
        ["Sat", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri"]
    );
    assert!(screen.query_by_role(Role::GridCell, "2026-09-26").is_some());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_week_start_follows_a_signal_and_the_headings_use_the_apps_messages() {
    let screen = render_with(
        RenderOptions {
            theme: ui::theme(kivact::style::ThemeMode::Light),
            messages: Messages::new()
                .with("pop.calendar.mon", "月")
                .with("pop.calendar.sun", "日"),
            ..RenderOptions::from_env()
        },
        || {
            let selected = signal(date(2026, 10, 14));
            let first = signal(Weekday::Sunday);
            column((
                ui::calendar(selected, date(2026, 10, 8)).week_starts_on(first),
                kivact::ui::button("Monday").on_press(move || first.set(Weekday::Monday)),
            ))
        },
    );
    assert!(screen.query_by_role(Role::GridCell, "2026-09-27").is_some());
    screen.get_by_role(Role::Text, "日");
    screen.get_by_role(Role::Button, "Monday").click();
    assert!(screen.query_by_role(Role::GridCell, "2026-09-27").is_none());
    assert!(screen.query_by_role(Role::GridCell, "2026-09-28").is_some());
    let (mon, sun) = (
        screen.get_by_role(Role::Text, "月").rect().unwrap().x,
        screen.get_by_role(Role::Text, "日").rect().unwrap().x,
    );
    assert!(mon < sun, "Monday is now first");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn weekend_colors_follow_the_weekday_not_the_column() {
    for first in [Weekday::Sunday, Weekday::Monday] {
        let (screen, _) = mount_in("en-US", move || {
            let selected = signal(date(2026, 10, 20));
            (
                selected,
                ui::calendar(selected, date(2026, 10, 21))
                    .week_starts_on(first)
                    .weekend(|day| match day {
                        Weekday::Sunday => Some(ui::Hue::Red),
                        Weekday::Saturday => Some(ui::Hue::Blue),
                        _ => None,
                    }),
            )
        });
        // 2026-10-10 is a Saturday, 2026-10-11 a Sunday and 2026-10-12 a Monday.
        let (saturday, sunday, monday) = (
            tint(&screen, "2026-10-10"),
            tint(&screen, "2026-10-11"),
            tint(&screen, "2026-10-12"),
        );
        assert!(saturday.1 > 0 && saturday.0 == 0, "{first:?}: {saturday:?}");
        assert!(sunday.0 > 0 && sunday.1 == 0, "{first:?}: {sunday:?}");
        assert!(monday == (0, 0), "{first:?}: {monday:?}");
    }
}

#[test]
fn year_buttons_are_off_by_default_and_move_the_shown_year() {
    let (screen, selected) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(date(2026, 10, 3));
        (selected, ui::calendar(selected, date(2026, 10, 3)))
    });
    screen.get_by_role(Role::Text, "2026-10");
    assert!(
        screen
            .query_by_role(Role::Button, "Previous year")
            .is_none()
    );
    assert!(screen.query_by_role(Role::Button, "Next year").is_none());
    assert_eq!(selected.get_untracked(), date(2026, 10, 3));

    let (screen, selected) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(date(2026, 10, 3));
        (
            selected,
            ui::calendar(selected, date(2026, 10, 3)).year_navigation(true),
        )
    });
    screen.get_by_role(Role::Button, "Previous year").click();
    screen.get_by_role(Role::Text, "2025-10");
    screen.get_by_role(Role::GridCell, "2025-10-12");
    screen.get_by_role(Role::Button, "Next year").click();
    screen.get_by_role(Role::Button, "Next year").click();
    screen.get_by_role(Role::Text, "2027-10");
    screen.get_by_role(Role::Button, "Previous month").click();
    screen.get_by_role(Role::Text, "2027-09");
    // The buttons look at the page only: the selection stays.
    assert_eq!(selected.get_untracked(), date(2026, 10, 3));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_month_title_can_be_written_by_the_app() {
    let (screen, _) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(date(2026, 10, 3));
        (
            selected,
            ui::calendar(selected, date(2026, 10, 3))
                .month_title(|month| format!("{} / {}", month.year(), month.month())),
        )
    });
    screen.get_by_role(Role::Text, "2026 / 10");
    screen.get_by_role(Role::Button, "Next month").click();
    screen.get_by_role(Role::Text, "2026 / 11");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn calendar_arrows_keep_focus_and_cross_month_boundaries() {
    let (screen, selected) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(date(2026, 10, 30));
        (selected, ui::calendar(selected, date(2026, 10, 3)))
    });
    screen.get_by_role(Role::GridCell, "2026-10-30").click();
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), date(2026, 10, 31));
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-31")
            .is_focused()
    );
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), date(2026, 11, 1));
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-11-01")
            .is_focused()
    );
    screen.press_key(Key::ArrowLeft);
    assert_eq!(selected.get_untracked(), date(2026, 10, 31));
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-31")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn shift_with_page_keys_moves_a_year_and_without_it_a_month() {
    let (screen, selected) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(date(2026, 10, 30));
        (selected, ui::calendar(selected, date(2026, 10, 3)))
    });
    screen.get_by_role(Role::GridCell, "2026-10-30").click();
    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };
    screen.press_key_with(Key::PageUp, shift);
    assert_eq!(selected.get_untracked(), date(2025, 10, 30));
    screen.get_by_role(Role::Text, "2025-10");
    assert!(
        screen
            .get_by_role(Role::GridCell, "2025-10-30")
            .is_focused()
    );
    screen.press_key_with(Key::PageDown, shift);
    screen.press_key_with(Key::PageDown, shift);
    assert_eq!(selected.get_untracked(), date(2027, 10, 30));
    screen.press_key(Key::PageUp);
    assert_eq!(selected.get_untracked(), date(2027, 9, 30));
    // The day clamps to the end of a shorter month.
    screen.press_key(Key::PageUp);
    assert_eq!(selected.get_untracked(), date(2027, 8, 30));
    screen.press_key_with(Key::PageUp, shift);
    assert_eq!(selected.get_untracked(), date(2026, 8, 30));
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-08-30")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn home_and_end_go_to_the_ends_of_the_week_that_is_shown() {
    for (first, home, end) in [
        (Weekday::Sunday, date(2026, 10, 11), date(2026, 10, 17)),
        (Weekday::Monday, date(2026, 10, 12), date(2026, 10, 18)),
    ] {
        let (screen, selected) = kivact::test::render_with_handle(crate::options(), move || {
            let selected = signal(date(2026, 10, 14));
            (
                selected,
                ui::calendar(selected, date(2026, 10, 3)).week_starts_on(first),
            )
        });
        screen.get_by_role(Role::GridCell, "2026-10-14").click();
        screen.press_key(Key::Home);
        assert_eq!(selected.get_untracked(), home, "{first:?}");
        assert!(
            screen
                .get_by_role(Role::GridCell, "2026-10-11")
                .is_selected()
                == (home == date(2026, 10, 11))
        );
        screen.press_key(Key::End);
        assert_eq!(selected.get_untracked(), end, "{first:?}");
        assert!(
            screen
                .get_by_role(Role::GridCell, &format!("2026-10-{}", end.day()))
                .is_focused()
        );
    }
    // A week that crosses a month moves the page with it.
    let (screen, selected) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(date(2026, 10, 1));
        (
            selected,
            ui::calendar(selected, date(2026, 10, 3)).week_starts_on(Weekday::Sunday),
        )
    });
    screen.get_by_role(Role::GridCell, "2026-10-01").click();
    screen.press_key(Key::Home);
    assert_eq!(selected.get_untracked(), date(2026, 9, 27));
    screen.get_by_role(Role::Text, "2026-09");
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-09-27")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_range_marks_every_day_in_it_and_not_by_color_alone() {
    let (screen, (selected, range)) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(date(2026, 10, 12));
        let range = signal(Some((date(2026, 10, 10), date(2026, 10, 14))));
        (
            (selected, range),
            ui::calendar(selected, date(2026, 10, 3)).range(range),
        )
    });
    let selected_days = || {
        (1..=31)
            .filter(|day| {
                screen
                    .get_by_role(Role::GridCell, &format!("2026-10-{day:02}"))
                    .is_selected()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(selected_days(), [10, 11, 12, 13, 14]);
    range.set(Some((date(2026, 10, 20), date(2026, 10, 20))));
    screen.run_until_idle();
    // Out of the range, the day that is selected still is.
    assert_eq!(selected_days(), [12, 20]);
    range.set(None);
    screen.run_until_idle();
    assert_eq!(selected_days(), [12]);
    selected.set(date(2026, 10, 13));
    screen.run_until_idle();
    assert_eq!(selected_days(), [13]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_activation_is_reported_also_for_the_day_that_is_selected() {
    let (screen, (selected, activated)) =
        kivact::test::render_with_handle(crate::options(), || {
            let selected = signal(date(2026, 10, 3));
            let activated = signal(Vec::<CalendarDate>::new());
            (
                (selected, activated),
                ui::calendar(selected, date(2026, 10, 3))
                    .on_activate(move |day: CalendarDate| activated.update(|days| days.push(day))),
            )
        });
    screen.get_by_role(Role::GridCell, "2026-10-03").click();
    screen.get_by_role(Role::GridCell, "2026-10-17").click();
    screen
        .get_by_role(Role::GridCell, "2026-10-17")
        .press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), date(2026, 10, 18));
    assert_eq!(
        activated.get_untracked(),
        [date(2026, 10, 3), date(2026, 10, 17)],
        "arrow keys select without activating"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_date_picker_writes_the_binding_and_closes() {
    let (screen, date_value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(date(2026, 10, 3));
        (value, ui::date_input("Date", value, date(2026, 10, 3)))
    });
    let opener = screen.get_by_role(Role::Button, "Choose a date");
    assert_eq!(opener.is_expanded(), Some(false));
    opener.click();
    assert_eq!(opener.is_expanded(), Some(true));
    screen.get_by_role(Role::GridCell, "2026-10-17").click();
    assert_eq!(date_value.get_untracked(), date(2026, 10, 17));
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Date").value(),
        Some("2026-10-17".to_string())
    );
    assert_eq!(opener.is_expanded(), Some(false));
    opener.click();
    screen.get_by_role(Role::GridCell, "2026-10-17").click();
    assert_eq!(opener.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_date_input_configures_the_calendar_it_opens() {
    let (screen, _) = kivact::test::render_with_handle(
        RenderOptions {
            theme: ui::theme(kivact::style::ThemeMode::Light),
            locale: Some(Locale::new("ja-JP")),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let value = signal(date(2026, 10, 3));
            (
                value,
                ui::date_input("Date", value, date(2026, 10, 3)).calendar(|calendar| {
                    calendar
                        .week_starts_on(Weekday::Monday)
                        .year_navigation(true)
                }),
            )
        },
    );
    screen.get_by_role(Role::Button, "Choose a date").click();
    assert_eq!(heading_order(&screen)[0], "Mon");
    screen.get_by_role(Role::Button, "Next year").click();
    screen.get_by_role(Role::GridCell, "2027-10-12").click();
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Date").value().as_deref(),
        Some("2027-10-12")
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn date_input_preserves_partial_edits_until_a_valid_date_is_entered() {
    let (screen, (value, valid)) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(date(2026, 10, 3));
        let valid = signal(true);
        (
            (value, valid),
            ui::date_input("Date", value, date(2026, 10, 8)).valid(valid),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Date");
    assert_eq!(input.value().as_deref(), Some("2026-10-03"));
    assert!(valid.get_untracked());
    select_all(&input);
    for draft in [
        "2",
        "20",
        "202",
        "2027",
        "2027-",
        "2027-1",
        "2027-12",
        "2027-12-",
        "2027-12-2",
    ] {
        input.type_text(&draft[draft.len() - 1..]);
        assert_eq!(input.value().as_deref(), Some(draft));
        // Not a date yet: nothing is written and nothing is said.
        assert_eq!(value.get_untracked(), date(2026, 10, 3));
        assert!(!valid.get_untracked(), "{draft}");
        assert!(!input.invalid(), "{draft}");
        assert!(
            screen
                .query_by_role(Role::Status, "Use YYYY-MM-DD")
                .is_none()
        );
    }
    input.type_text("0");
    assert_eq!(input.value().as_deref(), Some("2027-12-20"));
    assert_eq!(value.get_untracked(), date(2027, 12, 20));
    assert!(valid.get_untracked());

    // Left as the start of a date, the text is told to be wrong.
    select_all(&input);
    input.type_text("2027-12-2");
    assert!(!input.invalid());
    screen.press_tab(false);
    assert!(input.invalid());
    screen.get_by_role(Role::Status, "Use YYYY-MM-DD");
    input.type_text("0");
    assert!(!input.invalid());
    assert!(
        screen
            .query_by_role(Role::Status, "Use YYYY-MM-DD")
            .is_none()
    );
    // So is it when Enter is pressed.
    select_all(&input);
    input.type_text("2027-1");
    input.press_key(Key::Enter);
    assert!(input.invalid());
    screen.get_by_role(Role::Status, "Use YYYY-MM-DD");

    select_all(&input);
    input.type_text("invalid");
    assert_eq!(input.value().as_deref(), Some("invalid"));
    screen.get_by_role(Role::Button, "Choose a date").click();
    screen.get_by_role(Role::GridCell, "2027-12-02").click();
    assert_eq!(value.get_untracked(), date(2027, 12, 2));
    assert_eq!(input.value().as_deref(), Some("2027-12-02"));
    assert!(valid.get_untracked());
    select_all(&input);
    input.type_text("invalid");
    assert!(!valid.get_untracked());
    value.set(date(2028, 1, 5));
    screen.run_until_idle();
    assert_eq!(input.value().as_deref(), Some("2028-01-05"));
    assert!(valid.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_date_with_either_separator_is_accepted_and_shown_with_dashes() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(date(2026, 10, 3));
        (value, ui::date_input("Date", value, date(2026, 10, 8)))
    });
    let input = screen.get_by_role(Role::TextBox, "Date");
    select_all(&input);
    input.type_text("2027/01/9");
    assert_eq!(value.get_untracked(), date(2026, 10, 3), "no leading zero");
    input.press_key(Key::Backspace);
    input.type_text("09");
    assert_eq!(value.get_untracked(), date(2027, 1, 9));
    assert_eq!(input.value().as_deref(), Some("2027/01/09"));
    value.set(date(2027, 2, 1));
    screen.run_until_idle();
    assert_eq!(input.value().as_deref(), Some("2027-02-01"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_wrong_date_is_explained_and_marks_the_field_until_it_is_repaired() {
    let (screen, (value, valid)) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(date(2026, 10, 3));
        let valid = signal(true);
        (
            (value, valid),
            ui::date_input("Date", value, date(2026, 10, 8)).valid(valid),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Date");
    assert!(!input.invalid());
    assert!(input.described_by().is_empty());
    assert!(
        screen
            .query_by_role(Role::Status, "Use YYYY-MM-DD")
            .is_none()
    );

    select_all(&input);
    input.type_text("bad");
    assert!(!valid.get_untracked());
    assert!(input.invalid());
    let status = screen.get_by_role(Role::Status, "Use YYYY-MM-DD");
    let described = input.described_by();
    assert_eq!(described.len(), 1);
    assert_eq!(described[0].name(), status.name());

    select_all(&input);
    input.type_text("2026-02-30");
    assert!(!valid.get_untracked());
    assert!(input.invalid());
    screen.get_by_role(Role::Status, "This date does not exist");
    assert!(
        screen
            .query_by_role(Role::Status, "Use YYYY-MM-DD")
            .is_none()
    );
    assert_eq!(input.described_by().len(), 1);
    assert_eq!(value.get_untracked(), date(2026, 10, 3));

    // A text that can no longer become a date is told at once.
    select_all(&input);
    input.type_text("2026-13");
    assert!(input.invalid());
    screen.get_by_role(Role::Status, "Use YYYY-MM-DD");

    select_all(&input);
    input.press_key(Key::Backspace);
    assert_eq!(input.value().as_deref(), Some(""));
    assert!(!valid.get_untracked(), "empty is not a date");
    assert!(!input.invalid(), "empty may still become one");
    input.press_key(Key::Enter);
    screen.get_by_role(Role::Status, "Use YYYY-MM-DD");

    select_all(&input);
    input.type_text("2026-10-12");
    assert!(valid.get_untracked());
    assert!(!input.invalid());
    assert!(input.described_by().is_empty());
    assert!(
        screen
            .query_by_role(Role::Status, "Use YYYY-MM-DD")
            .is_none()
    );
    assert_eq!(value.get_untracked(), date(2026, 10, 12));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_error_texts_are_the_apps_messages() {
    let screen = render_with(
        RenderOptions {
            theme: ui::theme(kivact::style::ThemeMode::Light),
            messages: Messages::new().with("pop.date.error.format", "形式が違います"),
            ..RenderOptions::from_env()
        },
        || ui::date_input("Date", signal(date(2026, 10, 3)), date(2026, 10, 3)),
    );
    let input = screen.get_by_role(Role::TextBox, "Date");
    select_all(&input);
    input.type_text("x");
    screen.get_by_role(Role::Status, "形式が違います");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The start, the end and the validity an app of one range input holds.
type RangeState = (Signal<CalendarDate>, Signal<CalendarDate>, Signal<bool>);

fn range_app() -> (RangeState, ui::DateRangeInput) {
    let start = signal(date(2026, 10, 3));
    let end = signal(date(2026, 10, 5));
    let valid = signal(true);
    (
        (start, end, valid),
        ui::date_range_input(("Start", "End"), (start, end), date(2026, 10, 8)).valid(valid),
    )
}

fn field_text(screen: &Screen, label: &str) -> Option<String> {
    screen.get_by_role(Role::TextBox, label).value()
}

#[test]
fn two_choices_in_the_calendar_set_the_start_and_then_the_end() {
    let (screen, (start, end, valid)) =
        kivact::test::render_with_handle(crate::options(), range_app);
    let picker = screen.get_by_role(Role::Button, "Choose a date");
    picker.click();
    // The calendar opens with the range that is there.
    for (day, inside) in [(2, false), (3, true), (4, true), (5, true), (6, false)] {
        assert_eq!(
            screen
                .get_by_role(Role::GridCell, &format!("2026-10-{day:02}"))
                .is_selected(),
            inside,
            "{day}"
        );
    }
    screen.get_by_role(Role::GridCell, "2026-10-10").click();
    assert_eq!(
        (start.get_untracked(), end.get_untracked()),
        (date(2026, 10, 10), date(2026, 10, 10))
    );
    assert_eq!(picker.is_expanded(), Some(true), "waiting for the end");
    assert!(
        !screen
            .get_by_role(Role::GridCell, "2026-10-11")
            .is_selected()
    );
    assert_eq!(field_text(&screen, "Start").as_deref(), Some("2026-10-10"));
    assert_eq!(field_text(&screen, "End").as_deref(), Some("2026-10-10"));
    screen.get_by_role(Role::GridCell, "2026-10-14").click();
    assert_eq!(
        (start.get_untracked(), end.get_untracked()),
        (date(2026, 10, 10), date(2026, 10, 14))
    );
    assert_eq!(picker.is_expanded(), Some(false));
    assert_eq!(field_text(&screen, "End").as_deref(), Some("2026-10-14"));
    assert!(valid.get_untracked());

    // The next two choices start over, and an end before the start swaps them.
    picker.click();
    screen.get_by_role(Role::GridCell, "2026-10-20").click();
    assert_eq!(
        (start.get_untracked(), end.get_untracked()),
        (date(2026, 10, 20), date(2026, 10, 20))
    );
    screen.get_by_role(Role::GridCell, "2026-10-06").click();
    assert_eq!(
        (start.get_untracked(), end.get_untracked()),
        (date(2026, 10, 6), date(2026, 10, 20))
    );
    assert_eq!(picker.is_expanded(), Some(false));
    assert_eq!(field_text(&screen, "Start").as_deref(), Some("2026-10-06"));
    assert_eq!(field_text(&screen, "End").as_deref(), Some("2026-10-20"));
    assert!(valid.get_untracked());

    // Closing in the middle forgets the first choice.
    picker.click();
    screen.get_by_role(Role::GridCell, "2026-10-09").click();
    screen.press_key(Key::Escape);
    assert_eq!(picker.is_expanded(), Some(false));
    picker.click();
    screen.get_by_role(Role::GridCell, "2026-10-15").click();
    assert_eq!(picker.is_expanded(), Some(true), "a first choice again");
    assert_eq!(
        (start.get_untracked(), end.get_untracked()),
        (date(2026, 10, 15), date(2026, 10, 15))
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_start_after_the_end_makes_both_fields_invalid_with_one_message() {
    let (screen, (start, end, valid)) =
        kivact::test::render_with_handle(crate::options(), range_app);
    let (from, to) = (
        screen.get_by_role(Role::TextBox, "Start"),
        screen.get_by_role(Role::TextBox, "End"),
    );
    assert!(!from.invalid() && !to.invalid());
    select_all(&to);
    to.type_text("2026-10-01");
    assert_eq!(end.get_untracked(), date(2026, 10, 1));
    assert_eq!(start.get_untracked(), date(2026, 10, 3));
    assert!(!valid.get_untracked());
    assert!(from.invalid() && to.invalid());
    let order = screen.get_by_role(Role::Status, "The end date is before the start date");
    for field in [&from, &to] {
        let described = field.described_by();
        assert_eq!(described.len(), 1);
        assert_eq!(described[0].name(), order.name());
    }
    assert!(
        screen
            .query_by_role(Role::Status, "Use YYYY-MM-DD")
            .is_none()
    );

    select_all(&from);
    from.type_text("2026-09-30");
    assert!(valid.get_untracked());
    assert!(!from.invalid() && !to.invalid());
    assert!(
        screen
            .query_by_role(Role::Status, "The end date is before the start date")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_wrong_text_in_one_field_marks_only_that_field() {
    let (screen, (start, end, valid)) =
        kivact::test::render_with_handle(crate::options(), range_app);
    let (from, to) = (
        screen.get_by_role(Role::TextBox, "Start"),
        screen.get_by_role(Role::TextBox, "End"),
    );
    // The start of a date is no date, and not yet wrong.
    select_all(&to);
    to.type_text("2026-10-0");
    assert!(!valid.get_untracked());
    assert!(!to.invalid() && !from.invalid());
    assert_eq!(end.get_untracked(), date(2026, 10, 5));
    screen.press_tab(false);
    assert!(to.invalid());
    screen.get_by_role(Role::Status, "Use YYYY-MM-DD");

    select_all(&to);
    to.type_text("2026-02-30");
    assert!(!valid.get_untracked());
    assert!(to.invalid());
    assert!(!from.invalid());
    screen.get_by_role(Role::Status, "This date does not exist");
    assert_eq!(to.described_by().len(), 1);
    assert!(from.described_by().is_empty());
    assert_eq!(end.get_untracked(), date(2026, 10, 5));
    assert_eq!(start.get_untracked(), date(2026, 10, 3));

    // Choosing from the calendar repairs the text.
    screen.get_by_role(Role::Button, "Choose a date").click();
    screen.get_by_role(Role::GridCell, "2026-10-12").click();
    screen.get_by_role(Role::GridCell, "2026-10-13").click();
    assert!(valid.get_untracked());
    assert!(!to.invalid());
    assert_eq!(to.value().as_deref(), Some("2026-10-13"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn date_popup_uses_the_opener_for_focus_and_the_field_strip_for_placement() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(date(2026, 10, 3));
        (
            (),
            column((
                ui::date_input("Date", value, date(2026, 10, 3)),
                kivact::ui::button("After"),
            )),
        )
    });
    let opener = screen.get_by_role(Role::Button, "Choose a date");
    opener.a11y(A11yAction::Expand);
    let popup = screen.get_by_role(Role::Dialog, "Date");
    assert_eq!(opener.controls().len(), 1);
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-03")
            .is_selected()
    );
    let bounds = popup.rect().expect("calendar popup bounds");
    let field = screen
        .get_by_role(Role::TextBox, "Date")
        .rect()
        .expect("date field bounds");
    assert!(
        bounds.x < field.x,
        "the popup starts at the full field strip"
    );
    screen.get_by_role(Role::GridCell, "2026-10-03").focus();
    screen.press_key(Key::Escape);
    assert!(opener.is_focused());
    assert_eq!(opener.is_expanded(), Some(false));
    opener.a11y(A11yAction::Expand);
    screen
        .get_by_role(Role::Dialog, "Date")
        .a11y(A11yAction::Dismiss);
    assert!(opener.is_focused());
    assert_eq!(opener.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn year_month_menus_navigate_without_changing_the_selected_day() {
    let (screen, selected) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(date(2026, 10, 3));
        (
            selected,
            ui::calendar(selected, date(2026, 10, 3))
                .year_month_menus(true)
                .month_title(|date| {
                    assert_eq!(date.day(), 1);
                    format!("Month {}", date.month())
                }),
        )
    });
    screen.get_by_role(Role::ComboBox, "Year").click();
    screen.get_by_role(Role::Option, "1976");
    screen.get_by_role(Role::Option, "2076");
    assert!(screen.query_by_role(Role::Option, "1975").is_none());
    assert!(screen.query_by_role(Role::Option, "2077").is_none());
    screen.press_key(Key::ArrowDown);
    screen.get_by_role(Role::Option, "2027").click();
    screen.get_by_role(Role::GridCell, "2027-10-15");
    let month = screen.get_by_role(Role::ComboBox, "Month");
    month.focus();
    month.press_key(Key::Enter);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    screen.get_by_role(Role::GridCell, "2027-11-15");
    assert_eq!(selected.get_untracked(), date(2026, 10, 3));
    let year = screen.get_by_role(Role::ComboBox, "Year");
    year.focus();
    year.press_key(Key::Enter);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    screen.get_by_role(Role::GridCell, "2028-11-15");
    assert_eq!(selected.get_untracked(), date(2026, 10, 3));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn range_popup_announces_the_stage_and_closes_back_to_the_start_input() {
    let (screen, _) = kivact::test::render_with_handle(crate::options(), range_app);
    screen.get_by_role(Role::Button, "Choose a date").click();
    let instructions = screen.get_by_role(Role::Status, "Pick a start date");
    assert_eq!(instructions.live_region(), LiveRegion::Polite);
    screen.get_by_role(Role::GridCell, "2026-10-10").click();
    assert_eq!(instructions.name(), "Pick an end date");
    assert_eq!(instructions.live_region(), LiveRegion::Polite);
    screen.get_by_role(Role::Button, "Close").click();
    assert!(screen.query_by_role(Role::Dialog, "Start").is_none());
    assert!(screen.get_by_role(Role::TextBox, "Start").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn calendar_has_no_weekend_tones_by_default() {
    let (screen, _) = mount_in("en-US", || {
        let selected = signal(date(2026, 10, 20));
        (selected, ui::calendar(selected, date(2026, 10, 21)))
    });
    assert_eq!(tint(&screen, "2026-10-10"), (0, 0));
    assert_eq!(tint(&screen, "2026-10-11"), (0, 0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
