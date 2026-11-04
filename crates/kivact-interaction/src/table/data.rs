//! Role: Passive table inputs, the focus pattern and observable part state.

use crate::{ListOptions, ListRangeOptions};
use kivact_core::custom::PartState;
use kivact_core::view::{Reactive, SortDirection};

/// The row-erased projection of a [`ColumnDef`](crate::ColumnDef) that a
/// [`DataTableApi`](crate::DataTableApi) reads: its key, label, whether a header
/// activation sorts by it and its starting width. It is derived from the
/// current definitions and owns no order or configuration of its own.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Header<C> {
    pub(super) key: C,
    pub(super) label: Reactive<String>,
    pub(super) sortable: bool,
    /// The starting width in the caller's unit; `None` leaves it to layout.
    pub(super) width: Option<f64>,
    pub(super) min_width: Option<f64>,
}

/// The direction rows are sorted in. The announced semantics is
/// [`SortDirection`] (`SortOrder::Ascending` is `SortDirection::Ascending`);
/// an unsorted column is `None` of an `Option<SortOrder>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortOrder {
    /// Smallest first.
    Ascending,
    /// Largest first.
    Descending,
}

impl SortOrder {
    /// The other direction.
    pub fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

impl From<SortOrder> for SortDirection {
    fn from(order: SortOrder) -> Self {
        match order {
            SortOrder::Ascending => Self::Ascending,
            SortOrder::Descending => Self::Descending,
        }
    }
}

/// Which elements of a [`DataTableApi`](crate::DataTableApi) take the keyboard focus, with
/// the policy they use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableFocus {
    /// A semantic table with passive rows and cells. Controls placed in a
    /// header or cell still use their own keyboard focus rules.
    None,
    /// A data grid: one cell is the tab stop and the grid keys move between
    /// cells. Use it when cells hold data a user reads or edits one by one.
    /// `range` reads the selection modifiers; the cell keys replace the list's
    /// navigation.
    Cells(ListRangeOptions),
    /// A list of rows: one row is the tab stop, the list keys move between
    /// rows and select them, and the pointer selects rows. Use it when a row
    /// is the unit a user picks, like files or layers. The policy's mode must
    /// be [`ListMode::Select`](crate::ListMode::Select); a different mode is a
    /// usage error.
    Rows(ListOptions),
}

impl Default for TableFocus {
    fn default() -> Self {
        Self::Cells(ListRangeOptions::default())
    }
}

/// The sort state and actual input state of a column header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableHeaderState {
    pub input: PartState,
    /// The column's sort direction; `None` while the table is not sorted by it.
    pub sort: Option<SortOrder>,
    pub sortable: bool,
    /// One-based priority in an active multi-column sort.
    pub sort_rank: Option<usize>,
}

/// The selection, roving focus and actual input state of a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableRowState {
    pub input: PartState,
    pub selected: bool,
    /// The row holds the tab stop: itself with [`TableFocus::Rows`], one of
    /// its cells with [`TableFocus::Cells`].
    pub active: bool,
}

/// The row selection, roving focus and actual input state of a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableCellState {
    pub input: PartState,
    /// The cell's row is selected.
    pub selected: bool,
    /// The cell holds the tab stop only with [`TableFocus::Cells`].
    pub active: bool,
}
