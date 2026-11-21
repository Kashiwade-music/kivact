//! Role: Display reactive person images with initials and semantic status fallback.

pub(crate) mod initials;
pub(crate) mod look;
mod view;

pub use look::AvatarShape;
pub use view::{Avatar, avatar};
