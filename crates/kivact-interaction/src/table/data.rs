//! Role: Passive table inputs, the focus pattern and observable part state.

use crate::{ListOptions, ListRangeOptions};
use kivact_core::custom::PartState;
use kivact_core::custom::Reactive;
use kivact_core::view::SortDirection;

/// The row-erased projection of a [`ColumnDef`](crate::ColumnDef) that a
/// [`DataTableApi`](crate::DataTableApi) reads: its key, label, whether a header
/// activation sorts by it and its starting width. It is derived from the
/// current definitions and owns no order or configuration of its own.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Header<C> {
    pub(super) key: C,
    pub(super) label: Reactive<String>,
    // xtask-allow(state-fields): option: whether activating the header sorts by the column
    pub(super) sortable: bool,
    // xtask-allow(state-fields): option: the starting width in the caller's unit; absent leaves it to layout
    /// The starting width in the caller's unit; `None` leaves it to layout.
    pub(super) width: Option<f64>,
    // xtask-allow(state-fields): option: the minimum width in the caller's unit
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

/// The sort state of a column header: unsortable, sortable but not sorting,
/// or sorting (with its direction and its one-based priority in a multi-column
/// sort).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderSort {
    /// The column has no comparison, so activating it does not sort.
    Unsortable,
    /// The column can sort but the table is not sorted by it.
    Sortable,
    /// The table is sorted by this column; `rank` is one-based.
    Sorted {
        direction: SortOrder,
        rank: Option<usize>,
    },
}

/// The sort state and actual input state of a column header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableHeaderState {
    pub input: PartState,
    /// The column's sort state.
    pub sorting: HeaderSort,
}

/// The selection, roving focus and actual input state of a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableRowState {
    pub input: PartState,
    // xtask-allow(state-fields): projection: readonly snapshot of the table selection for recipes, written only by the row part
    pub selected: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the list tab stop for recipes, written only by the row part
    /// The row holds the tab stop: itself with [`TableFocus::Rows`], one of
    /// its cells with [`TableFocus::Cells`].
    pub active: bool,
}

/// The row selection, roving focus and actual input state of a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableCellState {
    pub input: PartState,
    // xtask-allow(state-fields): projection: readonly snapshot of the table selection for recipes, written only by the cell part
    /// The cell's row is selected.
    pub selected: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the list tab stop for recipes, written only by the cell part
    /// The cell holds the tab stop only with [`TableFocus::Cells`].
    pub active: bool,
}
