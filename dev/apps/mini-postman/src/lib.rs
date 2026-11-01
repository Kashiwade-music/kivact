//! Role: Mini Postman app: latest-request action, durable history queue and curl.
//!
//! Mini Postman on the Kivact public API: an action for the latest request,
//! a coalescing queue for durable history and real HTTP through curl.
#![warn(redundant_imports)]

mod components;
mod form;
mod http;
mod storage;

pub use components::postman;
pub use form::FormState;
pub use http::{
    PendingResponse, Request, Response, Transport, curl, execute, parse_headers, validate,
};
pub use storage::{HistoryEntry, history_path, load as load_history, save as save_history};
