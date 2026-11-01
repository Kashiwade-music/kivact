//! Role: Forms tests: fields, selects, choices, sliders, dates and questionnaire.

use super::{assert_clean, distance, pixel, pixel_in};
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::RenderOptions;
use kivact::ui::ListItem;
use kivact::view::Key;
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .padding(16)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

#[test]
fn field_shows_label_description_and_error() {
    let (screen, error) = kivact::test::render_with_handle(crate::options(), || {
        let name = signal(String::new());
        let error = signal(None::<String>);
        (
            error,
            page(
                kivact::ui::field("Username", kivact::ui::text_input("Username", name))
                    .hint("Choose a unique username.")
                    .error(error),
            ),
        )
    });
    let group = screen.get_by_role(Role::TextBox, "Username");
    assert!(group.exists());
    screen.get_by_text("Username");
    screen.get_by_text("Choose a unique username.");
    assert!(screen.query_by_role(Role::Alert, "Required").is_none());
    error.set(Some("Required".into()));
    screen.run_until_idle();
    screen.get_by_role(Role::Alert, "Required");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn input_group_draws_the_ring_on_the_group_and_types() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(String::new());
        (
            value,
            page(
                basic::input_group(
                    basic::input_group_input("Search", value)
                        .placeholder("Search...")
                        .test_id("inner"),
                )
                .start(kivact::control::icon("search"))
                .end(basic::input_group_text("12 results"))
                .test_id("group"),
            ),
        )
    });
    screen.get_by_text("12 results");
    let before = pixel_in(&screen, "group", 0.0);
    screen.get_by_role(Role::TextBox, "Search").click();
    screen.type_text("hi");
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), "hi");
    let after = pixel_in(&screen, "group", 0.0);
    assert!(
        distance(before, after) > 20,
        "the group border turns to the ring color: {before:06x} -> {after:06x}"
    );
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn input_otp_fills_slots_completes_and_backspaces() {
    let (screen, (code, done)) = kivact::test::render_with_handle(crate::options(), || {
        let code = signal(String::new());
        let done = signal(None::<String>);
        (
            (code, done),
            page(
                basic::input_otp("Code", code, 6)
                    .groups(&[3, 3])
                    .on_complete(move |c: String| done.set(Some(c))),
            ),
        )
    });
    screen.get_by_role(Role::TextBox, "Code").click();
    screen.type_text("12a3");
    screen.run_until_idle();
    assert_eq!(code.get_untracked(), "123", "letters are not digits");
    screen.get_by_text("1");
    screen.get_by_text("3");
    assert!(done.get_untracked().is_none());
    screen.type_text("456");
    screen.run_until_idle();
    assert_eq!(code.get_untracked(), "123456");
    assert_eq!(done.get_untracked().as_deref(), Some("123456"));
    screen.press_key(Key::Backspace);
    screen.run_until_idle();
    assert_eq!(code.get_untracked(), "12345");
    screen.assert_layout();
    assert_clean(&screen);
}

fn fruits() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("pear", "Pear").disabled(true),
        ListItem::new("fig", "Fig"),
    ]
}

