//! Role: Render the App's toasts in themed live regions.

pub(crate) mod look;
pub(crate) mod swipe;
mod view;

pub use view::{ToastRegion, toast_region};
