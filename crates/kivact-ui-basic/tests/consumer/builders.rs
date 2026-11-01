//! Role: Every forms, menu and overlay builder lets the caller's style and name win.

use super::assert_clean;
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{RenderOptions, Screen};
use kivact::ui::{ButtonVariant, ListItem, button};
use kivact::view::{Orientation, OverlayAlign, Placement};
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(16)
            .gap(8)
            .grow(1)
            .align_items(Align::Start),
    )
}

fn fruits() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
    ]
}

fn day(y: i32, m: u8, d: u8) -> CalendarDate {
    CalendarDate::new(y, m, d)
}

/// The width the callers ask for; no default of the kit is this wide.
const WIDTH: f32 = 211.0;

fn width_of(screen: &Screen, id: &str) -> f32 {
    screen.get_by_test_id(id).rect().expect("laid out").width
}

fn mount_page(child: impl FnOnce() -> Node + 'static) -> Screen {
    kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            width: 900.0,
            height: 3000.0,
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        move || ((), child()),
    )
    .0
}

#[test]
fn caller_style_and_name_win_for_the_form_builders() {
    let screen = mount_page(|| {
        let width = style().width(WIDTH);
        let steps = vec![basic::QuestionnaireStep::input("Name")];
        page((
            (
                basic::combobox(fruits(), signal(None::<&'static str>))
                    .name("Mine combobox")
                    .style(width.clone())
                    .test_id("combobox"),
                basic::combobox(fruits(), signal(Vec::<&'static str>::new()))
                    .name("Mine chips")
                    .style(width.clone())
                    .test_id("chips"),
                basic::date_picker("Own date", signal(None::<CalendarDate>))
                    .name("Mine date")
                    .style(width.clone())
                    .test_id("date"),
                basic::date_range_picker("Own span", signal(None::<DateRange>))
                    .name("Mine span")
                    .style(width.clone())
                    .test_id("span"),
                kivact::ui::toggle("Own toggle", signal(false))
                    .name("Mine toggle")
                    .style(width.clone())
                    .test_id("toggle"),
                basic::toggle_group(fruits(), signal(Vec::<&'static str>::new()))
                    .name("Mine group")
                    .style(width.clone())
                    .test_id("toggles"),
                basic::toggle_group(fruits(), signal(None::<&'static str>))
                    .name("Mine single")
                    .style(width.clone())
                    .test_id("single"),
            ),
            (
                basic::native_select(fruits(), signal(None::<&'static str>))
                    .name("Mine select")
                    .style(width.clone())
                    .test_id("select"),
                kivact::ui::field(
                    "Own label",
                    kivact::ui::text_input("Email", signal(String::new())),
                )
                .name("Mine field")
                .style(width.clone())
                .test_id("field"),
                basic::field_set(basic::field_group(()))
                    .legend("Own legend")
                    .name("Mine set")
                    .style(width.clone())
                    .test_id("set"),
                basic::input_group(basic::input_group_input("Query", signal(String::new())))
                    .name("Mine group box")
                    .style(width.clone())
                    .test_id("input-group"),
                basic::input_group_button("Own go")
                    .name("Mine go")
                    .style(style().width(77))
                    .test_id("go"),
                basic::calendar(signal(Some(day(2026, 10, 14))))
                    .name("Mine calendar")
                    .style(width.clone())
                    .test_id("calendar"),
                basic::calendar_range(signal(None::<DateRange>))
                    .name("Mine calendar range")
                    .style(width.clone())
                    .test_id("calendar-range"),
                basic::questionnaire(steps, signal(0usize), signal(Vec::new()))
                    .name("Mine questionnaire")
                    .style(width.clone())
                    .test_id("questionnaire"),
            ),
        ))
    });
    for id in [
        "combobox",
        "chips",
        "date",
        "span",
        "toggle",
        "toggles",
        "single",
        "select",
        "field",
        "set",
        "input-group",
        "calendar",
        "calendar-range",
        "questionnaire",
    ] {
        assert_eq!(
            width_of(&screen, id),
            WIDTH,
            "{id}: the caller's width wins"
        );
    }
    assert_eq!(width_of(&screen, "go"), 77.0);
    // The names the callers give replace the kit's.
    screen.get_by_role(Role::TextBox, "Mine combobox");
    screen.get_by_role(Role::Button, "Mine date");
    screen.get_by_role(Role::Button, "Mine span");
    screen.get_by_role(Role::Button, "Mine toggle");
    screen.get_by_role(Role::Group, "Mine group");
    screen.get_by_role(Role::Group, "Mine single");
    screen.get_by_role(Role::TextBox, "Own label");
    screen.get_by_role(Role::Group, "Mine set");
    screen.get_by_role(Role::Group, "Mine group box");
    screen.get_by_role(Role::Button, "Mine go");
    screen.get_by_role(Role::Group, "Mine calendar");
    screen.get_by_role(Role::Group, "Mine calendar range");
    screen.get_by_role(Role::Group, "Mine questionnaire");
    assert_clean(&screen);
}

#[test]
fn the_kit_names_its_builders_when_the_caller_does_not() {
    let screen = mount_page(|| {
        page((
            kivact::ui::field(
                "Email",
                kivact::ui::text_input("Email", signal(String::new())),
            ),
            basic::field_set(basic::field_group(())).legend("Preferences"),
            basic::input_group(basic::input_group_input("Query", signal(String::new()))),
            basic::questionnaire(
                vec![basic::QuestionnaireStep::input("Name")],
                signal(0usize),
                signal(Vec::new()),
            ),
            basic::calendar(signal(None::<CalendarDate>)),
        ))
    });
    screen.get_by_role(Role::TextBox, "Email");
    screen.get_by_role(Role::Group, "Preferences");
    screen.get_by_role(Role::Group, "Questionnaire");
    assert_clean(&screen);
}

#[test]
fn caller_style_wins_for_the_data_table_command_disclosure_and_menubar() {
    #[derive(Clone, PartialEq)]
    struct Row(u32);
    let screen = mount_page(|| {
        let width = style().width(WIDTH);
        page((
            kivact::ui::data_table(
                vec![Row(1)],
                |r: &Row| r.0,
                vec![kivact::ui::DataColumn::new("Id", |r: &Row| r.0.to_string())],
            )
            .name("Mine table")
            .style(width.clone())
            .test_id("data-table"),
            kivact::ui::disclosure("A", signal(false), || text("Body"))
                .style(width.clone())
                .test_id("disclosure"),
            basic::menubar(
                vec![("File".to_string(), vec![MenuEntry::item("new", "New")])],
                |_key: &'static str| {},
            )
            .name("Mine bar")
            .style(width.clone())
            .test_id("menubar"),
        ))
    });
    for id in ["data-table", "disclosure", "menubar"] {
        assert_eq!(
            width_of(&screen, id),
            WIDTH,
            "{id}: the caller's width wins"
        );
    }
    screen.get_by_role(Role::Group, "Mine table");
    screen.get_by_role(Role::MenuBar, "Mine bar");
    assert_clean(&screen);
}

#[test]
fn overlays_attach_to_triggers_that_keep_their_own_style_and_name() {
    let screen = mount_page(|| {
        let open = signal(false);
        let trigger = |label: &'static str| {
            button(label)
                .variant(ButtonVariant::Outline)
                .name(format!("Mine {label}"))
                .style(style().width(97))
                .test_id(label)
        };
        page((
            kivact::ui::dialog("Panel", open)
                .kind(kivact::ui::DialogKind::Popover)
                .placement(Placement::End)
                .attach(trigger("popover")),
            basic::hover_card(|| text("Card")).attach(trigger("hover")),
            menu(
                "Menu",
                vec![MenuEntry::item("a", "A")],
                |_key: &'static str| {},
            )
            .attach(trigger("dropdown")),
            menu(
                "Context",
                vec![MenuEntry::item("a", "A")],
                |_key: &'static str| {},
            )
            .attach_context(trigger("context")),
        ))
    });
    for id in ["popover", "hover", "dropdown", "context"] {
        assert_eq!(width_of(&screen, id), 97.0, "{id}");
        screen.get_by_role(Role::Button, &format!("Mine {id}"));
    }
    assert_clean(&screen);
}

#[test]
fn orientation_and_placement_use_the_shared_kivact_enums() {
    // A compile-time check as much as a run: the components take the shared
    // enums of `kivact`, not their own.
    let screen = mount_page(|| {
        page((
            basic::toggle_group(fruits(), signal(Vec::<&'static str>::new()))
                .orientation(Orientation::Vertical),
            basic::hover_card(|| text("Card"))
                .placement(Placement::Above)
                .align(OverlayAlign::End)
                .attach(button("Anchor")),
        ))
    });
    screen.get_by_role(Role::Button, "Anchor");
    assert_clean(&screen);
}
