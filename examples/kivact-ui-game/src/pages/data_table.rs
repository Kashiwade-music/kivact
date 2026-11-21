//! Role: The Data Table page of the game catalog: Sorting, searching, row selection and bulk actions over the production records.

use crate::support::common::{choice_options, segmented};
use crate::support::dialogs::overlay_layer;
use crate::support::state::{Demo, Overlay, Record};
use catalog_shell::{Example, Page};
use kivact::interaction::{Tone, use_toaster};
use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_game as g;
use std::time::Duration;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new(
        "data-table",
        "Data Table",
        "Console",
        include_str!("data_table.rs"),
    )
    .description("Sorting, searching, row selection and bulk actions over the production records")
    .example(Example::new(
        "table",
        "Records",
        "A searchable, sortable table with row selection, row actions and bulk actions.",
        demo,
    ))
}

// example: table
fn demo() -> impl IntoView {
    let scene = Demo::new();
    column((table(scene), overlay_layer(scene)))
}
// end example

/// The owner's avatar color.
fn owner_color(owner: &str) -> kivact::style::Color {
    rgb(match owner {
        "Mio Kuon" => 0x12a06e,
        "Riku Hayase" => 0xd98300,
        "Nagi Shiraishi" => 0xe0364d,
        "Kai Shinonome" => 0x0c1f36,
        _ => 0x0a62c2,
    })
}

/// Whether `text` contains the query, ignoring case.
fn contains(text: &str, query: &str) -> bool {
    text.to_lowercase().contains(&query.to_lowercase())
}

/// The columns of the table specimen. Their widths match CSS table
/// intrinsic minima and share the spare width: the title's border-box
/// minimum is 220 px and the progress wrapper adds 24 px of cell padding.
fn columns() -> Vec<kivact::ui::DataColumn<Record>> {
    vec![
        kivact::ui::DataColumn::new("ID", |r: &Record| r.id.clone())
            .width(80.)
            .cell(|r: &Record| g::identifier_cell(r.id.clone()))
            .compare(|a: &Record, b: &Record| a.id.cmp(&b.id))
            .filter(|r: &Record, q: &str| contains(&r.id, q)),
        kivact::ui::DataColumn::new("Title", |r: &Record| {
            format!("{} {}", r.title, r.description)
        })
        .width(220.)
        .cell(|r: &Record| g::detail_cell(r.title.clone(), r.description.clone()))
        .compare(|a: &Record, b: &Record| a.title.cmp(&b.title))
        .filter(|r: &Record, q: &str| contains(&r.title, q) || contains(&r.description, q)),
        kivact::ui::DataColumn::new("Owner", |r: &Record| r.owner.clone())
            .width(124.)
            .cell(|r: &Record| {
                g::user_cell(
                    r.owner.clone(),
                    r.owner.chars().take(1).collect::<String>(),
                    owner_color(&r.owner),
                )
            })
            .compare(|a: &Record, b: &Record| a.owner.cmp(&b.owner))
            .filter(|r: &Record, q: &str| contains(&r.owner, q)),
        kivact::ui::DataColumn::new("Status", |r: &Record| r.status.clone())
            .width(118.)
            .cell(|r: &Record| g::badge_cell(r.status.clone(), r.tone))
            .compare(|a: &Record, b: &Record| a.status.cmp(&b.status))
            .filter(|r: &Record, q: &str| contains(&r.status, q)),
        kivact::ui::DataColumn::new("Priority", |r: &Record| format!("P{}", r.priority))
            .width(70.)
            .cell(|r: &Record| {
                g::badge_cell(
                    format!("P{}", r.priority),
                    if r.priority == 3 {
                        kivact::interaction::Tone::Danger
                    } else {
                        kivact::interaction::Tone::Accent
                    },
                )
            })
            .compare(|a: &Record, b: &Record| a.priority.cmp(&b.priority)),
        kivact::ui::DataColumn::new("Progress", |r: &Record| format!("{:.0}%", r.progress))
            .width(144.)
            .cell(|r: &Record| {
                g::progress_cell(
                    r.progress,
                    if r.tone == kivact::interaction::Tone::Danger {
                        kivact::interaction::Tone::Danger
                    } else if r.progress == 100. {
                        kivact::interaction::Tone::Success
                    } else {
                        kivact::interaction::Tone::Info
                    },
                )
            })
            .compare(|a: &Record, b: &Record| a.progress.total_cmp(&b.progress)),
        kivact::ui::DataColumn::new("Update", |r: &Record| r.updated.clone())
            .width(110.)
            .compare(|a: &Record, b: &Record| a.updated.cmp(&b.updated)),
    ]
}

