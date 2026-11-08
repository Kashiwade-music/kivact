//! Role: The tables a script shows in the DevTools app.

use serde::{Deserialize, Serialize};

/// The parameters of `show_table`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShowTableParams {
    /// The title of the table.
    pub title: String,
    /// The column headers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
    /// The rows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<Vec<String>>,
}
