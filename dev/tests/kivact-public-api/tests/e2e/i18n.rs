//! Role: Tests for localization and component text handling.
//!
//! The locale and the texts of components (`kivact::i18n`): a component's
//! default text, the App's replacement, a switch at runtime and a subtree's
//! own.

use kivact::i18n::{Locale, Messages, message, message_with, use_locale};
use kivact::i18n::{provide_locale, provide_messages};
use kivact::prelude::*;
use kivact::test::{RenderOptions, render, render_with};

/// A part that shows one of its texts the way a component library does.
fn close_label() -> impl Component {
    text(message("dialog.close", "Close")).test_id("close")
}

fn locale_tag() -> impl Component {
    let locale = use_locale();
    text(move || locale.get().tag().to_owned()).test_id("locale")
}

#[test]
fn the_apps_messages_replace_the_default() {
    let screen = render_with(
        RenderOptions::from_env().messages(Messages::new().with("dialog.close", "閉じる")),
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
        RenderOptions::from_env()
            .locale(Locale::new("en-US"))
            .messages(
                Messages::new()
                    .with("dialog.close", "Close it")
                    .with("dialog.cancel", "Cancel it"),
            ),
        || {
            column((
                provide_locale(Locale::new("fr-FR"), || {
                    provide_messages(Messages::new().with("dialog.close", "Fermer"), || {
                        column((
                            text({
                                let locale = use_locale();
                                move || format!("inner {}", locale.get().tag())
                            }),
                            close_label(),
                            // Not set inside: the App's text shows through.
                            text(message("dialog.cancel", "Cancel")),
                        ))
                    })
                }),
                text({
                    let locale = use_locale();
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
fn en_us_formats_numbers_dates_and_names() {
    use kivact::i18n::{
        CalendarDate, DateStyle, FractionDigits, Grouping, HourCycle, NameWidth, NumberStyle,
        TimeOfDay, TimeStyle, Weekday,
    };
    let screen = render_with(
        RenderOptions::from_env().locale(Locale::new("en-US")),
        || {
            let current = use_locale().get();
            let date = CalendarDate::new(2026, 11, 5);
            let number = NumberStyle {
                fraction_digits: FractionDigits::Exact(2),
                grouping: Grouping::Locale,
            };
            column((
                text(current.format_number(1234.5, number)),
                text(current.format_date(date, DateStyle::Long)),
                text(current.format_date(date, DateStyle::Medium)),
                text(current.format_date(date, DateStyle::Iso)),
                text(current.month_name(11, NameWidth::Long)),
                text(current.month_name(11, NameWidth::Narrow)),
                text(current.weekday_name(Weekday::Thursday, NameWidth::Short)),
                text(current.format_time(
                    TimeOfDay {
                        hour: 13,
                        minute: 2,
                        second: 3,
                    },
                    TimeStyle { seconds: false },
                )),
                text(format!(
                    "parsed: {} {}",
                    current.parse_number("1,234.50").unwrap(),
                    current.parse_date("11/5/2026").unwrap()
                )),
                text(format!(
                    "cycle: {:?}; first: {:?}",
                    current.hour_cycle(),
                    current.first_day_of_week()
                )),
                text(format!("days: {}", CalendarDate::from_days(date.to_days()))),
            ))
        },
    );
    screen.get_by_text("1,234.50");
    screen.get_by_text("November 5, 2026");
    screen.get_by_text("Nov 5, 2026");
    screen.get_by_text("2026-11-05");
    screen.get_by_text("November");
    screen.get_by_text("N");
    screen.get_by_text("Thu");
    screen.get_by_text("1:02\u{202f}PM");
    screen.get_by_text("parsed: 1234.5 2026-11-05");
    screen.get_by_text(&format!(
        "cycle: {:?}; first: {:?}",
        HourCycle::H12,
        Weekday::Sunday
    ));
    screen.get_by_text("days: 2026-11-05");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn ja_jp_and_de_de_format_numbers_and_dates() {
    use kivact::i18n::{
        CalendarDate, DateStyle, FractionDigits, Grouping, NameWidth, NumberStyle, Weekday,
    };
    let screen = render(|| {
        let date = CalendarDate::new(2026, 11, 5);
        let number = NumberStyle {
            fraction_digits: FractionDigits::Range { min: 0, max: 2 },
            grouping: Grouping::Locale,
        };
        let ja = Locale::new("ja_JP");
        let de = Locale::new("de-DE");
        let ar = Locale::new("ar-EG");
        let arabic = ar.format_number(-1234.5, number);
        let thai = Locale::new("en-US-u-nu-thai");
        let thai_number = thai.format_number(1234.5, number);
        let han = Locale::new("en-US-u-nu-hanidec");
        let han_date = han.format_date(date, DateStyle::Short);
        column((
            text(format!("ja: {}", ja.format_number(1234.5, number))),
            text(format!("de: {}", de.format_number(1234.5, number))),
            text(ja.format_date(date, DateStyle::Long)),
            text(de.format_date(date, DateStyle::Long)),
            text(ja.month_name(11, NameWidth::Long)),
            text(de.weekday_name(Weekday::Thursday, NameWidth::Long)),
            text(format!(
                "ja parsed: {}",
                ja.parse_date("2026/11/5").unwrap()
            )),
            text(format!(
                "de parsed: {} {}",
                de.parse_number("1.234,5").unwrap(),
                de.parse_date("5.11.2026").unwrap()
            )),
            text(format!(
                "arabic parsed: {}",
                ar.parse_number(&arabic).unwrap()
            )),
            text(format!(
                "nu parsed: {}",
                thai.parse_number(&thai_number).unwrap()
            )),
            text(format!("han date: {}", han.parse_date(&han_date).unwrap())),
            text(format!(
                "invalid: {:?} {:?}",
                de.parse_number("12,3,4"),
                de.parse_date("31.2.2026")
            )),
            text(format!(
                "ISO parsed: {}",
                de.parse_date("2026-11-05").unwrap()
            )),
        ))
    });
    for expected in [
        "ja: 1,234.5",
        "de: 1.234,5",
        "2026年11月5日",
        "5. November 2026",
        "11月",
        "Donnerstag",
        "ja parsed: 2026-11-05",
        "de parsed: 1234.5 2026-11-05",
        "arabic parsed: -1234.5",
        "nu parsed: 1234.5",
        "han date: 2026-11-05",
        "invalid: None None",
        "ISO parsed: 2026-11-05",
    ] {
        screen.get_by_text(expected);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn plural_forms_follow_the_locale() {
    use kivact::i18n::PluralForms;
    let forms = PluralForms {
        zero: Some("{count} zero".into()),
        one: Some("{count} one".into()),
        two: Some("{count} two".into()),
        few: Some("{count} few".into()),
        many: Some("{count} many".into()),
        other: "{count} other".into(),
    };
    let screen = render_with(
        RenderOptions::from_env().messages(Messages::new().with_plural("items", forms)),
        || {
            column((
                provide_locale(Locale::new("en-US"), || {
                    text(message_with("items", "fallback", || {
                        [("count", "1".into())]
                    }))
                }),
                provide_locale(Locale::new("ja-JP"), || {
                    text(message_with("items", "fallback", || {
                        [("count", "1".into())]
                    }))
                }),
                provide_locale(Locale::new("ru-RU"), || {
                    text(message_with("items", "fallback", || {
                        [("count", "3".into())]
                    }))
                }),
                provide_locale(Locale::new("ar-EG"), || {
                    text(message_with("items", "fallback", || {
                        [("count", "2".into())]
                    }))
                }),
            ))
        },
    );
    for expected in ["1 one", "1 other", "3 few", "2 two"] {
        screen.get_by_text(expected);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn collation_sorts_by_locale() {
    let screen = render(|| {
        let mut de = ["z", "ä", "a"];
        de.sort_by(|a, b| Locale::new("de-DE").compare(a, b));
        let mut sv = ["z", "ä", "a"];
        sv.sort_by(|a, b| Locale::new("sv-SE").compare(a, b));
        column((
            text(format!("de: {}", de.join(", "))),
            text(format!("sv: {}", sv.join(", "))),
        ))
    });
    screen.get_by_text("de: a, ä, z");
    screen.get_by_text("sv: a, z, ä");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