#[test]
fn native_select_chooses_by_keyboard_and_list() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(None::<&'static str>);
        (
            value,
            page(
                basic::native_select(fruits(), value)
                    .placeholder("Select fruit")
                    .name("Fruit"),
            ),
        )
    });
    screen.get_by_text("Select fruit");
    let trigger = screen.get_by_role(Role::ComboBox, "Fruit");
    trigger.focus();
    screen.press_key(Key::ArrowDown);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some("apple"));
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.run_until_idle();
    assert_eq!(
        value.get_untracked(),
        Some("fig"),
        "the disabled option is skipped"
    );
    trigger.click();
    screen.run_until_idle();
    assert!(trigger.is_expanded() == Some(true));
    screen.get_by_role(Role::Option, "Banana").click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some("banana"));
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn radio_group_arrows_move_and_select() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(Some("apple"));
        let mut options = fruits();
        options[0] = ListItem::new("apple", "Apple").description("Crunchy");
        (value, page(radio_group("Fruit", options, value)))
    });
    screen.get_by_role(Role::RadioGroup, "Fruit");
    screen.get_by_text("Crunchy");
    let apple = screen.get_by_role(Role::RadioButton, "Apple");
    assert_eq!(apple.is_checked(), Some(true));
    apple.focus();
    screen.press_key(Key::ArrowDown);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some("banana"));
    screen.press_key(Key::ArrowDown);
    screen.run_until_idle();
    assert_eq!(
        value.get_untracked(),
        Some("fig"),
        "the disabled radio is skipped"
    );
    screen.get_by_role(Role::RadioButton, "Apple").click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some("apple"));
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn toggle_and_groups_track_pressed_state() {
    let (screen, (bold, multi, single)) =
        kivact::test::render_with_handle(crate::options(), || {
            let bold = signal(false);
            let multi = signal(vec!["b"]);
            let single = signal(Some("left"));
            let items = || {
                vec![
                    ListItem::new("a", "Alpha"),
                    ListItem::new("b", "Beta"),
                    ListItem::new("c", "Gamma"),
                ]
            };
            (
                (bold, multi, single),
                page((
                    kivact::ui::toggle("Bold", bold)
                        .icon(kivact::control::icon("bold"))
                        .variant(basic::ToggleVariant::Outline),
                    basic::toggle_group(items(), multi).name("Multi"),
                    basic::toggle_group(
                        vec![
                            ListItem::new("left", "Left"),
                            ListItem::new("right", "Right"),
                        ],
                        single,
                    )
                    .variant(basic::ToggleVariant::Outline)
                    .spacing(0.0)
                    .name("Single"),
                )),
            )
        });
    let toggle = screen.get_by_role(Role::Button, "Bold");
    assert_eq!(toggle.is_checked(), Some(false));
    toggle.click();
    screen.run_until_idle();
    assert!(bold.get_untracked());
    assert_eq!(toggle.is_checked(), Some(true));
    screen.get_by_role(Role::Button, "Alpha").click();
    screen.run_until_idle();
    assert_eq!(multi.get_untracked(), vec!["b", "a"]);
    screen.get_by_role(Role::Button, "Beta").click();
    screen.run_until_idle();
    assert_eq!(multi.get_untracked(), vec!["a"]);
    screen.get_by_role(Role::Button, "Right").click();
    screen.run_until_idle();
    assert_eq!(single.get_untracked(), Some("right"));
    screen.get_by_role(Role::Button, "Right").click();
    screen.run_until_idle();
    assert_eq!(single.get_untracked(), None, "pressing again clears");
    // Arrows rove the focus without pressing.
    screen.get_by_role(Role::Button, "Beta").focus();
    screen.press_key(Key::ArrowRight);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "Gamma").is_focused());
    assert_eq!(multi.get_untracked(), vec!["a"]);
    screen.press_key(Key::Space);
    screen.run_until_idle();
    assert_eq!(multi.get_untracked(), vec!["a", "c"]);
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn slider_keys_and_drag_change_the_binding() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(20.0);
        (
            value,
            page(
                slider("Volume", value)
                    .range(
                        kivact::ui::ScalarRange::new(0.0, 100.0)
                            .and_then(|range| range.with_step(5.0))
                            .expect("a valid range"),
                    )
                    .test_id("slider")
                    .style(style().width(200)),
            ),
        )
    });
    let slider = screen.get_by_role(Role::Slider, "Volume");
    assert_eq!(slider.value_range().2, Some(20.0));
    slider.focus();
    screen.press_key(Key::ArrowRight);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), 25.0);
    screen.press_key(Key::End);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), 100.0);
    screen.press_key(Key::Home);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), 0.0);
    slider.drag(
        kivact::draw::Point::new(6.0, 8.0),
        kivact::draw::Point::new(6.0 + 94.0, 8.0),
    );
    screen.run_until_idle();
    let now = value.get_untracked();
    assert!((45.0..=55.0).contains(&now), "dragged to the middle: {now}");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn range_slider_moves_the_nearest_of_two_thumbs() {
    let (screen, (low, high)) = kivact::test::render_with_handle(crate::options(), || {
        let (low, high) = (signal(20.0), signal(80.0));
        (
            (low, high),
            page(
                kivact::ui::range_slider("Price", low, high)
                    .range(
                        kivact::ui::ScalarRange::new(0.0, 100.0)
                            .and_then(|range| range.with_step(1.0))
                            .expect("a valid range"),
                    )
                    .test_id("range")
                    .style(style().width(200)),
            ),
        )
    });
    let value = move || (low.get_untracked(), high.get_untracked());
    let minimum = screen.get_by_role(Role::Slider, "Minimum");
    let maximum = screen.get_by_role(Role::Slider, "Maximum");
    minimum.focus();
    screen.press_key(Key::ArrowRight);
    screen.run_until_idle();
    assert_eq!(value(), (21.0, 80.0));
    maximum.focus();
    screen.press_key(Key::ArrowLeft);
    screen.run_until_idle();
    assert_eq!(value(), (21.0, 79.0));
    screen.press_key(Key::Home);
    screen.run_until_idle();
    assert_eq!(value(), (21.0, 21.0), "the thumbs never cross");
    // Dragging from near the right end moves the high thumb.
    screen.get_by_test_id("range").drag(
        kivact::draw::Point::new(6.0 + 188.0 * 0.2, 8.0),
        kivact::draw::Point::new(6.0 + 188.0 * 0.9, 8.0),
    );
    screen.run_until_idle();
    let (lo, hi) = value();
    assert_eq!(lo, 21.0);
    assert!((85.0..=95.0).contains(&hi), "high thumb followed: {hi}");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn slider_thumb_can_be_dragged() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(50.0);
        (
            value,
            page(slider("Level", value).style(style().width(200))),
        )
    });
    // The 12 px thumb of 50 is centered 6 + 0.5 * 188 = 100 px along the
    // track; 70 px to the left is 30 px along it.
    screen.get_by_role(Role::Slider, "Level").drag(
        kivact::draw::Point::new(6.0, 6.0),
        kivact::draw::Point::new(6.0 - 70.0, 6.0),
    );
    screen.run_until_idle();
    let now = value.get_untracked();
    assert!((10.0..=15.0).contains(&now), "{now}");
}

