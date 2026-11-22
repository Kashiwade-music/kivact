//! Role: Typed theme: theme variables for color, space, radius, type and control metrics,
//! and the focus ring. Standard controls take every color and size from here; the host has
//! no default chrome. This module holds the parts of a [`Theme`](super::theme::value::Theme); the
//! theme itself and `use_theme` are in
//! `kivact::style`; `provide_theme` is in `kivact::style`.

mod sealed;
pub(crate) mod value;
mod vars;

pub use vars::{FontFamily, ThemeValue};
