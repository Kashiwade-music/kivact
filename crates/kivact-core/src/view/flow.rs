//! Role: Control flow over views.
//! `show`, `show_some`, `dynamic`,
//! `error_boundary`, the keyed and indexed lists `for_each`/`for_index`,
//! and `virtual_list`, re-exported by `kivact::view`. (The overlay builder
//! lives with the rest of the overlay concept in `view::overlays`.)
//!
//! The branch and row builders take no setup context: each
//! branch or row is set up in its own owner, so plain-function components
//! and the setup functions work inside it.

#![allow(
    clippy::type_complexity,
    reason = "builder and row closures are stored as boxed trait objects throughout"
)]

pub(crate) mod branch;
pub(crate) mod error_boundary;
pub(crate) mod list;
pub(crate) mod virtual_list;

pub use branch::{Show, ShowSome, dynamic, show, show_some};
pub use error_boundary::{ErrorBoundary, error_boundary};
pub use list::{for_each, for_index};
pub use virtual_list::{RowHeight, VirtualList, virtual_list};