#[test]
fn caller_styles_win_over_the_kit_defaults() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let flag = signal(false);
        let pick = signal(None::<&'static str>);
        let pair = signal(Some("a"));
        (
            (),
            page((
                kivact::ui::toggle("Wide", flag)
                    .style(style().width(120))
                    .test_id("toggle"),
                basic::native_select(fruits(), pick)
                    .name("Fruit")
                    .style(style().width(240))
                    .test_id("select"),
                radio_group(
                    "Plan",
                    vec![ListItem::new("a", "A"), ListItem::new("b", "B")],
                    pair,
                )
                .style(style().width(150))
                .test_id("radios"),
                slider("Wide slider", signal(10.0))
                    .style(style().width(300))
                    .test_id("slider"),
                kivact::ui::field(
                    "X",
                    node().child(basic::input_group(basic::input_group_input(
                        "X",
                        signal(String::new()),
                    ))),
                )
                .style(style().width(260))
                .test_id("field"),
            )),
        )
    });
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    assert_eq!(width("toggle"), 120.0);
    assert_eq!(width("select"), 240.0);
    assert_eq!(width("radios"), 150.0);
    assert_eq!(width("slider"), 300.0);
    assert_eq!(width("field"), 260.0);
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn combobox_filters_chooses_and_clears() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(None::<&'static str>);
        (
            value,
            page(
                basic::combobox(fruits(), value)
                    .placeholder("Select fruit")
                    .show_clear(true)
                    .name("Fruit")
                    .style(style().width(240)),
            ),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Fruit");
    input.click();
    screen.type_text("an");
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "Banana");
    assert!(
        screen.query_by_role(Role::Option, "Apple").is_none(),
        "filtered out"
    );
    screen.press_key(Key::Enter);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some("banana"));
    assert_eq!(
        input.value().as_deref(),
        Some("Banana"),
        "the field shows the choice"
    );
    assert!(
        screen.query_by_role(Role::Option, "Banana").is_none(),
        "the list closed"
    );
    // Reopening lists every option again, with the chosen one checked.
    input.click();
    screen.press_key(Key::ArrowDown);
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "Apple");
    screen.get_by_role(Role::Option, "Fig").click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some("fig"));
    screen.get_by_role(Role::Button, "Clear").click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), None);
    assert_eq!(input.value().as_deref(), Some(""));
    // No match shows the empty text.
    input.click();
    screen.type_text("zzz");
    screen.run_until_idle();
    screen.get_by_text("No results found.");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn combobox_multiple_adds_and_removes_chips() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(vec!["apple"]);
        (
            value,
            page(
                basic::combobox(fruits(), value)
                    .name("Fruits")
                    .style(style().width(280)),
            ),
        )
    });
    screen.get_by_text("Apple");
    let input = screen.get_by_role(Role::TextBox, "Fruits");
    input.click();
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "Banana").click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), vec!["apple", "banana"]);
    screen.get_by_role(Role::Button, "Remove Apple").click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), vec!["banana"]);
    screen.get_by_role(Role::Option, "Banana").click();
    screen.run_until_idle();
    assert!(
        value.get_untracked().is_empty(),
        "choosing a chosen option removes it"
    );
    screen.assert_layout();
    assert_clean(&screen);
}

