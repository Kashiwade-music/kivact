//! Role: The Navigation page.

use super::common::*;
use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::style::GridPlacement;
use kivact::style::TextDecorationLine;
use kivact::view::{rich_text, span as text_span};
use kivact_ui_pop as ui;
/// The page of the Navigation components.
pub fn page() -> Page {
    Page::new(
        "navigation",
        "Navigation",
        "Components",
        include_str!("navigation.rs"),
    )
    .description("Breadcrumbs, pagination, steppers and menus for moving through the app.")
    .example(Example::new(
        "navigation",
        "Navigation",
        "Each control moves between pages, steps or sections.",
        navigation_demo,
    ))
}

fn navigation_demo() -> impl IntoView {
    // example: navigation
    let notify = notify();
    let current = signal(2usize);
    section(
        "09", "Navigation", "Breadcrumb, stepper, accordion and timeline",
        cards_grid((
            card("Breadcrumb and stepper", column((
                kivact::ui::breadcrumb(["Home", "Requests", "Central ward, outer district", "REQ-1003"].into_iter().enumerate().map(|(key,label)|kivact::interaction::ListItem::new(key,label)).collect::<Vec<_>>()).separator(||kivact::view::icon("chevron-right")),
                kivact::ui::stepper(
                    [("Intake","10/01"),("Team","10/02"),("Execution","In progress"),("Report","—")].into_iter().map(|(label,note)|kivact::ui::StepperItem::new(label).description(note)).collect::<Vec<_>>(),
                    current,
                ).variant(ui::DiamondSteps).min_step_width(0),
                wrap((
                    kivact::ui::button("Back").variant(kivact::ui::ButtonVariant::Secondary).size(kivact::ui::ControlSize::Small).icon(kivact::view::icon("chevron-left")).test_id("pop-step-prev").disabled(move || current.get() == 0).on_press(move || current.update(|v| *v = v.saturating_sub(1))),
                    kivact::ui::button("Next").size(kivact::ui::ControlSize::Small).icon(kivact::view::icon("chevron-right")).test_id("pop-step-next").disabled(move || current.get() == 4).on_press(move || {
                        current.update(|value| *value = (*value + 1).min(4));
                        if current.get() == 4 {
                            notify.call((ui::Hue::Mint, "All steps are complete".into(), "Well done, teacher!".into()));
                        }
                    }),
                )),
                text("Accordion").style(style().margin_top(4).font_size(15).font_weight(600)),
                column((
                    kivact::ui::disclosure("How do requests work?", signal(true)).content(|| muted("Check the details on the dashboard's Requests page, then pick the student in charge to accept.")),
                    kivact::ui::disclosure("Activity point regeneration", signal(false)).content(|| muted("One point regenerates every 6 minutes. Points above the cap are kept in the inbox.")),
                    kivact::ui::disclosure("Export data", signal(false)).content(|| muted("You can export at any time from CSV at the top right of the table.")),
                )),
            )).style(style().gap(16))).style(style().grid_column(GridPlacement::span(7))),
            card("Activity", column((
                ui::timeline_item("Minato deployed to mission REQ-1003").detail("").timestamp("10 min ago").icon("sword").tone(ui::Hue::Blue)
                    .title_content( rich_text([
                        text_span("Minato").font_weight(500).color(color::TEXT),
                        text_span(" deployed to mission "),
                        text_span("REQ-1003").font_weight(500).color(color::ACCENT)
                            .text_decoration(TextDecorationLine::UNDERLINE)
                            .text_underline_offset(3),
                    ])),
                ui::timeline_item("Haruka submitted a report").detail("").timestamp("1 hour ago").icon("circle-check").tone(ui::Hue::Gray)
                    .title_content( activity_title("Haruka", " submitted a report")),
                ui::timeline_item("Yuzu's budget request was approved").detail("Ammo supply ¥120,000 × 4 items").timestamp("3 hours ago").icon("coin").tone(ui::Hue::Gray)
                    .title_content( activity_title("Yuzu", "'s budget request was approved")),
                ui::timeline_item("Arisa's Bond rank rose to 15").detail("").timestamp("Yesterday").icon("heart").tone(ui::Hue::Gray)
                    .title_content( activity_title("Arisa", "'s Bond rank rose to 15")),
                ui::timeline_item("Koharu joined a club").detail("").timestamp("2 days ago").icon("user-plus").tone(ui::Hue::Gray)
                    .title_content( activity_title("Koharu", " joined a club")).style(style().padding_bottom(0)),
            ))).style(style().grid_column(GridPlacement::span(5))),
        )),
    )
    // end example
}

fn activity_title(name: &str, activity: &str) -> impl IntoView {
    rich_text([
        text_span(name).font_weight(500).color(color::TEXT),
        text_span(activity),
    ])
}
