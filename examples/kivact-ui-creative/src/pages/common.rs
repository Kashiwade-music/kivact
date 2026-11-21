//! Shared values and view builders for Creative catalog specimens.

use super::audio_values::{TransportState, playback};
use kivact::prelude::{IntoView, Role, column, grid, node, row, text};
use kivact::state::Signal;
use kivact::state::binding;
use kivact::state::{IntoText, IntoValue};
use kivact::style::Track;
use kivact::style::{Align, style};
use kivact::view::Node;
use kivact_ui_creative as ui;

pub(super) fn keyed_options(
    entries: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> Vec<kivact::interaction::ListItem<String>> {
    entries
        .into_iter()
        .map(|(key, label)| kivact::interaction::ListItem::new(key.to_owned(), label))
        .collect()
}

pub(super) fn separator() -> Node {
    node()
        .role(Role::Separator)
        .style(style().size(1, 20).margin_x(2))
        .style(style().background(ui::color::HAIR_STRONG))
}

pub(super) fn specs(content: impl IntoView) -> Node {
    grid(content).style(
        style()
            .grid_template_columns(Track::repeat_auto_fill(Track::minmax(290, Track::fr(1))))
            .min_width(0),
    )
}

pub(super) fn spec(title: &str, subtitle: &str, content: impl IntoView) -> Node {
    column((
        row((
            text(title.to_string()).style(style().font_weight(600).font_size(11.5)),
            text(subtitle.to_string())
                .style(style().font_size(11))
                .style(style().color(ui::color::DIM)),
        ))
        .style(style().gap(8).min_height(16)),
        content,
    ))
    .style(
        style()
            .padding_top(16)
            .padding_x(18)
            .padding_bottom(20)
            .gap(10)
            .min_width(0),
    )
    .style(
        style()
            .border_right(1, ui::color::HAIR)
            .border_bottom(1, ui::color::HAIR),
    )
}

pub(super) fn line(content: impl IntoView) -> Node {
    row(content).style(style().gap(6).align_items(Align::Center).flex_wrap(true))
}

pub(super) fn col(content: impl IntoView) -> Node {
    column(content).style(style().gap(8).min_width(0))
}

pub(super) fn note(message: impl IntoText) -> kivact::view::Node {
    text(message)
        .style(style().font_size(11.5).line_height(1.7))
        .style(style().color(ui::color::DIM))
}

pub(super) fn bind<T: kivact::state::PropValue>(
    read: impl IntoValue<T>,
    write: impl Fn(T) + 'static,
) -> Signal<T> {
    binding(read.into_value(), write)
}

/// A row of the catalog's text tables: an id, one text per column and the
/// color of its left edge.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TextRow {
    pub(super) id: u64,
    pub(super) cells: Vec<String>,
    pub(super) color: Option<kivact::style::Color>,
}

impl TextRow {
    pub(super) fn new(id: u64, cells: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            id,
            cells: cells.into_iter().map(Into::into).collect(),
            color: None,
        }
    }
    pub(super) fn color(mut self, color: kivact::style::Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// A table of `TextRow`s with a column for each `(title, width)` (a width
/// of 0 shares the room), choosing `selected` and sorted by `sort`. Every
/// column sorts by its text and matches the table's filter, ignoring case.
pub(super) fn text_table(
    columns: &[(&str, f64)],
    rows: impl IntoValue<Vec<TextRow>>,
    selected: kivact::state::Signal<Vec<u64>>,
    sort: kivact::state::Signal<Option<(usize, kivact::interaction::SortOrder)>>,
) -> kivact::ui::DataTable<TextRow, u64> {
    let columns = columns
        .iter()
        .enumerate()
        .map(|(i, (title, width))| {
            kivact::ui::DataColumn::new(*title, move |row: &TextRow| {
                row.cells.get(i).cloned().unwrap_or_default()
            })
            .sortable()
            .filter(move |row, query| {
                row.cells
                    .get(i)
                    .is_some_and(|cell| cell.to_lowercase().contains(&query.to_lowercase()))
            })
            .width(*width)
        })
        .collect();
    kivact::ui::data_table(rows, |row: &TextRow| row.id, columns)
        .selection(selected)
        .sort(sort)
        .focus(kivact::interaction::TableFocus::Rows(
            kivact::interaction::ListOptions {
                typeahead: true,
                range: kivact::interaction::ListRangeOptions {
                    missing_anchor: kivact::interaction::ListMissingAnchor::FirstSelected,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .row_color(|row: &TextRow| row.color)
        // The editors' tables are dense: no selection summary, and the
        // filter text is set by the editor's own field.
        .toolbar(|_| ())
        .footer(|_| ())
}

/// A transport state for a demo, with its playback driver running.
pub(super) fn transport() -> Signal<TransportState> {
    let state = kivact::state::signal(TransportState::default());
    playback(state);
    state
}