fn day(y: i32, m: u8, d: u8) -> CalendarDate {
    CalendarDate::new(y, m, d)
}

#[test]
fn calendar_moves_with_keys_changes_month_and_skips_disabled_dates() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(Some(day(2026, 10, 14)));
        (
            value,
            page(
                basic::calendar(value)
                    .default_month(CalendarMonth::new(2026, 10))
                    .week_start(kivact::i18n::Weekday::Sunday)
                    .disabled_dates(|d| d.day() == 16),
            ),
        )
    });
    screen.get_by_text("October 2026");
    screen.get_by_text("Sun");
    let cell = |d: u8| screen.get_by_role(Role::GridCell, &format!("2026-10-{d:02}"));
    assert!(cell(14).is_selected());
    assert!(cell(16).is_disabled());
    cell(14).focus();
    screen.press_key(Key::ArrowRight);
    assert!(cell(15).is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(cell(17).is_focused(), "the disabled 16th is skipped");
    screen.press_key(Key::Enter);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some(day(2026, 10, 17)));
    screen.press_key(Key::ArrowDown);
    assert!(cell(24).is_focused());
    screen.press_key(Key::Home);
    assert!(cell(18).is_focused(), "Sunday starts the week");
    screen.press_key(Key::PageDown);
    screen.run_until_idle();
    screen.get_by_text("November 2026");
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-11-18")
            .is_focused(),
        "the same day of the next month"
    );
    screen.get_by_role(Role::Button, "Previous month").click();
    screen.run_until_idle();
    screen.get_by_text("October 2026");
    screen.get_by_role(Role::Button, "Next month").click();
    screen.get_by_role(Role::Button, "Next month").click();
    screen.run_until_idle();
    screen.get_by_text("December 2026");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn calendar_range_takes_two_clicks_in_any_order() {
    let (screen, range) = kivact::test::render_with_handle(crate::options(), || {
        let range = signal(None::<DateRange>);
        (
            range,
            page(
                basic::calendar_range(range)
                    .default_month(CalendarMonth::new(2026, 10))
                    .number_of_months(2),
            ),
        )
    });
    screen.get_by_text("October 2026");
    screen.get_by_text("November 2026");
    let cell = |m: u8, d: u8| screen.get_by_role(Role::GridCell, &format!("2026-{m:02}-{d:02}"));
    cell(10, 20).click();
    screen.run_until_idle();
    assert_eq!(
        range.get_untracked(),
        Some(DateRange::new(day(2026, 10, 20), day(2026, 10, 20)))
    );
    cell(10, 12).click();
    screen.run_until_idle();
    let chosen = range.get_untracked().expect("a range");
    assert_eq!(
        (chosen.start(), chosen.end()),
        (day(2026, 10, 12), day(2026, 10, 20))
    );
    assert!(cell(10, 15).is_selected(), "days between are selected");
    // The next click starts over.
    cell(11, 3).click();
    cell(11, 9).click();
    screen.run_until_idle();
    let chosen = range.get_untracked().expect("a range");
    assert_eq!(
        (chosen.start(), chosen.end()),
        (day(2026, 11, 3), day(2026, 11, 9))
    );
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn calendar_dropdown_caption_changes_the_month() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                basic::calendar(signal(None::<CalendarDate>))
                    .default_month(CalendarMonth::new(2026, 10))
                    .caption(basic::CalendarCaption::Dropdown),
            ),
        )
    });
    screen.get_by_role(Role::ComboBox, "Month").click();
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "Feb").click();
    screen.run_until_idle();
    screen.get_by_role(Role::GridCell, "2026-02-10");
    screen.get_by_role(Role::ComboBox, "Year").click();
    screen.run_until_idle();
    // The open list focuses the shown year; two steps down choose 2028.
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    screen.run_until_idle();
    screen.get_by_role(Role::GridCell, "2028-02-10");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn calendar_dates_do_the_math() {
    assert_eq!(day(2026, 10, 31).add_months(1), day(2026, 11, 30));
    assert_eq!(day(2024, 2, 28).add_days(2), day(2024, 3, 1));
    assert_eq!(day(2026, 1, 1).add_days(-1), day(2025, 12, 31));
    assert_eq!(day(2026, 10, 3).weekday(), kivact::i18n::Weekday::Saturday);
    assert_eq!(
        CalendarMonth::new(2026, 12).add_months(1),
        CalendarMonth::new(2027, 1)
    );
    assert!(DateRange::new(day(2026, 5, 9), day(2026, 5, 1)).contains(day(2026, 5, 5)));
}

