//! Role: Render panes sized by fractions with a themed splitter handle between each pair.
//!
//! The panes share the room the handles leave, by the fractions in the
//! caller's `Binding<Vec<f64>>`, which only the handles write. A handle is an
//! [`use_splitter`] whose value is the boundary after its pane, in percent of
//! the shared room; the pixels the panes really take are read back from their
//! rects, so a handle of any thickness (the theme's splitter recipe) moves the
//! boundary exactly as far as the pointer.
pub(crate) mod look;
mod view;

pub use view::{Split, split};