pub(crate) fn table(scene: Demo) -> impl kivact::view::Element {
    let query = signal(String::new());
    let filter = signal(Some("all".to_owned()));
    let page_size = signal(Some(25_usize));
    let rows = move || {
        let wanted = filter.get().unwrap_or_else(|| "all".to_owned());
        scene
            .rows
            .get()
            .into_iter()
            .filter(|r| wanted == "all" || r.status == wanted)
            .collect::<Vec<_>>()
    };
    g::hud(column((
        table_header(scene),
        table_filters(query, filter, scene),
        kivact::ui::data_table(rows, |r: &Record| r.id.clone(), columns())
            .selection(scene.selected)
            .style(style().border(0, color::BORDER))
            .sort(signal(Some((
                6,
                kivact::interaction::SortOrder::Descending,
            ))))
            .filter(query)
            .page(signal(0_usize), move || page_size.get().unwrap_or(25))
            .row_label(|r: &Record| r.id.clone())
            .footer(move |context| table_footer(context, page_size))
            .row_actions(42., move |record| {
                g::tool_button(move || format!("Details of {}", record.get().id))
                    .style(
                        style()
                            .opacity(0.)
                            .group_hover(style().opacity(1.))
                            .focus_visible(style().opacity(1.)),
                    )
                    .on_press(move || scene.show(Overlay::Drawer))
            })
            .toolbar(move |context| table_toolbar(context, scene)),
    )))
    .test_id("TBL-01")
}

fn table_footer(
    context: kivact::ui::DataTableContext<Record, String>,
    page_size: kivact::state::Signal<Option<usize>>,
) -> Node {
    let shown = context.model.keys();
    let total = context.model.row_count();
    let selected = context
        .selection
        .map(|selection| kivact::state::IntoValue::into_value(selection));
    row((
        g::mono(kivact::i18n::message_with(
            "game.table.shown",
            "{shown} shown of {total}",
            move || {
                [
                    ("shown", shown.with(Vec::len).to_string()),
                    ("total", total.get().to_string()),
                ]
            },
        ))
        .style(style().flex_grow(1)),
        g::mono(kivact::i18n::message_with(
            "game.table.selected",
            "{count} selected",
            move || {
                [(
                    "count",
                    selected.map_or(0, |s| s.with(Vec::len)).to_string(),
                )]
            },
        )),
        row((
            text("Rows shown").style(style().font_size(11.5).color(g::color::TEXT_3)),
            kivact::ui::select(
                vec![
                    kivact::interaction::ListItem::new(25_usize, "25"),
                    kivact::interaction::ListItem::new(50, "50"),
                    kivact::interaction::ListItem::new(100, "100"),
                ],
                page_size,
            )
            .placeholder("")
            .name("Rows shown")
            .test_id("game-table-page-size")
            .style(style().width(70).height(28)),
        ))
        .style(style().gap(10).align_items(Align::Center)),
    ))
    .style(
        style()
            .padding_x(12)
            .gap(16)
            .height(48)
            .align_items(Align::Center),
    )
}

fn table_toolbar(context: kivact::ui::DataTableContext<Record, String>, scene: Demo) -> Node {
    let selection = context.selection.expect("the table has a selection");
    let selected = kivact::state::IntoValue::into_value(selection);
    row((
        g::mono(kivact::i18n::message_with(
            "game.table.selected",
            "{count} selected",
            move || [("count", selected.with(Vec::len).to_string())],
        ))
        .style(style().flex_grow(1).font_size(18).color(g::color::CYAN)),
        table_bulk_actions(selected, scene),
        kivact::ui::button(kivact::i18n::message("game.table.clear", "Clear selection"))
            .variant(kivact::ui::Slant)
            .size(kivact::ui::ControlSize::Small)
            .variant(kivact::ui::ButtonVariant::Ghost)
            .style(style().color(color::ACCENT_TEXT))
            .on_press(move || selection.set(Vec::new())),
    ))
    .hidden(move || selected.with(Vec::is_empty))
    .role(kivact::view::Role::Group)
    .name(kivact::i18n::message("game.table.bulk", "Selected records"))
    .live_region(kivact::view::LiveRegion::Polite)
    .style(
        style()
            .padding_x(12)
            .padding_y(8)
            .align_items(Align::Center),
    )
    .style(move || style().background(g::color::NAVY).color(color::ACCENT_TEXT))
}

