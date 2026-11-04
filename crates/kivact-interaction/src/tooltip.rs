//! Role: Expose tooltip parts while keeping the App hub transitions private.
mod api;
mod machine;
pub use crate::tooltip_policy::TooltipTiming;
pub use api::{TooltipApi, use_tooltip};