#[test]
fn date_picker_opens_a_popover_picks_a_day_and_closes() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(None::<CalendarDate>);
        (
            value,
            page(basic::date_picker("Date of birth", value).calendar(|c| {
                c.default_month(CalendarMonth::new(2026, 10))
                    .disabled_dates(|d| d.day() == 1)
            })),
        )
    });
    screen.get_by_text("Pick a date");
    let trigger = screen.get_by_role(Role::Button, "Date of birth");
    trigger.click();
    screen.run_until_idle();
    assert_eq!(trigger.is_expanded(), Some(true));
    screen.get_by_role(Role::Dialog, "Date of birth");
    screen.get_by_text("October 2026");
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-01")
            .is_disabled(),
        "calendar options are applied"
    );
    screen.get_by_role(Role::GridCell, "2026-10-14").click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some(day(2026, 10, 14)));
    assert!(
        screen
            .query_by_role(Role::Dialog, "Date of birth")
            .is_none(),
        "closed"
    );
    screen.get_by_text("October 14, 2026");
    assert!(trigger.is_focused(), "the focus returns to the button");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn date_range_picker_closes_after_the_second_click() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(None::<DateRange>);
        (
            value,
            page(
                basic::date_range_picker("Stay", value)
                    .format(|d| format!("{}/{}", d.month(), d.day()))
                    .calendar(|c| c.default_month(CalendarMonth::new(2026, 10))),
            ),
        )
    });
    screen.get_by_role(Role::Button, "Stay").click();
    screen.run_until_idle();
    screen.get_by_role(Role::GridCell, "2026-10-05").click();
    screen.run_until_idle();
    screen.get_by_role(Role::Dialog, "Stay");
    screen.get_by_role(Role::GridCell, "2026-10-09").click();
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Dialog, "Stay").is_none());
    assert_eq!(
        value.get_untracked(),
        Some(DateRange::new(day(2026, 10, 5), day(2026, 10, 9)))
    );
    screen.get_by_text("10/5 - 10/9");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn date_picker_keyboard_opens_and_chooses() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(Some(day(2026, 10, 14)));
        (
            value,
            page(
                basic::date_picker("Due", value)
                    .calendar(|c| c.default_month(CalendarMonth::new(2026, 10))),
            ),
        )
    });
    screen.get_by_role(Role::Button, "Due").focus();
    screen.press_key(Key::Enter);
    screen.run_until_idle();
    assert!(
        screen
            .get_by_role(Role::GridCell, "2026-10-14")
            .is_focused(),
        "the chosen day takes the focus"
    );
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::Enter);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some(day(2026, 10, 15)));
    assert!(screen.get_by_role(Role::Button, "Due").is_focused());
    // Escape closes without choosing.
    screen.press_key(Key::Enter);
    screen.run_until_idle();
    screen.press_key(Key::Escape);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Dialog, "Due").is_none());
    assert_eq!(value.get_untracked(), Some(day(2026, 10, 15)));
    assert_clean(&screen);
}

