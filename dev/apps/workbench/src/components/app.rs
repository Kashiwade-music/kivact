//! Role: Root component of the workbench: records, search, timeline and record list.
//!
//! Root of the workbench: generated records, the shared `Board` context,
//! the search box, the timeline, the toolbar and the virtual record list.
//! The toolbar's buttons are the external `IconButton`, whose faces are
//! original SVG icons embedded with `include_bytes!`.

use std::rc::Rc;

use kivact::prelude::*;
use kivact::view::RowHeight;
use kivact_external_component::parts::icon_button;
use kivact_external_component::{Span, search_box, timeline};

pub use super::state::{BUCKETS, Board};

use super::detail::detail;
use super::row::record_row;
use crate::model::{Record, RecordKey};

static CLEAR_SELECTION: &[u8] = include_bytes!("../../assets/icons/clear-selection.svg");
static CLEAR_RANGE: &[u8] = include_bytes!("../../assets/icons/clear-range.svg");

/// The workbench over `records` (see `generate_records`).
pub fn workbench(records: Rc<Vec<Record>>) -> impl IntoView {
    let total = records.len();
    let records = signal(records);
    let selected = signal(None::<RecordKey>);
    let query = signal(String::new());
    let range = signal(None::<Span>);
    let per_bucket = total.div_ceil(BUCKETS).max(1);
    // Keys matching the query (the whole 100k scan runs only when the query changes).
    let matching = memo(move || {
        let needle = query.with(|q| q.trim().to_lowercase());
        records.with(|all| {
            all.iter()
                .filter(|r| r.matches(&needle))
                .map(|r| r.key)
                .collect::<Vec<RecordKey>>()
        })
    });
    let histogram = memo(move || {
        let mut h = vec![0.0f32; BUCKETS];
        matching.with(|keys| {
            for k in keys {
                h[(*k as usize / per_bucket).min(BUCKETS - 1)] += 1.0;
            }
        });
        h
    });
    let visible = memo(move || {
        let span = range.get();
        matching.with(|keys| match span {
            None => keys.clone(),
            Some(s) => keys
                .iter()
                .copied()
                .filter(|k| s.contains(*k as usize / per_bucket))
                .collect(),
        })
    });
    let count = move || {
        format!(
            "{} visible records / {total} total",
            visible.with(|v| v.len())
        )
    };
    let board = Board { records, selected };
    provide(board, move || {
        column((
            text("Incident investigation workbench")
                .role(Role::Heading)
                .style(style().font_size(18).weight(600)),
            search_box("Search records", query)
                .placeholder("service, summary or record number")
                .test_id("external-search"),
            text(count).role(Role::Status).test_id("visible-count"),
            timeline("Record timeline", histogram, range)
                .bucket_label(move |b| format!("#{}", b * per_bucket))
                .height(56.0)
                .test_id("external-timeline"),
            text(move || match range.get() {
                Some(s) => format!("Timeline range: {} buckets from {}", s.len(), s.start),
                None => "Timeline range: all".to_string(),
            })
            .test_id("timeline-position"),
            row((
                icon_button("Clear selection", CLEAR_SELECTION)
                    .disabled(move || selected.get().is_none())
                    .on_press(move || selected.set(None))
                    .test_id("clear-selection"),
                icon_button("Clear range", CLEAR_RANGE)
                    .disabled(move || range.get().is_none())
                    .on_press(move || range.set(None))
                    .test_id("clear-range"),
            ))
            .role(Role::Toolbar)
            .name("Record tools")
            .test_id("toolbar")
            .style(style().gap(4)),
            show_some(selected, detail),
            virtual_list(visible, |k: &RecordKey| *k, |key, _| record_row(key))
                .row_height(RowHeight::Fixed(36.0))
                .name("Records")
                .test_id("records")
                .style(
                    style()
                        .grow(1)
                        .min_height(200)
                        .border(1, ColorToken::Border),
                ),
        ))
        .style(
            style()
                .padding(12)
                .gap(8)
                .grow(1)
                // Without this the column's automatic minimum height is its
                // content (the 3.6M px list) and the list never scrolls.
                .min_height(0)
                .background(ColorToken::Surface)
                .color(ColorToken::Text),
        )
    })
}
