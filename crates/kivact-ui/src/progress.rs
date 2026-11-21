//! Role: Build the themed progress bar from its value and the motion it shows.
//!
//! The bar is a `Role::ProgressBar` whose track is the root and whose fill is
//! its one child. Both recipes get one [`ProgressState`]: the value, the
//! fraction shown and a repeating clock for a paint that moves. The theme
//! sets their timing with [`ProgressMotion`]. The clock only runs in an active
//! window while the bar is indeterminate or [`Progress::moving`] is true.

pub(crate) mod look;
mod view;

pub use look::ProgressMotion;
pub use look::ProgressVariant;
pub use view::{Progress, progress};