#[test]
fn questionnaire_steps_through_answers_and_submits() {
    let (screen, (step, answers, submitted)) =
        kivact::test::render_with_handle(crate::options(), || {
            let step = signal(0usize);
            let answers = signal(Vec::<Option<String>>::new());
            let submitted = signal(None::<Vec<Option<String>>>);
            let steps = vec![
                basic::QuestionnaireStep::choices(
                    "How did you hear about us?",
                    vec![
                        basic::QuestionnaireChoice::new("search", "Search")
                            .description("Google or Bing"),
                        basic::QuestionnaireChoice::new("friend", "A friend"),
                    ],
                )
                .description("Pick one."),
                basic::QuestionnaireStep::input("Your name")
                    .placeholder("Ada")
                    .optional(true),
                basic::QuestionnaireStep::input("Any comment?"),
            ];
            (
                (step, answers, submitted),
                page(
                    basic::questionnaire(steps, step, answers)
                        .on_submit(move |a: Vec<Option<String>>| submitted.set(Some(a))),
                ),
            )
        });
    screen.get_by_text("1 of 3");
    screen.get_by_text("How did you hear about us?");
    assert!(screen.query_by_role(Role::Button, "Previous").is_none());
    assert!(
        screen.get_by_role(Role::Button, "Next").is_disabled(),
        "unanswered"
    );
    screen.get_by_role(Role::RadioButton, "Search").click();
    screen.run_until_idle();
    assert_eq!(answers.get_untracked(), vec![Some("search".to_string())]);
    screen.get_by_role(Role::Button, "Next").click();
    screen.run_until_idle();
    assert_eq!(step.get_untracked(), 1);
    screen.get_by_text("2 of 3");
    screen.get_by_role(Role::Button, "Previous").click();
    screen.run_until_idle();
    assert_eq!(step.get_untracked(), 0);
    screen.get_by_role(Role::Button, "Next").click();
    screen.run_until_idle();
    // The optional step can be skipped.
    screen.get_by_role(Role::Button, "Skip").click();
    screen.run_until_idle();
    assert_clean(&screen);
    assert_eq!(step.get_untracked(), 2);
    assert!(screen.get_by_role(Role::Button, "Submit").is_disabled());
    screen.get_by_role(Role::TextBox, "Any comment?").click();
    screen.type_text("Nice");
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Submit").click();
    screen.run_until_idle();
    assert_eq!(
        submitted.get_untracked(),
        Some(vec![
            Some("search".to_string()),
            None,
            Some("Nice".to_string())
        ])
    );
    screen.assert_layout();
    assert_clean(&screen);
}

