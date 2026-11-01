//! Role: Chart and data table tests: colors, tooltips, filtering, sorting, paging.

use super::{assert_caller_won, assert_clean, caller_style, distance, pixel};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::RenderOptions;
use kivact::test::Screen;
use kivact::view::{Key, Orientation, SortDirection};
use kivact_ui_basic as basic;

fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(16)
            .gap(16)
            .grow(1)
            .align_items(Align::Start),
    )
}

fn options(mode: kivact::style::ThemeMode) -> RenderOptions {
    RenderOptions {
        theme: basic::theme(mode),
        width: 640.0,
        height: 2400.0,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

fn config() -> basic::ChartConfig {
    basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop").color(rgb(0xe11d48)),
        basic::ChartSeries::new("mobile", "Mobile").color(rgb(0x2563eb)),
    ])
}

fn rows() -> Vec<basic::ChartRow> {
    vec![
        basic::ChartRow::new("January", [186.0, 80.0]),
        basic::ChartRow::new("February", [305.0, 200.0]),
        basic::ChartRow::new("March", [237.0, 120.0]),
        basic::ChartRow::new("April", [73.0, 190.0]),
    ]
}

#[derive(Clone, PartialEq)]
struct Payment {
    id: u32,
    status: &'static str,
    email: &'static str,
    amount: f64,
}

fn payments() -> Vec<Payment> {
    let row = |id, status, email, amount| Payment {
        id,
        status,
        email,
        amount,
    };
    vec![
        row(1, "success", "ken99@example.com", 316.0),
        row(2, "success", "abe45@example.com", 242.0),
        row(3, "processing", "monserrat44@example.com", 837.0),
        row(4, "failed", "silas22@example.com", 874.0),
        row(5, "failed", "carmella@example.com", 721.0),
        row(6, "pending", "dora71@example.com", 99.0),
        row(7, "success", "zed@example.com", 15.0),
    ]
}

/// The handles of a mounted payments table.
#[derive(Clone, Copy)]
struct Handles {
    query: Signal<String>,
    selected: Signal<Vec<u32>>,
    sort: Signal<Option<(usize, kivact::view::SortDirection)>>,
    page: Signal<usize>,
    visible: Signal<Vec<bool>>,
    acted: Signal<Option<u32>>,
}

fn payments_table() -> (Handles, impl IntoView) {
    let handles = Handles {
        query: signal(String::new()),
        selected: signal(Vec::new()),
        sort: signal(None),
        page: signal(0),
        visible: signal(vec![true; 3]),
        acted: signal(None),
    };
    let visible = handles.visible;
    let columns = vec![
        kivact::ui::DataColumn::new("Status", |p: &Payment| p.status.to_string())
            .visible(move || visible.with(|v| v[0])),
        kivact::ui::DataColumn::new("Email", |p: &Payment| p.email.to_string())
            .compare(|a, b| a.email.cmp(b.email))
            .filter(|p: &Payment, query: &str| p.email.contains(query))
            .visible(move || visible.with(|v| v[1])),
        kivact::ui::DataColumn::new("Amount", |p: &Payment| format!("${:.2}", p.amount))
            .compare(|a, b| a.amount.total_cmp(&b.amount))
            .numeric(true)
            .width(100.0)
            .visible(move || visible.with(|v| v[2])),
    ];
    let view = kivact::ui::data_table(payments(), |p: &Payment| p.id, columns)
        .row_label(|p: &Payment| p.email.to_string())
        .selection(handles.selected)
        .filter(handles.query)
        .sort(handles.sort)
        .page(handles.page, 3)
        .toolbar(move |context| {
            let query = context.filter.expect("the table has a filter");
            let page = context.page.map(|(page, _)| page).expect("the table pages");
            let filter = kivact::ui::text_input(
                "Filter emails...",
                kivact::binding(query, move |text: String| {
                    query.set(text);
                    page.set(0);
                }),
            )
            .placeholder("Filter emails...");
            let titles = ["Status", "Email", "Amount"];
            let entries = kivact::memo(move || {
                let shown = visible.get();
                titles
                    .iter()
                    .enumerate()
                    .map(|(i, title)| {
                        kivact::ui::MenuEntry::item(i, *title)
                            .checked(shown[i])
                            .keep_open(true)
                    })
                    .collect::<Vec<_>>()
            });
            let columns_menu = kivact::ui::menu("Columns", entries, move |index: usize| {
                visible.update(|shown| shown[index] = !shown[index]);
            })
            .attach(
                kivact::ui::button("Columns")
                    .variant(kivact::ui::ButtonVariant::Outline)
                    .into_node(),
            );
            row((filter, columns_menu)).style(style().gap(8))
        })
        .row_actions(40.0, move |row: Value<Payment>| {
            let id = row.with(|p| p.id);
            kivact::ui::menu(
                "Actions",
                vec![kivact::ui::MenuEntry::item(id, "Copy payment ID")],
                move |id: u32| handles.acted.set(Some(id)),
            )
            .attach(
                kivact::ui::button(format!("Open menu {id}"))
                    .variant(kivact::ui::ButtonVariant::Outline)
                    .hide_label(true)
                    .icon(kivact::control::icon("ellipsis"))
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .into_node(),
            )
            .into_view()
        });
    (handles, page(view.style(style().width(600))))
}

