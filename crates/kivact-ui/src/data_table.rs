//! Role: A standard keyed data table composes the one interaction model and table.
//!
//! [`data_table`] filters, sorts and pages the caller's rows with
//! [`use_table_model`](kivact_interaction::use_table_model) and gives the
//! grid keys, selection and semantics of
//! [`use_table`](kivact_interaction::use_table) to the nodes it draws. The
//! rows, the selection, the sort, the filter text, the page and the column
//! widths stay the caller's: the table keeps no copy of them.
mod builder;
mod chrome;
mod grouped;
mod row_status;
mod rows;
mod runtime;
mod sticky_paint;
mod types;

pub use builder::data_table;
pub use grouped::DataTableGroupState;
pub use row_status::{DataTableRowStatusPaint, RowStatus};
pub use sticky_paint::DataTableStickyPaint;
pub use types::{DataTable, DataTableContext, DataTableDensity};