fn table_bulk_actions(selected: kivact::state::Value<Vec<String>>, scene: Demo) -> Node {
    row((
        kivact::ui::button("Approve all")
            .variant(kivact::ui::Slant)
            .test_id("game-bulk-approve")
            .icon(kivact::view::icon("check"))
            .size(kivact::ui::ControlSize::Small)
            .variant(kivact::ui::ButtonVariant::Ghost)
            .style(style().color(color::ACCENT_TEXT))
            .on_press(move || {
                let selected = selected.get();
                scene.rows.update(|rows| {
                    for row in rows {
                        if selected.contains(&row.id) {
                            row.approve();
                        }
                    }
                });
                use_toaster().push(
                    Toast::new("Processing is complete")
                        .body(format!("Approved {} items.", selected.len()))
                        .tone(Tone::Success)
                        .duration(Some(Duration::from_secs(4))),
                );
            }),
        kivact::ui::menu("Change owner", vec![MenuEntry::item("soh", "Sou Mashiba")])
            .on_activate(move |_: &'static str| {
                scene.rows.update(|rows| {
                    for row in rows {
                        if selected.with_untracked(|ids| ids.contains(&row.id)) {
                            row.owner = "Sou Mashiba".into();
                        }
                    }
                })
            })
            .offset(0.)
            .attach(
                kivact::ui::button("Change owner")
                    .variant(kivact::ui::Slant)
                    .variant(kivact::ui::ButtonVariant::Secondary),
            ),
        kivact::ui::button("Delete")
            .variant(kivact::ui::Slant)
            .icon(kivact::view::icon("trash"))
            .size(kivact::ui::ControlSize::Small)
            .variant(kivact::ui::ButtonVariant::Danger)
            .on_press(move || scene.show(Overlay::Delete)),
    ))
    .style(style().gap(10).align_items(Align::Center))
}

fn table_header(scene: Demo) -> Node {
    row((
        g::mono("TBL-01").style(style().color(color::ACCENT)),
        text("Operations table").style(style().font_weight(700)),
        spacer(),
        g::mono(move || format!("{} selected", scene.selected.with(Vec::len)))
            .style(style().font_size(10.5).color(g::color::TEXT_3)),
    ))
    .style(
        style()
            .height(36)
            .padding_x(14)
            .gap(10)
            .align_items(Align::Center)
            .background(color::SURFACE_ALT)
            .border_bottom(1, color::BORDER),
    )
}

fn table_filters(
    query: kivact::state::Signal<String>,
    filter: kivact::state::Signal<Option<String>>,
    scene: Demo,
) -> Node {
    row((
        kivact::ui::input_group(
            "",
            (kivact::ui::text_input(query)
                .label("Search operations")
                .test_id("game-table-search")
                .placeholder("Filter by ID, title or owner")
                .style(style().width(pct(100)).padding_left(34).padding_right(52)),),
        )
        .variant(g::InputGroupVariant::Adorned)
        .start(kivact::view::icon("search").style(style().size(15, 15)))
        .end(())
        .style(style().width(260)),
        segmented(
            "Status",
            choice_options(&[
                ("all", "All"),
                ("RUNNING", "In progress"),
                ("PENDING", "On hold"),
                ("FAILED", "Failed"),
            ]),
            filter,
        ),
        spacer(),
        kivact::ui::button("CSV")
            .variant(kivact::ui::Slant)
            .icon(kivact::view::icon("download"))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .size(kivact::ui::ControlSize::Small)
            .on_press(move || {
                use_toaster().push(
                    Toast::new("CSV export")
                        .body("Exports the shown operations to CSV.")
                        .tone(Tone::Info)
                        .duration(Some(Duration::from_secs(4))),
                );
            }),
        kivact::ui::button("New operation")
            .variant(kivact::ui::Slant)
            .icon(kivact::view::icon("plus"))
            .size(kivact::ui::ControlSize::Small)
            .on_press(move || scene.show(Overlay::Create)),
    ))
    .style(
        style()
            .padding_y(10)
            .padding_x(12)
            .gap(8)
            .flex_wrap(true)
            .align_items(Align::Center)
            .border_bottom(1, color::BORDER),
    )
}
