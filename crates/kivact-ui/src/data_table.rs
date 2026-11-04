//! Role: A standard keyed data table composes the one interaction model and table.
//!
//! [`data_table`] filters, sorts and pages the caller's rows with
//! [`use_data_table_model`](kivact_interaction::use_data_table_model) and gives the
//! grid keys, selection and semantics of
//! [`use_data_table`](kivact_interaction::use_data_table) to the nodes it draws. The
//! rows, the selection, the sort, the filter text, the page and the column
//! widths stay the caller's: the table keeps no copy of them.
mod builder;
mod chrome;
mod filter_value;
mod filters;
mod group_state;
mod grouped;
mod metadata;
mod navigation;
mod row_status;
mod rows;
mod runtime;
mod sticky_paint;
mod tree;
mod types;

pub use crate::data_table_tokens::{
    DataTableRowStatusPaint, DataTableStickyPaint, DataTableTokens,
};
pub use builder::data_table;
pub use filter_value::{FilterField, FilterUi};
pub use group_state::DataTableGroupState;
pub use row_status::RowStatus;
pub use types::{DataTable, DataTableContext, DataTableDensity, DataTableTreeIndent};
