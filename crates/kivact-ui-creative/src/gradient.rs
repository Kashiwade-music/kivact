//! Role: Inline index-based gradient editor sharing core drag and edit sessions.

mod data;
mod geometry;
mod state;
mod view;

pub use data::{ColorTransform, GradientStop};
pub use view::{GradientEditor, gradient_editor};