fn mount_payments() -> (Screen, Handles) {
    kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), payments_table)
}

/// The window y of the text `what`.
fn y_of(screen: &Screen, what: &str) -> f32 {
    screen.get_by_text(what).rect().expect("laid out").y
}

#[test]
fn data_table_filters_rows_and_returns_to_the_first_page() {
    let (screen, handles) = mount_payments();
    screen.get_by_text("0 of 7 row(s) selected.");
    screen.get_by_text("ken99@example.com");
    assert!(screen.query_by_text("silas22@example.com").is_none());
    handles.page.set(2);
    screen.run_until_idle();
    screen.get_by_text("zed@example.com");
    screen
        .get_by_role(Role::TextBox, "Filter emails...")
        .click();
    screen.type_text("sil");
    screen.run_until_idle();
    assert_eq!(handles.query.get_untracked(), "sil");
    assert_eq!(handles.page.get_untracked(), 0, "a new filter shows page 1");
    screen.get_by_text("silas22@example.com");
    assert!(screen.query_by_text("ken99@example.com").is_none());
    screen.get_by_text("0 of 1 row(s) selected.");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn data_table_shows_an_empty_state_when_nothing_matches() {
    let (screen, handles) = mount_payments();
    handles.query.set("nobody".into());
    screen.run_until_idle();
    screen.get_by_text("No results found.");
    screen.get_by_text("0 of 0 row(s) selected.");
    assert!(screen.get_by_role(Role::Button, "Next").is_disabled());
    assert_clean(&screen);
}

#[test]
fn data_table_sorts_with_the_header_buttons() {
    let (screen, handles) = mount_payments();
    let email = || screen.get_by_role(Role::ColumnHeader, "Email");
    assert_eq!(email().sort(), SortDirection::None);
    email().click();
    screen.run_until_idle();
    assert_eq!(
        handles.sort.get_untracked(),
        Some((1, SortDirection::Ascending))
    );
    assert_eq!(email().sort(), SortDirection::Ascending);
    assert!(
        y_of(&screen, "abe45@example.com") < y_of(&screen, "dora71@example.com"),
        "ascending by email"
    );
    email().click();
    screen.run_until_idle();
    assert_eq!(
        handles.sort.get_untracked(),
        Some((1, SortDirection::Descending))
    );
    assert!(
        y_of(&screen, "zed@example.com") < y_of(&screen, "silas22@example.com"),
        "descending by email"
    );
    screen.get_by_role(Role::ColumnHeader, "Amount").click();
    screen.run_until_idle();
    assert_eq!(
        handles.sort.get_untracked(),
        Some((2, SortDirection::Ascending))
    );
    screen.get_by_text("$15.00");
    assert_clean(&screen);
}

#[test]
fn data_table_selects_the_rows_of_the_page_and_keeps_partial_selection() {
    let (screen, handles) = mount_payments();
    let all = || screen.get_by_role(Role::CheckBox, "Select all");
    assert_eq!(all().is_checked(), Some(false));
    all().click();
    screen.run_until_idle();
    assert_eq!(handles.selected.get_untracked(), vec![1, 2, 3]);
    assert_eq!(all().is_checked(), Some(true));
    screen.get_by_text("3 of 7 row(s) selected.");
    assert!(
        screen
            .get_by_role(Role::Row, "abe45@example.com")
            .is_selected()
    );
    screen
        .get_by_role(Role::CheckBox, "Select abe45@example.com")
        .click();
    screen.run_until_idle();
    assert_eq!(handles.selected.get_untracked(), vec![1, 3]);
    assert!(
        all().is_indeterminate(),
        "some rows of the page are selected"
    );
    assert!(
        !screen
            .get_by_role(Role::Row, "abe45@example.com")
            .is_selected()
    );
    // Selection belongs to the row keys: it survives a page change.
    screen.get_by_role(Role::Button, "Next").click();
    screen.run_until_idle();
    assert_eq!(handles.page.get_untracked(), 1);
    assert_eq!(all().is_checked(), Some(false));
    screen.get_by_text("2 of 7 row(s) selected.");
    screen
        .get_by_role(Role::CheckBox, "Select silas22@example.com")
        .click();
    assert_eq!(handles.selected.get_untracked(), vec![1, 3, 4]);
    // Clicking the text of a cell is not a selection gesture.
    screen.get_by_text("carmella@example.com").click();
    screen.run_until_idle();
    assert_eq!(handles.selected.get_untracked(), vec![1, 3, 4]);
    // The header check box acts on the rows of its page only.
    all().click();
    screen.run_until_idle();
    assert_eq!(handles.selected.get_untracked(), vec![1, 3, 4, 5, 6]);
    all().click();
    screen.run_until_idle();
    assert_eq!(handles.selected.get_untracked(), vec![1, 3]);
    assert_clean(&screen);
}

#[test]
fn data_table_pages_with_previous_and_next() {
    let (screen, handles) = mount_payments();
    let previous = || screen.get_by_role(Role::Button, "Previous");
    let next = || screen.get_by_role(Role::Button, "Next");
    assert!(previous().is_disabled());
    next().click();
    screen.run_until_idle();
    assert_eq!(handles.page.get_untracked(), 1);
    screen.get_by_text("silas22@example.com");
    assert!(screen.query_by_text("ken99@example.com").is_none());
    next().click();
    screen.run_until_idle();
    screen.get_by_text("zed@example.com");
    assert!(next().is_disabled(), "page 3 of 3 is the last");
    previous().click();
    screen.run_until_idle();
    assert_eq!(handles.page.get_untracked(), 1);
    // An out-of-range page shows the last page without rewriting the binding.
    handles.page.set(40);
    screen.run_until_idle();
    screen.get_by_text("zed@example.com");
    assert_clean(&screen);
}

#[test]
fn data_table_hides_columns_from_the_columns_menu() {
    let (screen, handles) = mount_payments();
    assert_eq!(screen.all_by_text("success").len(), 2);
    screen.get_by_role(Role::Button, "Columns").click();
    screen.run_until_idle();
    let status = screen.get_by_role(Role::MenuItemCheckBox, "Status");
    assert_eq!(status.is_checked(), Some(true));
    status.click();
    screen.run_until_idle();
    assert_eq!(handles.visible.get_untracked(), vec![false, true, true]);
    assert_eq!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Status")
            .is_checked(),
        Some(false),
        "the menu stays open for the next toggle"
    );
    screen.press_key(Key::Escape);
    screen.run_until_idle();
    assert!(screen.query_by_text("success").is_none());
    assert!(screen.query_by_role(Role::ColumnHeader, "Status").is_none());
    screen.get_by_text("ken99@example.com");
    handles.visible.set(vec![true, true, true]);
    screen.run_until_idle();
    assert_eq!(screen.all_by_text("success").len(), 2);
    assert_clean(&screen);
}

