//! Role: Chart and data table tests: colors, tooltips, filtering, sorting, paging.

use crate::support::{assert_clean, distance};
use kivact::interaction::SortOrder;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::RenderOptions;
use kivact::test::Screen;
use kivact::view::SortDirection;
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
    sort: Signal<Option<(usize, kivact::interaction::SortOrder)>>,
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
                        kivact::ui::MenuEntry::checkbox(i, *title, shown[i]).keep_open(true)
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
fn data_table_sorts_with_the_header_buttons() {
    let (screen, handles) = mount_payments();
    let email = || screen.get_by_role(Role::ColumnHeader, "Email");
    assert_eq!(email().sort(), SortDirection::None);
    email().click();
    screen.run_until_idle();
    assert_eq!(
        handles.sort.get_untracked(),
        Some((1, SortOrder::Ascending))
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
        Some((1, SortOrder::Descending))
    );
    assert!(
        y_of(&screen, "zed@example.com") < y_of(&screen, "silas22@example.com"),
        "descending by email"
    );
    screen.get_by_role(Role::ColumnHeader, "Amount").click();
    screen.run_until_idle();
    assert_eq!(
        handles.sort.get_untracked(),
        Some((2, SortOrder::Ascending))
    );
    screen.get_by_text("$15.00");
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
