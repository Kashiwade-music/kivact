//! Role: The errors the app reported.

use serde::{Deserialize, Serialize};

use crate::common::SourceLocation;

/// One reported error.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ErrorEntry {
    /// The sequence number of the error.
    pub seq: u64,
    /// The kind of the error.
    pub kind: String,
    /// The message.
    pub message: String,
    /// Where the error was reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceLocation>,
    /// The turn that reported the error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn: Option<u64>,
    /// Whether the app handled the error.
    pub handled: bool,
}

/// The reported errors.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ErrorList {
    /// The errors, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorEntry>,
}

/// The parameters of `list_errors`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListErrorsParams {
    /// Only errors after this sequence number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    /// The newest number of errors to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}
