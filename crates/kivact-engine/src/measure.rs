//! Role: Passive leaf measurement state owned above the structural tree.

pub(crate) mod editor;
mod inherited_text;
pub(crate) mod input;
pub(crate) mod leaf;
pub(crate) mod state;

pub(crate) use inherited_text::{computed_text, input_style};
#[cfg(any(debug_assertions, feature = "testing"))]
pub(crate) use inherited_text::text_props;
