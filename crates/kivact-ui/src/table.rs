//! Role: Compose static tables with themed rows and cells.
pub(crate) mod look;
mod view;

pub use view::{
    Table, TableCell, TableRow, table, table_body, table_cell, table_footer, table_head,
    table_header, table_row,
};
