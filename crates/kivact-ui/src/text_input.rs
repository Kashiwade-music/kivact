//! Role: `text_input`.
//!
//! `text_input` and `text_area` put the
//! [`TextEditApi`] control on an ordinary node and
//! hand the native settings and callbacks to the `TextEditApi`; the field's
//! style is the theme's `text_input` or `text_area` recipe with a
//! [`TextInputState`]. [`TextInput`] implements `Element` with the field
//! as its node, so modifiers land on the input itself. `Style::paint` is box
//! drawing, which a native editing leaf does not support, so these recipes
//! return style only.

pub(crate) mod look;
mod view;

pub use view::{TextInput, text_area, text_input};
