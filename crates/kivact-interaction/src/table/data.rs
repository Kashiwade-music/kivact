//! Role: Passive table inputs, the focus pattern and observable part state.

use kivact_core::primitive::PartState;
use kivact_core::view::{Reactive, SortDirection};

/// The row-erased projection of a [`ColumnDef`](crate::ColumnDef) that a
/// [`Table`](crate::Table) reads: its key, label, whether a header
/// activation sorts by it and its starting width. It is derived from the
/// current definitions and owns no order or configuration of its own.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Header<C> {
    pub(super) key: C,
    pub(super) label: Reactive<String>,
    pub(super) sortable: bool,
    /// The starting width in the caller's unit; `None` leaves it to layout.
    pub(super) width: Option<f64>,
}

/// Which elements of a [`Table`](crate::Table) take the keyboard focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TableFocus {
    /// A data grid: one cell is the tab stop and the grid keys move between
    /// cells. Use it when cells hold data a user reads or edits one by one.
    #[default]
    Cells,
    /// A list of rows: one row is the tab stop, the list keys move between
    /// rows and select them, and the pointer selects rows. Use it when a row
    /// is the unit a user picks, like files or layers.
    Rows,
}

/// The sort state and actual input state of a column header.
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TableHeaderState {
    pub input: PartState,
    pub sort: SortDirection,
    pub sortable: bool,
}

/// The selection, roving focus and actual input state of a row.
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TableRowState {
    pub input: PartState,
    pub selected: bool,
    /// The row holds the tab stop: itself with [`TableFocus::Rows`], one of
    /// its cells with [`TableFocus::Cells`].
    pub active: bool,
}

/// The row selection, roving focus and actual input state of a cell.
/// Built with [`TableCellState::new`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TableCellState {
    pub input: PartState,
    /// The cell's row is selected.
    pub selected: bool,
    /// The cell holds the tab stop; always false with [`TableFocus::Rows`].
    pub active: bool,
}

impl TableCellState {
    /// The state of a cell whose part is in `input`, whose row is `selected`
    /// or not and which holds the tab stop (`active`) or not.
    pub const fn new(input: PartState, selected: bool, active: bool) -> Self {
        TableCellState {
            input,
            selected,
            active,
        }
    }
}
