//! Role: Render a themed delayed preview attached to a caller-owned trigger.

pub(crate) mod look;
pub(crate) mod tokens;
mod view;

pub use view::{FocusTrigger, HoverCard, hover_card};
