//! Role: Forms state tests: disabled, non-finite, mirrored, formats, groups, layout.

use super::{assert_clean, distance, pixel_in};
use kivact::draw::Point;
use kivact::i18n::{CalendarDate, DateRange, Messages};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{RenderOptions, Screen};
use kivact::ui::{ButtonVariant, ListItem};
use kivact::view::{Key, Orientation};
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(16)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

fn fruits() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("fig", "Fig"),
    ]
}

fn day(y: i32, m: u8, d: u8) -> CalendarDate {
    CalendarDate::new(y, m, d)
}

#[test]
fn a_disabled_combobox_is_inert_and_dims_once() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(Some("apple"));
        (
            value,
            page((
                basic::combobox(fruits(), value)
                    .name("Fruit")
                    .show_clear(true)
                    .disabled(true)
                    .style(style().width(240))
                    .test_id("off"),
                basic::combobox(fruits(), signal(Some("apple")))
                    .name("Other")
                    .style(style().width(240))
                    .test_id("on"),
            )),
        )
    });
    let off = screen.get_by_test_id("off");
    let inside = screen.within(off);
    let input = screen.get_by_role(Role::TextBox, "Fruit");
    assert!(input.is_disabled());
    assert!(
        inside
            .get_by_role(Role::Button, "Toggle options")
            .is_disabled()
    );
    assert!(inside.get_by_role(Role::Button, "Clear").is_disabled());
    input.force_click();
    screen.run_until_idle();
    assert!(
        screen.query_by_role(Role::Option, "Banana").is_none(),
        "pressing the field does not open the list"
    );
    inside
        .get_by_role(Role::Button, "Toggle options")
        .force_click();
    inside.get_by_role(Role::Button, "Clear").force_click();
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Option, "Banana").is_none());
    assert_eq!(value.get_untracked(), Some("apple"), "Clear did nothing");
    // The border at half opacity over white: #e5e5e5 -> #f2f2f2. Dimming the
    // group and the field both would give #f9f9f9.
    let border = pixel_in(&screen, "off", 0.5);
    assert!(distance(border, 0xf2f2f2) <= 3, "dimmed once: {border:06x}");
    assert!(distance(pixel_in(&screen, "on", 0.5), 0xe5e5e5) <= 3);
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn a_disabled_multiple_combobox_ignores_its_chips() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(vec!["apple", "fig"]);
        (
            value,
            page(
                basic::combobox(fruits(), value)
                    .name("Fruits")
                    .disabled(true)
                    .style(style().width(280)),
            ),
        )
    });
    assert!(screen.get_by_role(Role::TextBox, "Fruits").is_disabled());
    let remove = screen.get_by_role(Role::Button, "Remove Apple");
    assert!(remove.is_disabled());
    remove.force_click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), vec!["apple", "fig"]);
    assert_clean(&screen);
}

#[test]
fn a_custom_filter_decides_which_options_match() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                basic::combobox(fruits(), signal(None::<&'static str>))
                    .name("Fruit")
                    // Only labels that start with the text.
                    .filter(|query, label| label.to_lowercase().starts_with(&query.to_lowercase()))
                    .style(style().width(240)),
            ),
        )
    });
    screen.get_by_role(Role::TextBox, "Fruit").click();
    screen.type_text("an");
    screen.run_until_idle();
    assert!(
        screen.query_by_role(Role::Option, "Banana").is_none(),
        "'an' is inside Banana but does not start it"
    );
    screen.get_by_text("No results found.");
    assert_clean(&screen);
}

#[test]
fn a_disabled_slider_ignores_keys_and_a_drag() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(20.0);
        (
            value,
            page(
                slider("Volume", value)
                    .disabled(true)
                    .style(style().width(200)),
            ),
        )
    });
    let slider = screen.get_by_role(Role::Slider, "Volume");
    assert!(slider.is_disabled());
    slider.force_click();
    screen.press_key(Key::ArrowRight);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), 20.0);
    assert_clean(&screen);
}

