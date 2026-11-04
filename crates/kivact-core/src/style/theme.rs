//! Role: Public theme token tables of the facade.
//!
//! The parts of a [`Theme`](super::Theme): its palette, spacing, radii and
//! typography. The theme itself and `use_theme` are in `kivact::style`;
//! `provide_theme` is in `kivact::view`.

#[doc(inline)]
pub use kivact_style::theme::{Palette, Radii, Spacing, Typography};
