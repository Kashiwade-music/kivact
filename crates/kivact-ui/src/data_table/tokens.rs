//! Role: Group sticky-column and row-status paint tokens.
use super::row_status_tokens::DataTableRowStatusPaint;
use super::sticky_paint_tokens::DataTableStickyPaint;
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DataTableTokens {
    pub sticky: DataTableStickyPaint,
    pub row_status: DataTableRowStatusPaint,
}