#[test]
fn sliders_mirror_in_a_right_to_left_boundary() {
    let (screen, (single, low, high)) = kivact::test::render_with_handle(crate::options(), || {
        let single = signal(25.0);
        let (low, high) = (signal(20.0), signal(80.0));
        (
            (single, low, high),
            page(kivact::view::provide_direction(
                kivact::view::ReadingDirection::Rtl,
                move || {
                    column((
                        slider("Level", single)
                            .range(
                                kivact::ui::ScalarRange::new(0.0, 100.0)
                                    .and_then(|range| range.with_step(1.0))
                                    .expect("a valid range"),
                            )
                            .style(style().width(200))
                            .test_id("level"),
                        kivact::ui::range_slider("Span", low, high)
                            .range(
                                kivact::ui::ScalarRange::new(0.0, 100.0)
                                    .and_then(|range| range.with_step(1.0))
                                    .expect("a valid range"),
                            )
                            .style(style().width(200))
                            .test_id("span"),
                    ))
                    .style(style().gap(12))
                },
            )),
        )
    });
    // ArrowLeft increases, ArrowRight decreases.
    let level = screen.get_by_role(Role::Slider, "Level");
    level.focus();
    screen.press_key(Key::ArrowLeft);
    assert_eq!(single.get_untracked(), 26.0);
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::ArrowRight);
    assert_eq!(single.get_untracked(), 24.0);
    // The maximum is at the left: dragging to the left end raises the value.
    screen
        .get_by_test_id("level")
        .drag(Point::new(100.0, 8.0), Point::new(6.0, 8.0));
    screen.run_until_idle();
    assert!(single.get_untracked() > 90.0, "{}", single.get_untracked());
    // The low thumb of a range sits to the right of the high one.
    let span = screen.within(screen.get_by_test_id("span"));
    let minimum = span.get_by_role(Role::Slider, "Minimum");
    let maximum = span.get_by_role(Role::Slider, "Maximum");
    assert!(minimum.rect().expect("laid out").x > maximum.rect().expect("laid out").x);
    minimum.focus();
    screen.press_key(Key::ArrowLeft);
    assert_eq!((low.get_untracked(), high.get_untracked()), (21.0, 80.0));
    screen.press_key(Key::ArrowRight);
    assert_eq!((low.get_untracked(), high.get_untracked()), (20.0, 80.0));
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn a_joined_toggle_group_draws_each_shared_border_once_along_its_axis() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        let outline = |orientation| {
            basic::toggle_group(fruits(), signal(None::<&'static str>))
                .variant(basic::ToggleVariant::Outline)
                .spacing(0.0)
                .orientation(orientation)
        };
        (
            (),
            page((
                outline(Orientation::Vertical).test_id("column"),
                outline(Orientation::Horizontal).test_id("row"),
                basic::toggle_group(fruits(), signal(None::<&'static str>))
                    .variant(basic::ToggleVariant::Outline)
                    .test_id("apart"),
            )),
        )
    });
    let within_group = |id: &'static str| screen.within(screen.get_by_test_id(id));
    let rect = |id: &'static str, name: &str| {
        within_group(id)
            .get_by_role(Role::Button, name)
            .rect()
            .expect("laid out")
    };
    let (a, b) = (rect("column", "Apple"), rect("column", "Banana"));
    // Every item but the first drops the border it shares (`border-t-0`).
    assert_eq!(b.y, a.y + a.height, "stacked, sharing one border");
    assert_eq!(b.x, a.x, "and not shifted sideways");
    assert_eq!(b.width, a.width, "a column stretches its items");
    let (a, b) = (rect("row", "Apple"), rect("row", "Banana"));
    assert_eq!(b.x, a.x + a.width, "side by side, sharing one border");
    assert_eq!(b.y, a.y);
    // `px-2` instead of `px-2.5` when joined.
    let apart = rect("apart", "Apple");
    assert_eq!(apart.width - rect("row", "Apple").width, 4.0);
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn input_group_buttons_sit_closer_to_the_border_and_a_disabled_group_dims() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page((
                basic::input_group(basic::input_group_input("A", signal(String::new())))
                    .start(basic::input_group_button("Go").test_id("button"))
                    .style(style().width(240))
                    .test_id("with-button"),
                basic::input_group(basic::input_group_input("B", signal(String::new())))
                    .start(basic::input_group_text("https://").test_id("text"))
                    .style(style().width(240))
                    .test_id("with-text"),
                basic::input_group(
                    basic::input_group_input("C", signal(String::new())).disabled(true),
                )
                .start(kivact::control::icon("search"))
                .disabled(true)
                .style(style().width(240))
                .test_id("off"),
            )),
        )
    });
    let x = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").x;
    let button_offset = x("button") - x("with-button");
    let text_offset = x("text") - x("with-text");
    assert!(
        button_offset < text_offset - 3.0,
        "a button addon starts 0.3rem closer: {button_offset} against {text_offset}"
    );
    let border = pixel_in(&screen, "off", 0.5);
    assert!(
        distance(border, 0xf2f2f2) <= 3,
        "has-disabled:opacity-50: {border:06x}"
    );
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn a_date_picker_is_as_wide_as_its_text_and_formats_through_messages() {
    let options = RenderOptions {
        theme: basic::theme(kivact::style::ThemeMode::Light),
        messages: Messages::new()
            .with("basic.date_picker.format", "{year}/{month}/{day}")
            .with("basic.date_picker.range", "{start} to {end}"),
        reduced_motion: true,
        ..RenderOptions::from_env()
    };
    let (screen, ()) = kivact::test::render_with_handle(options, || {
        (
            (),
            page((
                basic::date_picker("Plain", signal(Some(day(2026, 10, 14)))).test_id("plain"),
                basic::date_picker("Wide", signal(None::<CalendarDate>))
                    .style(style().width(280))
                    .test_id("wide"),
                basic::date_picker("Own", signal(Some(day(2026, 10, 14))))
                    .format(|d| format!("{}-{}", d.month(), d.day())),
                basic::date_range_picker(
                    "Stay",
                    signal(Some(DateRange::new(day(2026, 10, 5), day(2026, 10, 9)))),
                ),
            )),
        )
    });
    screen.get_by_text("2026/October/14");
    screen.get_by_text("10-14");
    screen.get_by_text("2026/October/5 to 2026/October/9");
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    assert!(
        width("plain") < 280.0,
        "no width of the kit's own: {}",
        width("plain")
    );
    assert_eq!(width("wide"), 280.0);
    assert_clean(&screen);
}