/// Every form component of the kit on two screens, for both color modes.
fn gallery_one() -> Node {
    let text_value = signal(String::new());
    let code = signal(String::new());
    let pick = signal(Some("banana"));
    let choice = signal(Some("apple"));
    let flag = signal(true);
    let multi = signal(vec!["a"]);
    let single = signal(Some("a"));
    let level = signal(40.0);
    let span = (signal(20.0), signal(70.0));
    let items = || vec![ListItem::new("a", "Align"), ListItem::new("b", "Bold")];
    page(
        column((
            basic::field_group((
                kivact::ui::field("Email", kivact::ui::text_input("Email", text_value))
                    .hint("We never share it.")
                    .error(Some("Looks wrong".to_string())),
                basic::field_separator().text("Or"),
                basic::field_set((
                    kivact::ui::field("Newsletter", kivact::ui::switch("Newsletter", flag))
                        .style(style().direction(kivact::style::Direction::Row)),
                    kivact::ui::field(
                        "Favorite",
                        basic::native_select(fruits(), pick).name("Fruit"),
                    ),
                ))
                .legend("Preferences")
                .description("Pick what you like."),
            )),
            basic::input_group(basic::input_group_input("Query", text_value).placeholder("Search"))
                .start(kivact::control::icon("search"))
                .end(basic::input_group_button("Go").size(basic::InputGroupButtonSize::Sm)),
            basic::input_otp("Code", code, 6).groups(&[3, 3]),
            radio_group("Favorite fruit", fruits(), choice),
            row((
                kivact::ui::toggle("Bold", flag).variant(basic::ToggleVariant::Outline),
                basic::toggle_group(items(), multi).icon_only(false),
                basic::toggle_group(items(), single)
                    .variant(basic::ToggleVariant::Outline)
                    .spacing(0.0),
            ))
            .style(style().gap(12)),
            slider("Level", level).style(style().width(240)),
            kivact::ui::range_slider("Span", span.0, span.1).style(style().width(240)),
        ))
        .style(style().gap(16)),
    )
    .style(style().width(420).shrink(0))
}

fn gallery_two() -> Node {
    let combo = signal(Some("fig"));
    let many = signal(vec!["apple", "banana"]);
    let date = signal(Some(day(2026, 10, 14)));
    let range = signal(Some(DateRange::new(day(2026, 10, 5), day(2026, 10, 9))));
    let step = signal(0usize);
    page(
        column((
            basic::combobox(fruits(), combo)
                .name("Combo")
                .show_clear(true),
            basic::combobox(fruits(), many).name("Many"),
            row((
                basic::calendar(date).default_month(CalendarMonth::new(2026, 10)),
                basic::calendar_range(range)
                    .default_month(CalendarMonth::new(2026, 10))
                    .disabled_dates(|d| d.day() == 20),
            ))
            .style(style().gap(12)),
            basic::date_picker("Birthday", date),
            basic::date_range_picker("Stay", range),
            basic::questionnaire(
                vec![basic::QuestionnaireStep::choices(
                    "Pick one",
                    vec![
                        basic::QuestionnaireChoice::new("a", "A").description("First"),
                        basic::QuestionnaireChoice::new("b", "B"),
                    ],
                )],
                step,
                signal(Vec::new()),
            ),
        ))
        .style(style().gap(16)),
    )
    .style(style().width(480).shrink(0))
}

