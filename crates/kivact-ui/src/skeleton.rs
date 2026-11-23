//! Role: Render loading placeholders and pause their clock for reduced motion.
pub(crate) mod look;
pub use look::SkeletonShape;
mod view;
pub use view::{Skeleton, skeleton};
