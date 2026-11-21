//! Role: The Schedule page.

use super::common::*;
use catalog_shell::{Example, Page};
use kivact::i18n::{CalendarDate, Weekday};
use kivact::prelude::*;
use kivact::style::GridPlacement;
use kivact_ui_pop as ui;
/// The page of the Schedule components.
pub fn page() -> Page {
    Page::new(
        "schedule",
        "Schedule",
        "Components",
        include_str!("schedule.rs"),
    )
    .description("A weekly agenda and a calendar, with each event in its hue.")
    .example(Example::new(
        "schedule",
        "Agenda and calendar",
        "The agenda lists the day's events; the calendar picks the day.",
        schedule_demo,
    ))
}

const SCHEDULE_EVENTS: [(u8, &str, &str, &str, u32, ui::Hue); 11] = [
    (
        1,
        "09:00",
        "Morning stand-up",
        "Meeting room A · everyone",
        0x1f9bff,
        ui::Hue::Blue,
    ),
    (
        3,
        "10:00",
        "Outer district patrol",
        "Minato, Arisa, Sena",
        0x1f9bff,
        ui::Hue::Blue,
    ),
    (
        3,
        "12:30",
        "Lunch (Minato)",
        "Cafeteria",
        0xff7cb0,
        ui::Hue::Pink,
    ),
    (
        3,
        "16:00",
        "Budget review",
        "Yuzu · online",
        0xf2bf0a,
        ui::Hue::Yellow,
    ),
    (
        7,
        "13:00",
        "Rescue drill",
        "Haruka · gymnasium",
        0x2fcfa6,
        ui::Hue::Mint,
    ),
    (
        10,
        "18:00",
        "REQ-1003 deadline",
        "Submit report",
        0xff5a6e,
        ui::Hue::Red,
    ),
    (
        10,
        "19:00",
        "After-party",
        "Lunch Club",
        0xff7cb0,
        ui::Hue::Pink,
    ),
    (
        15,
        "14:00",
        "All-out battle planning",
        "Operations room",
        0x9d86ff,
        ui::Hue::Blue,
    ),
    (
        17,
        "23:59",
        "Moon-viewing festival ends",
        "Event",
        0xff7cb0,
        ui::Hue::Pink,
    ),
    (
        22,
        "11:00",
        "New member interview",
        "Koharu",
        0x2fcfa6,
        ui::Hue::Mint,
    ),
    (
        28,
        "15:00",
        "Monthly report",
        "All academies",
        0x1f9bff,
        ui::Hue::Blue,
    ),
];

fn schedule_demo() -> impl IntoView {
    // example: schedule
    let date = signal(CalendarDate::new(2026, 10, 3));
    section(
        "11",
        "Schedule",
        "Calendar and agenda",
        cards_grid((calendar_panel(date), agenda_panel(date))),
    )
    // end example
}

fn agenda_panel(date: Signal<CalendarDate>) -> impl kivact::view::Element {
    kivact::ui::card(
        column((
            row((
                text(move || {
                    let d = date.get();
                    format!(
                        "Agenda for {}/{} ({})",
                        d.month(),
                        d.day(),
                        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                            [d.weekday().days_since(Weekday::Monday)]
                    )
                })
                .style(style().font_size(15).font_weight(600).flex_grow(1)),
                kivact::ui::button("Add event")
                    .icon(kivact::view::icon("plus"))
                    .size(kivact::ui::ControlSize::Small)
                    .variant(kivact_ui_pop::ButtonTone::Soft),
            ))
            .style(style().align_items(Align::Center)),
            dynamic(date, move |selected_date| {
                let items = SCHEDULE_EVENTS
                    .into_iter()
                    .filter(|(day, _, _, _, _, _)| {
                        selected_date.year() == 2026
                            && selected_date.month() == 10
                            && selected_date.day() == *day
                    })
                    .map(|(_, time, title, subtitle, color, hue)| {
                        ui::agenda_item(time, title)
                            .description(subtitle)
                            .tone(hue)
                            .period(if time < "12:00" { "AM" } else { "PM" })
                            .on_details(move || {
                                kivact::interaction::use_toaster().push(
                                    kivact::interaction::Toast::new(title)
                                        .tone(super::common::tone_of(hue))
                                        .body(subtitle),
                                );
                            })
                            .style(style().border_left(3, rgb(color)))
                    })
                    .collect::<Vec<_>>();
                if !items.is_empty() {
                    column(items).style(style().gap(8))
                } else {
                    row((
                        kivact::view::icon("calendar"),
                        muted("No events. Take a rest."),
                    ))
                    .style(
                        style()
                            .padding(22)
                            .gap(8)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .border_radius(8)
                            .border(1, color::BORDER),
                    )
                }
            }),
        ))
        .style(style().gap(16)),
    )
    .style(style().grid_column(GridPlacement::span(7)))
}

fn calendar_panel(date: Signal<CalendarDate>) -> impl kivact::view::Element {
    kivact::ui::card(
        {
            let selected: kivact::state::Signal<kivact::i18n::CalendarDate> = date;
            let selected = kivact::state::binding(
                move || Some(selected.get()),
                move |day| {
                    if let Some(day) = day {
                        selected.set(day);
                    }
                },
            );
            kivact::ui::calendar(selected)
                .today(CalendarDate::new(2026, 10, 3))
                .month_title(|date| format!("{}.{:02}", date.year(), date.month()))
        }
        .variant(ui::CalendarWeekdayHues([
            None,
            None,
            None,
            None,
            None,
            Some(ui::Hue::Blue),
            Some(ui::Hue::Red),
        ]))
        .events(
            SCHEDULE_EVENTS
                .into_iter()
                .map(|(day, _, _, _, color, _)| kivact::ui::CalendarEvent {
                    date: CalendarDate::new(2026, 10, day),
                    color: rgb(color),
                })
                .collect::<Vec<_>>(),
        )
        .test_id("pop-calendar"),
    )
    .style(style().grid_column(GridPlacement::span(5)))
}
