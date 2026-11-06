//! Role: Expose the popup behavior and keep its transition rules private.
mod api;
mod machine;
pub use api::{PopupApi, PopupState, use_popup};
