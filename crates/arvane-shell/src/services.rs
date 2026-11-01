//! Role: Defines transferable options for native file and folder dialogs.

use std::path::PathBuf;

/// Native dialog options, transferable between threads before UI-thread execution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShellFileDialog {
    /// Dialog caption.
    pub title: String,
    /// Initial directory, when provided.
    pub directory: Option<PathBuf>,
    /// Initial file name, when provided.
    pub file_name: Option<String>,
    /// Filter label and extensions without leading dots; `*` accepts all files.
    pub filters: Vec<(String, Vec<String>)>,
}