#[test]
fn forms_galleries_are_clean_in_light_and_dark() {
    for (mode, background) in [
        (kivact::style::ThemeMode::Light, 0xffffff),
        (kivact::style::ThemeMode::Dark, 0x0a0a0a),
    ] {
        for build in [gallery_one, gallery_two] {
            let (screen, ()) = kivact::test::render_with_handle(
                RenderOptions {
                    theme: basic::theme(mode),
                    height: 1100.0,
                    reduced_motion: true,
                    ..RenderOptions::from_env()
                },
                move || ((), build()),
            );
            assert!(distance(pixel(&screen, 2.0, 2.0), background) <= 2);
            screen.assert_layout();
            assert_clean(&screen);
        }
    }
}

#[test]
fn labels_activate_their_controls_like_a_click() {
    let (screen, (agreed, notify, query)) =
        kivact::test::render_with_handle(crate::options(), || {
            let agreed = signal(false);
            let notify = signal(false);
            let query = signal(String::new());
            let terms = node_ref();
            (
                (agreed, notify, query),
                page((
                    row((
                        kivact::ui::checkbox("Terms", agreed)
                            .hide_label(true)
                            .node_ref(terms),
                        basic::label("Accept terms").target(terms).test_id("terms"),
                    ))
                    .style(style().gap(8)),
                    kivact::ui::field(
                        "Notify me",
                        kivact::ui::switch("Notifications", notify).hide_label(true),
                    )
                    .style(style().direction(kivact::style::Direction::Row)),
                    basic::input_group(basic::input_group_input("Search", query))
                        .start(basic::input_group_text("https://").test_id("addon")),
                )),
            )
        });
    screen.get_by_test_id("terms").click();
    assert!(agreed.get_untracked(), "the label toggles its check box");
    assert!(screen.get_by_role(Role::CheckBox, "Terms").is_focused());
    let notifications = screen.get_by_role(Role::Switch, "Notify me");
    screen.get_by_text("Notify me").click();
    assert!(notify.get_untracked());
    assert!(notifications.is_focused());
    screen.get_by_test_id("addon").click();
    assert!(
        screen.get_by_role(Role::TextBox, "Search").is_focused(),
        "an addon focuses the group's input"
    );
    screen.type_text("kivact");
    assert_eq!(query.get_untracked(), "kivact");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn a_small_switch_is_narrower_and_an_invalid_input_turns_destructive() {
    let (screen, bad) = kivact::test::render_with_handle(crate::options(), || {
        let on = signal(true);
        let bad = signal(false);
        (
            bad,
            page((
                kivact::ui::switch("Small", on)
                    .size(kivact::ui::ControlSize::Small)
                    .hide_label(true)
                    .test_id("small"),
                kivact::ui::switch("Default", on)
                    .hide_label(true)
                    .test_id("default"),
                kivact::ui::text_input("Email", signal(String::new()))
                    .invalid(bad)
                    .test_id("email"),
            )),
        )
    });
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    // `w-[24px]` and `w-[32px]` tracks, each before an 8 px label gap.
    assert_eq!(width("default") - width("small"), 8.0);
    let border = |screen: &kivact::test::Screen| pixel_in(screen, "email", 0.5);
    let calm = border(&screen);
    bad.set(true);
    screen.run_until_idle();
    let red = border(&screen);
    assert!(
        distance(calm, red) > 40,
        "the border turns destructive: {calm:06x} -> {red:06x}"
    );
    assert_clean(&screen);
}

#[test]
fn backspace_in_the_empty_search_removes_the_last_chip() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(vec!["apple", "banana"]);
        (
            value,
            page(
                basic::combobox(fruits(), value)
                    .name("Fruits")
                    .style(style().width(280)),
            ),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Fruits");
    input.click();
    screen.type_text("c");
    screen.press_key(Key::Backspace);
    assert_eq!(
        value.get_untracked(),
        vec!["apple", "banana"],
        "a typed letter goes first"
    );
    screen.press_key(Key::Backspace);
    assert_eq!(value.get_untracked(), vec!["apple"]);
    screen.press_key(Key::Backspace);
    screen.press_key(Key::Backspace);
    assert!(value.get_untracked().is_empty());
    assert_clean(&screen);
}