#[test]
fn data_table_invokes_the_row_actions() {
    let (screen, handles) = mount_payments();
    screen.get_by_role(Role::Button, "Open menu 2").click();
    screen.run_until_idle();
    screen
        .get_by_role(Role::MenuItem, "Copy payment ID")
        .click();
    screen.run_until_idle();
    assert_eq!(handles.acted.get_untracked(), Some(2));
    assert_clean(&screen);
}

#[test]
fn data_table_cells_move_with_the_arrow_keys() {
    let (screen, _) = mount_payments();
    screen
        .get_by_role(Role::CheckBox, "Select ken99@example.com")
        .focus();
    screen.press_tab(false);
    screen.run_until_idle();
    screen.press_key(Key::ArrowDown);
    screen.run_until_idle();
    screen.press_key(Key::Space);
    screen.run_until_idle();
    assert!(
        screen
            .get_by_role(Role::Row, "abe45@example.com")
            .is_selected(),
        "Space selects the row of the focused cell"
    );
    assert_clean(&screen);
}

/// How many pixels of `screen` inside `rect` are near `color`.
fn count_near(screen: &Screen, rect: kivact::draw::Rect, color: u32) -> usize {
    let shot = screen.screenshot();
    let scale = screen.scale();
    let (x0, y0) = ((rect.x * scale) as u32, (rect.y * scale) as u32);
    let x1 = ((rect.x + rect.width) * scale) as u32;
    let y1 = ((rect.y + rect.height) * scale) as u32;
    let mut found = 0;
    for y in y0..y1.min(shot.height) {
        for x in x0..x1.min(shot.width) {
            if shot.pixel(x, y).is_some_and(|p| distance(p, color) <= 6) {
                found += 1;
            }
        }
    }
    found
}

