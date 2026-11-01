//! Role: Tests for localization and component text handling.
//!
//! The locale and the texts of components (`kivact::i18n`): a component's
//! default text, the App's replacement, a switch at runtime and a subtree's
//! own.

use kivact::i18n::{Locale, Messages, Weekday, locale, message, message_with};
use kivact::prelude::*;
use kivact::test::{RenderOptions, render, render_with};
use kivact::view::{provide_locale, provide_messages};

/// A part that shows one of its texts the way a component library does.
fn close_label() -> impl IntoView {
    text(message("dialog.close", "Close")).test_id("close")
}

fn locale_tag() -> impl IntoView {
    let locale = locale();
    text(move || locale.get().tag().to_owned()).test_id("locale")
}

#[test]
fn a_message_keeps_the_components_default_until_the_app_sets_it() {
    let screen = render(close_label);
    screen.get_by_text("Close");

    // A key the App does not set keeps the default whatever else it sets.
    let screen = render_with(
        RenderOptions {
            messages: Messages::new().with("palette.label", "Command palette"),
            ..RenderOptions::from_env()
        },
        close_label,
    );
    screen.get_by_text("Close");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_apps_messages_replace_the_default() {
    let screen = render_with(
        RenderOptions {
            messages: Messages::new().with("dialog.close", "閉じる"),
            ..RenderOptions::from_env()
        },
        || {
            column((
                close_label(),
                text(message("dialog.cancel", "Cancel")).test_id("cancel"),
                // The label a screen reader reads follows too.
                node()
                    .role(Role::Button)
                    .name(message("dialog.close", "Close"))
                    .focusable(true),
            ))
        },
    );
    screen.get_by_text("閉じる");
    screen.get_by_text("Cancel");
    screen.get_by_role(Role::Button, "閉じる");
    assert!(screen.query_by_text("Close").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_locale_is_en_us_when_nobody_names_one_and_the_apps_when_it_gives_one() {
    let screen = render(locale_tag);
    screen.get_by_text("en-US");

    let screen = render_with(
        RenderOptions {
            locale: Some(Locale::new("ja-JP")),
            ..RenderOptions::from_env()
        },
        locale_tag,
    );
    screen.get_by_text("ja-JP");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_app_without_a_locale_speaks_the_hosts_and_follows_it() {
    let host = |tag: Option<&str>| RenderOptions {
        host_locale: tag.map(str::to_owned),
        ..RenderOptions::from_env()
    };
    let screen = render_with(host(Some("ja-JP")), locale_tag);
    screen.get_by_text("ja-JP");
    let before = screen.stats();

    // The system's locale changes while the App runs: what reads the
    // locale follows, remounting nothing.
    screen.set_host_locale(Some("de-DE"));
    screen.get_by_text("de-DE");
    assert!(screen.query_by_text("ja-JP").is_none());
    assert_eq!(screen.stats().creates, before.creates);

    // A host that names none leaves the default.
    screen.set_host_locale(None);
    screen.get_by_text("en-US");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());

    let screen = render_with(host(None), locale_tag);
    screen.get_by_text("en-US");
}

#[test]
fn the_locale_the_app_gives_wins_over_the_hosts_and_ignores_its_changes() {
    let screen = render_with(
        RenderOptions {
            locale: Some(Locale::new("fr-FR")),
            host_locale: Some("ja-JP".to_owned()),
            ..RenderOptions::from_env()
        },
        locale_tag,
    );
    screen.get_by_text("fr-FR");

    screen.set_host_locale(Some("de-DE"));
    screen.get_by_text("fr-FR");
    assert!(screen.query_by_text("de-DE").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_subtree_with_its_own_locale_ignores_the_hosts_changes() {
    let screen = render_with(
        RenderOptions {
            host_locale: Some("ja-JP".to_owned()),
            ..RenderOptions::from_env()
        },
        || {
            column((
                provide_locale(Locale::new("fr-FR"), || {
                    let locale = locale();
                    text(move || format!("inner {}", locale.get().tag()))
                }),
                text({
                    let locale = locale();
                    move || format!("outer {}", locale.get().tag())
                }),
            ))
        },
    );
    screen.get_by_text("inner fr-FR");
    screen.get_by_text("outer ja-JP");
    screen.set_host_locale(Some("de-DE"));
    screen.get_by_text("inner fr-FR");
    screen.get_by_text("outer de-DE");
}

#[test]
fn switching_the_locale_and_the_messages_at_runtime_updates_what_reads_them() {
    let screen = render(|| {
        let tag = signal(Locale::new("en-US"));
        let messages = signal(Messages::new());
        column((
            provide_locale(tag, move || {
                provide_messages(messages, || column((locale_tag(), close_label())))
            }),
            node()
                .role(Role::Button)
                .name("to-ja")
                .focusable(true)
                .style(style().size(80, 20))
                .on_press(move || {
                    tag.set(Locale::new("ja-JP"));
                    messages.set(Messages::new().with("dialog.close", "閉じる"));
                }),
        ))
    });
    screen.get_by_text("en-US");
    screen.get_by_text("Close");
    let before = screen.stats();

    screen.get_by_role(Role::Button, "to-ja").click();
    screen.get_by_text("ja-JP");
    screen.get_by_text("閉じる");
    assert!(screen.query_by_text("Close").is_none());
    assert_eq!(
        screen.stats().creates,
        before.creates,
        "the switch remounts nothing"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_subtree_has_its_own_locale_and_messages_and_siblings_keep_the_outer() {
    let screen = render_with(
        RenderOptions {
            locale: Some(Locale::new("en-US")),
            messages: Messages::new()
                .with("dialog.close", "Close it")
                .with("dialog.cancel", "Cancel it"),
            ..RenderOptions::from_env()
        },
        || {
            column((
                provide_locale(Locale::new("fr-FR"), || {
                    provide_messages(Messages::new().with("dialog.close", "Fermer"), || {
                        column((
                            text({
                                let locale = locale();
                                move || format!("inner {}", locale.get().tag())
                            }),
                            close_label(),
                            // Not set inside: the App's text shows through.
                            text(message("dialog.cancel", "Cancel")),
                        ))
                    })
                }),
                text({
                    let locale = locale();
                    move || format!("outer {}", locale.get().tag())
                }),
                text(message("dialog.close", "Close")),
            ))
        },
    );
    screen.get_by_text("inner fr-FR");
    screen.get_by_text("Fermer");
    screen.get_by_text("Cancel it");
    screen.get_by_text("outer en-US");
    screen.get_by_text("Close it");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_locale_knows_its_language_and_the_first_day_of_the_week() {
    let locale = Locale::new("pt_BR");
    assert_eq!(locale.tag(), "pt_BR");
    assert_eq!(locale.language(), "pt");

    let first_day = |tag: &str| Locale::new(tag).first_day_of_week();
    for tag in ["ja-JP", "en-US", "ko-KR", "pt-BR", "en", "ja", "zh-Hant-TW"] {
        assert_eq!(first_day(tag), Weekday::Sunday, "{tag}");
    }
    for tag in [
        "en-GB", "de-DE", "fr-FR", "es-ES", "it", "nl-NL", "ru", "sv-SE",
    ] {
        assert_eq!(first_day(tag), Weekday::Monday, "{tag}");
    }
    assert_eq!(first_day("ar-EG"), Weekday::Saturday);
    assert_eq!(first_day("fa"), Weekday::Saturday);
    assert_eq!(first_day("dv-MV"), Weekday::Friday);
    // A region the table does not list starts on Monday.
    assert_eq!(first_day("xx-ZZ"), Weekday::Monday);
    // The region wins over the language, a script does not hide it.
    assert_eq!(first_day("en-AU"), Weekday::Sunday);
    assert_eq!(first_day("zh-Hans-CN"), Weekday::Monday);
}

#[test]
fn a_message_with_values_follows_both_the_values_and_the_messages() {
    let screen = render(|| {
        let count = signal(1usize);
        let wording = signal(Messages::new());
        column((
            provide_messages(wording, move || {
                text(message_with(
                    "table.selected",
                    "{count} selected",
                    move || [("count", count.get().to_string())],
                ))
            }),
            // A name the wording does not give stays as written.
            text(message_with("x.unknown", "{who} and {count}", || {
                [("count", "2".to_owned())]
            })),
            node()
                .role(Role::Button)
                .name("more")
                .focusable(true)
                .style(style().size(60, 20))
                .on_press(move || count.update(|n| *n += 1)),
            node()
                .role(Role::Button)
                .name("ja")
                .focusable(true)
                .style(style().size(60, 20))
                .on_press(move || {
                    wording.set(Messages::new().with("table.selected", "{count} 件選択中"));
                }),
        ))
    });
    screen.get_by_text("1 selected");
    screen.get_by_text("{who} and 2");
    screen.get_by_role(Role::Button, "more").click();
    screen.get_by_text("2 selected");
    screen.get_by_role(Role::Button, "ja").click();
    screen.get_by_text("2 件選択中");
    screen.get_by_role(Role::Button, "more").click();
    screen.get_by_text("3 件選択中");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_calendar_date_is_always_a_real_day_and_counts_in_calendar_order() {
    use kivact::i18n::CalendarDate;

    assert_eq!(CalendarDate::checked(2026, 2, 29), None);
    assert_eq!(
        CalendarDate::checked(2024, 2, 29).map(|d| d.day()),
        Some(29)
    );
    assert_eq!(CalendarDate::checked(2026, 13, 1), None);
    assert_eq!(CalendarDate::checked(2026, 4, 31), None);
    assert_eq!(CalendarDate::checked(2026, 0, 1), None);
    assert_eq!(CalendarDate::checked(2026, 1, 0), None);
    // `new` moves a date to the nearest real one.
    let moved = CalendarDate::new(2026, 14, 40);
    assert_eq!((moved.year(), moved.month(), moved.day()), (2026, 12, 31));
    assert_eq!(CalendarDate::new(2026, 2, 0), CalendarDate::new(2026, 2, 1));
    // Leap years: every 4th, not every 100th, but every 400th.
    for (year, days) in [(2024, 29), (2026, 28), (1900, 28), (2000, 29), (2100, 28)] {
        assert_eq!(
            CalendarDate::new(year, 2, 1).days_in_month(),
            days,
            "{year}"
        );
    }
    let lengths = (1..=12)
        .map(|m| CalendarDate::new(2026, m, 1).days_in_month())
        .collect::<Vec<_>>();
    assert_eq!(lengths, [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]);
    assert_eq!(
        CalendarDate::new(2026, 10, 14).first_of_month(),
        CalendarDate::new(2026, 10, 1)
    );
    assert!(CalendarDate::new(2026, 10, 31) < CalendarDate::new(2026, 11, 1));
    assert!(CalendarDate::new(2025, 12, 31) < CalendarDate::new(2026, 1, 1));
    assert_eq!(CalendarDate::new(2026, 3, 7).to_string(), "2026-03-07");
}

#[test]
fn date_arithmetic_crosses_months_years_and_clamps_the_day() {
    use kivact::i18n::CalendarDate;

    let d = CalendarDate::new;
    assert_eq!(d(2026, 12, 31).add_days(1), d(2027, 1, 1));
    assert_eq!(d(2027, 1, 1).add_days(-1), d(2026, 12, 31));
    assert_eq!(d(2026, 2, 28).add_days(1), d(2026, 3, 1));
    assert_eq!(d(2024, 2, 28).add_days(1), d(2024, 2, 29));
    assert_eq!(d(2026, 10, 14).add_days(365), d(2027, 10, 14));
    assert_eq!(
        d(2026, 10, 14).add_days(-14_000).add_days(14_000),
        d(2026, 10, 14)
    );
    assert_eq!(d(2026, 1, 31).add_months(1), d(2026, 2, 28));
    assert_eq!(d(2024, 1, 31).add_months(1), d(2024, 2, 29));
    assert_eq!(d(2026, 3, 31).add_months(-1), d(2026, 2, 28));
    assert_eq!(d(2026, 11, 15).add_months(2), d(2027, 1, 15));
    assert_eq!(d(2026, 1, 15).add_months(-1), d(2025, 12, 15));
    assert_eq!(d(2026, 6, 15).add_months(-18), d(2024, 12, 15));
    assert_eq!(d(2024, 2, 29).add_months(12), d(2025, 2, 28));
    // Days since 1970-01-01 round trip, before and after it.
    for days in [-800_000i64, -1, 0, 1, 20_000, 1_000_000] {
        assert_eq!(CalendarDate::from_epoch_days(days).to_epoch_days(), days);
    }
    assert_eq!(d(1970, 1, 1).to_epoch_days(), 0);
    assert_eq!(d(2000, 3, 1).to_epoch_days(), 11_017);
}

#[test]
fn the_weekday_of_a_date_and_the_columns_of_a_week_follow_the_first_day() {
    use kivact::i18n::{CalendarDate, CalendarMonth};

    let d = CalendarDate::new;
    assert_eq!(d(1970, 1, 1).weekday(), Weekday::Thursday);
    assert_eq!(d(2000, 1, 1).weekday(), Weekday::Saturday);
    assert_eq!(d(2026, 10, 3).weekday(), Weekday::Saturday);
    assert_eq!(d(2024, 2, 29).weekday(), Weekday::Thursday);
    assert_eq!(d(1900, 1, 1).weekday(), Weekday::Monday);
    assert_eq!(d(1600, 12, 31).weekday(), Weekday::Sunday);
    assert_eq!(Weekday::Sunday.days_after(1), Weekday::Monday);
    assert_eq!(Weekday::Monday.days_after(13), Weekday::Sunday);
    assert_eq!(Weekday::Sunday.days_since(Weekday::Monday), 6);
    assert_eq!(Weekday::Monday.days_since(Weekday::Sunday), 1);
    assert_eq!(Weekday::Saturday.days_since(Weekday::Saturday), 0);
    // October 2026 starts on a Thursday.
    let october = CalendarMonth::new(2026, 10);
    for (first, columns, lead) in [
        (Weekday::Monday, 5, 3),
        (Weekday::Sunday, 5, 4),
        (Weekday::Saturday, 6, 5),
    ] {
        let weeks = october.weeks(first, 0);
        assert_eq!(weeks.len(), columns);
        assert_eq!(weeks[0][lead], d(2026, 10, 1));
        assert!(weeks.iter().all(|week| week[0].weekday() == first));
        assert_eq!(
            weeks
                .concat()
                .windows(2)
                .filter(|w| w[0].add_days(1) != w[1])
                .count(),
            0
        );
    }
    // A month of four exact weeks (February 2027 starts on a Monday) and one
    // of six; a grid of a fixed height asks for six.
    assert_eq!(
        CalendarMonth::new(2027, 2).weeks(Weekday::Monday, 0).len(),
        4
    );
    assert_eq!(
        CalendarMonth::new(2027, 2).weeks(Weekday::Monday, 6).len(),
        6
    );
    assert_eq!(
        CalendarMonth::new(2026, 8).weeks(Weekday::Monday, 0).len(),
        6
    );
    assert_eq!(
        CalendarMonth::new(2026, 12).add_months(1),
        CalendarMonth::new(2027, 1)
    );
    assert_eq!(
        CalendarMonth::new(2026, 1).add_months(-1),
        CalendarMonth::new(2025, 12)
    );
    assert_eq!(
        CalendarMonth::new(2026, 10).months_until(CalendarMonth::new(2027, 2)),
        4
    );
    assert_eq!(
        CalendarMonth::new(2026, 10).months_until(CalendarMonth::new(2026, 3)),
        -7
    );
}
