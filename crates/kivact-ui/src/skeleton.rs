//! Role: Render loading placeholders and pause their clock for reduced motion.

pub(crate) mod look;
mod view;

pub use look::SkeletonShape;
pub use view::{Skeleton, skeleton};
