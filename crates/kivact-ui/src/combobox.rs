//! Role: Expose the themed combobox family and its theme variables.
mod data;
mod handle;
mod labels;
pub(crate) mod look;
mod notices;
mod render;
pub use data::Combobox;
pub use handle::combobox;

/// Structural presentation of a combobox list: inline, always shown, or a
/// popup that the caller's `open` signal shows and hides.
#[derive(Clone, Copy)]
pub enum ComboboxMode {
    Inline,
    Popup,
}
