//! Role: Space held on a focused surface, the modifier of a Space-plus-primary pan.
//!
//! [`use_space_pan`] tracks whether Space is held while a surface has the
//! focus. [`ZoomPanApi`](crate::ZoomPanApi) uses it for its Space-plus-primary pan;
//! an editor with its own pan attaches [`SpacePanApi::control`] to its root and
//! reads [`SpacePanApi::armed`].

mod handle;
mod machine;
pub use handle::{SpacePanApi, use_space_pan};
