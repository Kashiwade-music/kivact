//! Role: Headless native text editing over the public primitive contracts.
//!
//! [`use_text_edit`] (one line) and [`use_text_area`] (several) give an
//! ordinary node native editing without a look:
//! [`TextEditApi::control`] attaches the editor to the node, and the core keeps
//! no text field type or `text_field`/`text_area` constructor of its own.
//! The standard `kivact::ui::text_input` and `text_area` are the one
//! implementation every kit uses, an ordinary node with this control whose
//! style comes from the theme's recipe. The combobox and the command palette
//! put their query on it.

mod draft;
mod handle;
mod machine;
mod policy;

pub use handle::{TextEditApi, use_text_area, use_text_edit};
pub use policy::text_edit_policy;
