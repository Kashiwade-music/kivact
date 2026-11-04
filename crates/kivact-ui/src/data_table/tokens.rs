//! Role: Group sticky-column and row-status paint tokens.
use super::{DataTableRowStatusPaint, DataTableStickyPaint};
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DataTableTokens {
    pub sticky: DataTableStickyPaint,
    pub row_status: DataTableRowStatusPaint,
}