const RED: u32 = 0xe11d48;
const BLUE: u32 = 0x2563eb;

fn chart_of(kind: basic::ChartKind) -> basic::Chart {
    basic::chart(config(), rows())
        .kind(kind)
        .height(220.0)
        .style(style().width(520))
        .test_id("chart")
}

fn mount_chart(
    mode: kivact::style::ThemeMode,
    chart: impl FnOnce() -> basic::Chart + 'static,
) -> Screen {
    kivact::test::render_with_handle(options(mode), move || ((), page(chart()))).0
}

#[test]
fn every_chart_kind_draws_its_series_in_their_colors_in_both_themes() {
    for mode in [
        kivact::style::ThemeMode::Light,
        kivact::style::ThemeMode::Dark,
    ] {
        for kind in [
            basic::ChartKind::Bar,
            basic::ChartKind::StackedBar,
            basic::ChartKind::Line,
            basic::ChartKind::Area,
            basic::ChartKind::Pie,
            basic::ChartKind::Donut,
            basic::ChartKind::Radar,
            basic::ChartKind::Radial,
        ] {
            let screen = mount_chart(mode, move || chart_of(kind));
            let rect = screen.get_by_test_id("chart").rect().expect("laid out");
            for (name, color) in [("red", RED), ("blue", BLUE)] {
                assert!(
                    count_near(&screen, rect, color) >= 10,
                    "{kind:?} draws the {name} series in {mode:?}"
                );
            }
            assert_clean(&screen);
            screen.assert_layout();
        }
    }
}

#[test]
fn charts_use_the_theme_chart_colors_by_default() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        basic::chart(
            basic::ChartConfig::new([
                basic::ChartSeries::new("a", "A"),
                basic::ChartSeries::new("b", "B"),
                basic::ChartSeries::new("c", "C"),
            ]),
            vec![basic::ChartRow::new("One", [100.0, 80.0, 60.0])],
        )
        .style(style().width(520))
        .test_id("chart")
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    for color in [0xd4d4d4, 0x737373, 0x525252] {
        assert!(
            count_near(&screen, rect, color) >= 50,
            "{color:06x} is a bar"
        );
    }
}

