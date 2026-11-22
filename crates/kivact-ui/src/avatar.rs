//! Role: Display reactive person images with initials and semantic status fallback.
pub(crate) mod initials;
pub(crate) mod look;
pub use look::AvatarShape;
mod view;
pub use view::{Avatar, avatar};
