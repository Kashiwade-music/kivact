//! Role: Typed theme: token tables for color, spacing, radius, type and focus ring.
//! Standard controls take every color and size from here; the host has
//! no default chrome. This module holds the parts of a [`Theme`](super::theme::value::Theme); the
//! theme itself and `use_theme` are in
//! `kivact::style`; `provide_theme` is in `kivact::view`.

mod tokens;
pub(crate) mod value;

pub use tokens::{Palette, Radii, Spacing, Typography};
