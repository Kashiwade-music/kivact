//! Role: Build the themed toggle_button button from its actual interaction part state.
//!
//! A toggle_button takes its value as the constructor's one `Signal<bool>`, like a
//! checkbox. Its recipe gets the real pressed state in a [`ToggleButtonState`];
//! the `.icon` view is a child of the root, and the recipe lays the row out.
//! The recipe is the root's base style, so the caller's `.style(..)` wins
//! whatever the order of the calls.
pub(crate) mod look;
mod view;
pub use view::{ToggleButton, toggle_button};
