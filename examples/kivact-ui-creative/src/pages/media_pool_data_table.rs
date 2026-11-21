//! Role: The page of the Creative catalog.

use super::common::col;
use super::media_pool::*;
use catalog_shell::{Example, Page};
use kivact::interaction::ListItem;
use kivact::interaction::{
    ListGesture, ListOptions, ListRangeMode, ListRangeOptions, Selection, use_list,
};
use kivact::prelude::{IntoView, signal};
use kivact::state::IntoValue;

pub(crate) fn page() -> Page {
    Page::new(
        "media-pool-data-table",
        "Media Pool & Data Table",
        "Layout",
        include_str!("media_pool_data_table.rs"),
    )
    .description("Sorting, filters, row selection and labels.")
    .example(Example::new(
        "media-pool-data-table",
        "Media Pool & Data Table",
        "",
        demo,
    ))
}

// example: media-pool-data-table
fn demo() -> impl IntoView {
    let query = signal(String::new());
    let mode = signal(Some("tiles".to_owned()));
    let selected = signal(vec![1u64]);
    let sort = signal(Some((1, kivact::interaction::SortOrder::Ascending)));
    let rows = signal(fixtures());
    let tile_size = signal(120.0);
    let widths = signal(vec![
        18.0_f64, 190.0, 86.0, 98.0, 96.0, 62.0, 130.0, 70.0, 84.0, 128.0,
    ]);
    let shown = (move || {
        rows.get()
            .into_iter()
            .filter(|r| {
                r.cells
                    .iter()
                    .any(|s| s.to_lowercase().contains(&query.get().to_lowercase()))
            })
            .collect::<Vec<_>>()
    })
    .into_value();
    let selection_control = use_list(
        move || {
            shown.with(|rows| {
                rows.iter()
                    .map(|row| {
                        ListItem::new(row.id, row.cells.first().cloned().unwrap_or_default())
                    })
                    .collect::<Vec<_>>()
            })
        },
        Selection::Multiple(selected),
        ListOptions {
            range: ListRangeOptions {
                shift: ListRangeMode::Add,
                ..ListRangeOptions::default()
            },
            ..ListOptions::default()
        },
    );
    let pick = move |id: u64, shift: bool, ctrl: bool| {
        selection_control.select(
            id,
            if shift {
                ListGesture::Extend {
                    mode: ListRangeMode::Add,
                }
            } else if ctrl {
                ListGesture::Toggle
            } else {
                ListGesture::Replace
            },
        );
    };
    col((
        media_toolbar(query, mode, selected, tile_size, shown),
        media_table(query, mode, selected, sort, rows, widths),
        media_tiles(mode, selected, tile_size, shown, pick),
    ))
}
// end example
