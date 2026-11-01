//! Role: The string rows the consumer tests give the generic table.

use kivact::interaction::{ListMissingAnchor, ListPolicy, ListRangePolicy, TableFocus};
use kivact::prelude::*;

/// A row of the tests' string tables: an id and one text per column.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Line {
    pub(super) id: u64,
    pub(super) cells: Vec<String>,
}

pub(super) fn line(id: u64, cells: impl IntoIterator<Item = impl Into<String>>) -> Line {
    Line {
        id,
        cells: cells.into_iter().map(Into::into).collect(),
    }
}

/// A table of `Line`s titled by `headers`, one column per cell sorted by
/// its text, choosing `selected`, unsorted until a header is activated.
pub(super) fn lines(
    headers: &[&str],
    rows: impl IntoValue<Vec<Line>>,
    selected: impl Into<kivact::Binding<Vec<u64>>>,
) -> kivact::ui::DataTable<Line, u64> {
    let columns = headers
        .iter()
        .enumerate()
        .map(|(i, title)| {
            kivact::ui::DataColumn::new(*title, move |line: &Line| {
                line.cells.get(i).cloned().unwrap_or_default()
            })
            .sortable()
        })
        .collect();
    kivact::ui::data_table(rows, |line: &Line| line.id, columns)
        .selection(selected)
        .focus(TableFocus::Rows)
        .policy(ListPolicy {
            typeahead: true,
            range: ListRangePolicy {
                missing_anchor: ListMissingAnchor::FirstSelected,
                ..ListRangePolicy::default()
            },
            ..ListPolicy::default()
        })
        .toolbar(|_| ())
        .footer(|_| ())
}