#[test]
fn a_bar_chart_puts_each_value_where_it_belongs() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Bar)
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    // The February desktop bar is the tallest (305 of 400): its red column
    // reaches higher than the April one (73).
    let top_of_red = |x0: f32, x1: f32| {
        (0..220)
            .find(|dy| {
                (0..((x1 - x0) as u32)).any(|dx| {
                    distance(pixel(&screen, x0 + dx as f32, rect.y + *dy as f32), RED) <= 6
                })
            })
            .expect("a red bar")
    };
    let width = rect.width;
    let february = top_of_red(rect.x + width * 0.3, rect.x + width * 0.5);
    let april = top_of_red(rect.x + width * 0.8, rect.x + width);
    assert!(
        february + 40 < april,
        "305 is taller than 73: {february} vs {april}"
    );
}

#[test]
fn a_pie_chart_splits_the_disc_by_series_total() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Pie)
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + 110.0);
    // Desktop is 801 of 1391 (58 %): it covers 3 o'clock, mobile covers 9 o'clock.
    assert!(distance(pixel(&screen, cx + 40.0, cy), RED) <= 6);
    assert!(distance(pixel(&screen, cx - 40.0, cy), BLUE) <= 6);
}

#[test]
fn a_donut_chart_shows_the_total_in_its_hole() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Donut)
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + 110.0);
    assert!(
        distance(pixel(&screen, cx + 30.0, cy - 35.0), 0xffffff) <= 4,
        "the hole"
    );
    let middle = kivact::draw::Rect::new(cx - 40.0, cy - 20.0, 80.0, 40.0);
    assert!(
        count_near(&screen, middle, 0x0a0a0a) > 20,
        "the total is written in the foreground color"
    );
}

#[test]
fn a_chart_is_an_image_named_and_described_by_its_data() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Line)
    });
    let image = screen.get_by_role(Role::Image, "Line chart: Desktop, Mobile");
    let description = image.description();
    assert!(
        description.contains("January: Desktop 186, Mobile 80"),
        "{description}"
    );
    assert!(description.contains("February: Desktop 305, Mobile 200"));
    screen.get_by_text("Desktop");
    screen.get_by_text("Mobile");
}

#[test]
fn the_legend_can_be_hidden() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Bar).legend(false)
    });
    assert!(screen.query_by_text("Desktop").is_none());
}

#[test]
fn hovering_a_chart_shows_the_tooltip_card_with_the_values() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Bar)
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    assert!(screen.query_by_text("February").is_none());
    // The second of four categories lies at 3/8 of the plot width.
    screen.mouse_move(rect.x + rect.width * 0.4, rect.y + 100.0);
    screen.run_until_idle();
    screen.get_by_text("February");
    screen.get_by_text("305");
    screen.get_by_text("200");
    assert_eq!(screen.all_by_text("Desktop").len(), 2, "legend and tooltip");
    // The tooltip swatches carry the series colors.
    let card = screen.get_by_text("February").rect().expect("laid out");
    let swatches = kivact::draw::Rect::new(card.x - 20.0, card.y, 200.0, 80.0);
    assert!(count_near(&screen, swatches, RED) >= 20);
    // Within the same category the card follows the pointer.
    screen.mouse_move(rect.x + rect.width * 0.4 + 6.0, rect.y + 110.0);
    screen.run_until_idle();
    let moved = screen.get_by_text("February").rect().expect("laid out");
    assert_eq!((moved.x - card.x, moved.y - card.y), (6.0, 10.0));
    // The pointer passes through the card: over it, the plot still reports
    // what lies under the pointer.
    screen.mouse_move(card.x + 4.0, card.y + 4.0);
    screen.run_until_idle();
    assert!(screen.query_by_text("February").is_some() || screen.query_by_text("March").is_some());
    screen.assert_layout();
    screen.mouse_leave();
    screen.run_until_idle();
    assert!(screen.query_by_text("February").is_none());
    assert_clean(&screen);
}

#[test]
fn hovering_pie_and_radar_charts_reports_what_is_under_the_pointer() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Pie)
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + 110.0);
    screen.mouse_move(cx - 40.0, cy);
    screen.run_until_idle();
    screen.get_by_text("590");
    drop(screen);
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Radar)
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    screen.mouse_move(rect.x + rect.width / 2.0 + 30.0, rect.y + 110.0);
    screen.run_until_idle();
    screen.get_by_text("February");
}

