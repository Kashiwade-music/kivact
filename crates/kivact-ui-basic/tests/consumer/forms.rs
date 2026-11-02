//! Role: Forms tests: fields, selects, choices, sliders, dates and questionnaire.

use crate::support::assert_clean;
use kivact::i18n::{CalendarDate, CalendarMonth};
use kivact::prelude::*;
use kivact::style::Align;
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
    let (screen, error) = kivact::test::render_with_handle(crate::support::options(), || {
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
fn input_otp_fills_slots_completes_and_backspaces() {
    let (screen, (code, done)) =
        kivact::test::render_with_handle(crate::support::options(), || {
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
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), || {
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
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), || {
        let value = signal(Some("apple"));
        let mut options = fruits();
        options[0] = ListItem::new("apple", "Apple").description("Crunchy");
        (value, page(radio_group("Fruit", options, value)))
    });
    screen.get_by_role(Role::RadioGroup, "Fruit");
    screen.get_by_text("Crunchy");
    let apple = screen.get_by_role(Role::RadioButton, "Apple");
    assert_eq!(apple.checked(), Some(CheckedState::Checked));
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
fn slider_keys_and_drag_change_the_binding() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), || {
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
fn combobox_filters_chooses_and_clears() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), || {
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

fn day(y: i32, m: u8, d: u8) -> CalendarDate {
    CalendarDate::new(y, m, d)
}

#[test]
fn calendar_moves_with_keys_changes_month_and_skips_disabled_dates() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), || {
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
fn date_picker_opens_a_popover_picks_a_day_and_closes() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), || {
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
