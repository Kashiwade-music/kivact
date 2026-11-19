//! Role: Render a native file drop target with validation and optional picking.
pub(crate) mod look;
pub(crate) mod tokens;
mod view;

pub use view::{Dropzone, dropzone};
