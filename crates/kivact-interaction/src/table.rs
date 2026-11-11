//! Role: A table behavior attaches grid, header, row, cell and resize parts.
//!
//! [`use_data_table`] takes the caller's rows in display order, their key, the
//! [`ColumnDef`]s (key, reactive label, optional starting width, and a header
//! sorts exactly when the definition has a `compare`) and a [`Selection`]. The parts are `grid`,
//! `header(col)`, `row(key)`, `cell(key, col)` and `resize(col)`. A header
//! flips ascending and descending ([`SortOrder`]) and publishes [`DataTableApi::sort_value`];
//! sharing the sort is [`DataTableApi::sort`]. The widths are
//! [`DataTableApi::widths_value`] and [`DataTableApi::widths`], and a resize part is a
//! splitter whose range and key step come from [`DataTableApi::resize_limits`].
//!
//! With [`TableFocus::Cells`] (the default) one grid cell is the tab stop and
//! [`GridMove`](crate::GridMove) moves it in two dimensions; the row
//! selection keys follow the list's rules, and the variant carries only the
//! range policy the selection modifiers need. [`TableFocus`] is the fifth
//! argument of `use_data_table`.
//! With [`TableFocus::Rows`] the rows are list items that share roving focus,
//! typeahead and selection, and `row_label` names them. The Basic, Game and
//! Pop kits use cells and Creative uses rows; a kit adds only drawing and
//! domain data. A calendar is not a fixed grid of rows and columns, so
//! [`use_calendar_grid`](crate::use_calendar_grid) applies
//! [`GridMove::from_key`](crate::GridMove::from_key) to dates itself (month
//! edges, date wrapping, PageUp/PageDown) rather than use table parts.
//! Row and column count properties are not published yet.
//!
//! [`use_data_table_model`] is the headless row model (TanStack DataTableApi's) that
//! filters, sorts and pages the rows before they reach the table; give both
//! the same sort binding. Row and key types are generic, so kits need not
//! agree on a key type.

mod cells;
mod data;
mod handle;
mod model;
mod sort;
mod state;

pub use data::{
    HeaderSort, SortOrder, TableCellState, TableFocus, TableHeaderState, TableRowState,
};
pub use handle::{DataTableApi, use_data_table};
pub use model::{ColumnDef, DataTableModelApi, FilterValue, use_data_table_model};
