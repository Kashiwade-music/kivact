//! Role: The selection page: segmented radio groups, toggle buttons and tabs with keyed choices.

use kivact::prelude::*;
use kivact::style::{Align, Color, GridPlacement, Justify, Track, color, pct, rgb, rgba, style};
use kivact::view::Node;
use kivact_ui_pop as ui;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::i18n::{CalendarDate, Weekday};
use kivact::interaction::{ListItem, MenuEntry, ScalarRange};
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::state::{Callback, IntoHandler, IntoText, IntoValue, Machine, MachineHandle, Signal, Step, Value, callback, machine};
use kivact::style::{Direction, Filter, Shadow, Shape, TextDecorationLine, linear_gradient};
use kivact::ui::RadioGroup;
use kivact::view::{Orientation, Placement, rich_text, span as text_span};
use kivact_ui_pop::Hue;
use std::time::Duration;
use catalog_shell::{Example, Page};
use super::common::*;
/// The page of the segmented controls, toggles and tabs.
pub fn page() -> Page {
    Page::new(
        "selection",
        "Selection",
        "Components",
        include_str!("selection.rs"),
    )
    .description("Choices keyed by their own identity: a segmented control, toggles and tabs.")
    .example(Example::new(
        "selection",
        "Segments and tabs",
        "A segmented control is a radio group in the pill look.",
        selection_demo,
    ))
}

fn option(key: &str, label: &str) -> ListItem<String> {
    ListItem::new(key.to_owned(), label)
}

/// A segmented control: a standard radio group in the kit's pill look, its
/// segments showing the icon, the label and the count badge.
fn segmented<K: kivact::state::ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: kivact::state::Signal<Option<K>>,
) -> RadioGroup<K> {
    kivact::ui::radio_group(label, options, value)
        .orientation(Orientation::Horizontal)
        .variant(kivact::ui::RadioGroupVariant::Segmented)
}

fn selection_demo() -> impl IntoView {
    // example: selection
    let tab = signal("overview".to_owned());
    let tabs = signal(vec![
        option("overview", "Overview"),
        option("skills", "Skills").with_description("4"),
        option("equipment", "Equipment"),
        option("profile", "Profile"),
    ]);
    let periods = signal(vec![
        option("day", "Day"),
        option("week", "Week"),
        option("month", "Month"),
        option("year", "Year"),
    ]);
    let period = signal(Some("day".to_owned()));
    let layouts = signal(vec![
        option("cards", "Cards").with_icon("layout-grid"),
        option("list", "List").with_icon("list"),
        option("chart", "Chart").with_icon("chart-bar"),
    ]);
    let layout = signal(Some("cards".to_owned()));
    let segments = column((
        segmented("Period", periods, period),
        segmented("Layout", layouts, layout),
        kivact::ui::button("Reverse the layouts")
            .on_press(move || layouts.update(|options| options.reverse())),
        wrap((
            kivact::ui::toggle_button("Striker", signal(true)),
            kivact::ui::toggle_button("Special", kivact::state::signal(false)),
            kivact::ui::toggle_button("Tank", signal(true)),
            kivact::ui::toggle_button("Healer", kivact::state::signal(false)),
            kivact::ui::toggle_button("Support", kivact::state::signal(false)),
        )),
    ))
    .style(style().gap(16).align_items(Align::Start));
    let tab_choices = column((
        kivact::ui::tabs("Skills view", tabs, tab)
            .item(ui::tab_label)
            .content(|key| {
                text(panel(Some(key.as_str()))).style(style().color(color::TEXT_MUTED))
            }),
        row((
            kivact::ui::button("Reverse the tab order")
                .on_press(move || tabs.update(|options| options.reverse())),
            kivact::ui::button("Toggle the skills tab").on_press(move || {
                tabs.update(|options| {
                    if options.iter().any(|option| option.key() == "skills") {
                        options.retain(|option| option.key() != "skills");
                    } else {
                        options.insert(
                            1.min(options.len()),
                            option("skills", "Skills").with_description("4"),
                        );
                    }
                })
            }),
        ))
        .style(style().gap(8).margin_top(12)),
        segmented(
            "Filter",
            vec![
                option("all", "All"),
                option("unread", "Unread"),
                option("favorites", "Favorites"),
            ],
            signal(Some("all".to_owned())),
        )
        .style(style().margin_top(16)),
    ))
    .style(style().align_items(Align::Start));
    section(
        "04",
        "Selection & Tabs",
        "Segments, tabs and toggles",
        cards_grid((
            half(card("Segmented control", segments)),
            half(card("Tabs", tab_choices)),
        )),
    )
    // end example
}

fn panel(key: Option<&str>) -> String {
    match key {
        Some("overview") => {
            "The calm vice captain of the investigation unit. Supports allies with precise fire from behind. Striker / Back line"
        }
        Some("skills") => "EX skill \"Azure Barrage\": deals 412% ATK damage to enemies in range.",
        Some("equipment") => "Hat T7 / Gloves T6 / Watch T7",
        Some("profile") => "Birthday Oct 3 · Hobby stargazing · Height 158 cm",
        _ => "Select a tab.",
    }
    .to_owned()
}
