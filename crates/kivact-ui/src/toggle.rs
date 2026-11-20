//! Role: Build themed binary toggles from their actual interaction part state.
//!
//! A checkbox or a switch takes its value as the constructor's one
//! `Signal<bool>`, like a select and a text input; there is no hidden local
//! binding and no separate constant `checked` mode. Its recipe gets the real
//! checked state in a `CheckableState`, and `.size` only reaches the recipe,
//! which decides the dimensions.
pub(crate) mod look;
mod parts;
mod switch_drag;
mod view;

pub use view::{Checkbox, Switch, checkbox, switch};