/// The pixels of the node `id`, for comparing a screen with another.
fn pixels_of(screen: &Screen, id: &str) -> Vec<u32> {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    let scale = screen.scale();
    let shot = screen.screenshot();
    let mut out = Vec::new();
    for y in (r.y * scale) as u32..((r.y + r.height) * scale) as u32 {
        for x in (r.x * scale) as u32..((r.x + r.width) * scale) as u32 {
            out.push(shot.pixel(x, y).unwrap_or(0));
        }
    }
    out
}

#[test]
fn disclosure_and_combobox_buttons_follow_a_live_theme_switch() {
    fn gallery() -> impl IntoView {
        page((
            kivact::ui::disclosure("Section", signal(true), || text("Open body"))
                .style(style().width(240))
                .test_id("accordion"),
            basic::combobox(fruits(), signal(Some("apple")))
                .name("Fruit")
                .show_clear(true)
                .style(style().width(240))
                .test_id("combobox"),
            basic::collapsible(signal(true))
                .content(|| text("Collapsed body"))
                .attach(kivact::ui::button("More").variant(ButtonVariant::Ghost)),
        ))
    }
    fn mode(dark: bool) -> kivact::style::ThemeMode {
        if dark {
            kivact::style::ThemeMode::Dark
        } else {
            kivact::style::ThemeMode::Light
        }
    }
    let (switched, dark) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let dark = signal(false);
            (
                dark,
                kivact::view::provide_theme(move || basic::theme(mode(dark.get())), gallery),
            )
        },
    );
    dark.set(true);
    switched.run_until_idle();
    let (fresh, ()) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Dark),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || ((), gallery()),
    );
    for id in ["accordion", "combobox"] {
        assert!(
            pixels_of(&switched, id) == pixels_of(&fresh, id),
            "{id}: a live switch draws what a fresh dark mount draws"
        );
    }
    assert_clean(&switched);
}

#[test]
fn input_group_block_addons_pad_the_addon_rows_and_make_the_input_full_height() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                column((
                    basic::input_group(basic::input_group_input("Plain", signal(String::new())))
                        .test_id("plain"),
                    basic::input_group(basic::input_group_input("Above", signal(String::new())))
                        .block_start(move || basic::input_group_text("Heading"))
                        .test_id("above"),
                    basic::input_group(basic::input_group_input("Below", signal(String::new())))
                        .block_end(move || basic::input_group_text("Footer"))
                        .test_id("below"),
                    basic::input_group(basic::input_group_textarea("Notes", signal(String::new())))
                        .block_end(move || basic::input_group_text("Counter"))
                        .test_id("notes"),
                ))
                .style(style().gap(16).width(320)),
            ),
        )
    });
    let by_id = |id: &str| screen.get_by_test_id(id).rect().expect("laid out");
    let input = |name: &str| {
        screen
            .get_by_role(Role::TextBox, name)
            .rect()
            .expect("laid out")
    };
    let words = |text: &str| screen.get_by_text(text).rect().expect("laid out");
    assert_eq!(input("Plain").height, 30.0, "h-8 inside the 1 px border");
    assert_eq!(by_id("plain").height, 32.0);

    // `px-2.5 pt-2` and `py-1.5` around the heading, then the full-height input.
    let (group, heading, above) = (by_id("above"), words("Heading"), input("Above"));
    assert_eq!(input("Above").height, 32.0, "h-8 in an h-auto group");
    // The group's 1 px border takes room inside it, as CSS's border-box.
    assert_eq!(heading.y - group.y, 9.0, "border + pt-2");
    assert_eq!(heading.x - group.x, 11.0, "border + px-2.5");
    assert_eq!(above.y - (heading.y + heading.height), 6.0, "pb-1.5");
    assert_eq!(
        group.y + group.height,
        above.y + above.height + 1.0,
        "border"
    );

    // `pb-2` under the footer, `pt-1.5` over it.
    let (group, footer, below) = (by_id("below"), words("Footer"), input("Below"));
    assert_eq!(below.height, 32.0);
    assert_eq!(footer.y - (below.y + below.height), 6.0, "pt-1.5");
    assert_eq!(
        group.y + group.height - (footer.y + footer.height),
        9.0,
        "pb-2 + border"
    );

    // A text area keeps its own height and padding; the group grows around it.
    let (group, counter, notes) = (by_id("notes"), words("Counter"), input("Notes"));
    assert!(notes.height >= 64.0, "min-h-16");
    assert_eq!(counter.y - (notes.y + notes.height), 6.0);
    assert!(group.height > notes.height + counter.height);
    screen.assert_layout();
    assert_clean(&screen);
}