#[test]
fn horizontal_bars_list_categories_down_the_left() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Bar).orientation(Orientation::Horizontal)
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    assert!(count_near(&screen, rect, RED) >= 50);
    // April is the fourth band from the top.
    screen.mouse_move(rect.x + rect.width * 0.5, rect.y + 8.0 + 170.0 * 0.9);
    screen.run_until_idle();
    screen.get_by_text("April");
    assert_clean(&screen);
}

#[test]
fn a_chart_follows_a_live_theme_switch() {
    let (screen, dark) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let dark = signal(false);
            (
                dark,
                kivact::view::provide_theme(
                    move || {
                        basic::theme(if dark.get() {
                            kivact::style::ThemeMode::Dark
                        } else {
                            kivact::style::ThemeMode::Light
                        })
                    },
                    || page(chart_of(basic::ChartKind::Bar)),
                ),
            )
        });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    // The hover cursor behind the bars is the muted token: a spot of the
    // April band above its shortest bar.
    let at = (rect.x + rect.width * 0.88, rect.y + 20.0);
    screen.mouse_move(at.0, at.1);
    screen.run_until_idle();
    assert!(distance(pixel(&screen, at.0, at.1), 0xf5f5f5) <= 3);
    dark.set(true);
    screen.run_until_idle();
    assert!(
        distance(pixel(&screen, at.0, at.1), 0x262626) <= 4,
        "the cursor band took the dark muted color: {:06x}",
        pixel(&screen, at.0, at.1)
    );
    assert!(
        count_near(&screen, rect, RED) >= 50,
        "series keep their color"
    );
    assert_clean(&screen);
}

#[test]
fn the_gallery_has_no_layout_violations() {
    let (screen, _) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let (handles, table) = payments_table();
            (
                handles,
                page((
                    chart_of(basic::ChartKind::Area),
                    chart_of(basic::ChartKind::Radial),
                    table,
                )),
            )
        });
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn data_table_aligns_columns_and_shows_only_what_it_is_given() {
    let (screen, _) = mount_payments();
    let amount = screen.get_by_text("$316.00").rect().expect("laid out");
    assert!(
        amount.x + amount.width > 560.0,
        "an end-aligned amount sits at the right of its column"
    );
    drop(screen);
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            (
                (),
                page(kivact::ui::data_table(
                    payments(),
                    |p: &Payment| p.id,
                    vec![
                        kivact::ui::DataColumn::new("Email", |p: &Payment| p.email.to_string())
                            .compare(|a, b| a.email.cmp(b.email)),
                    ],
                )),
            )
        });
    // Without its state, the table has no columns menu, filter, check
    // boxes, sort buttons, summary or pager: it shows every row.
    assert!(screen.query_by_role(Role::Button, "Columns").is_none());
    assert!(screen.query_by_role(Role::TextBox, "").is_none());
    assert!(screen.query_by_role(Role::CheckBox, "Select all").is_none());
    assert!(screen.query_by_role(Role::Button, "Next").is_none());
    assert!(screen.query_by_text("0 of 7 row(s) selected.").is_none());
    assert_eq!(
        screen.get_by_role(Role::ColumnHeader, "Email").sort(),
        SortDirection::None
    );
    screen.get_by_text("zed@example.com");
    assert_clean(&screen);
}

#[test]
fn a_chart_writes_its_values_through_the_value_format() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Line).value_format(|value| format!("{value:.0}k"))
    });
    let image = screen.get_by_role(Role::Image, "Line chart: Desktop, Mobile");
    let description = image.description();
    assert!(
        description.contains("January: Desktop 186k, Mobile 80k"),
        "{description}"
    );
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    screen.mouse_move(rect.x + rect.width * 0.4, rect.y + 100.0);
    screen.run_until_idle();
    screen.get_by_text("305k");
    screen.get_by_text("200k");
    assert_clean(&screen);
}

#[test]
fn a_chart_keeps_the_callers_style() {
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            (
                (),
                page(
                    basic::chart(config(), rows())
                        .style(caller_style())
                        .test_id("styled-chart"),
                ),
            )
        });
    assert_caller_won(&screen, "styled-chart", true);
    assert_clean(&screen);
}
