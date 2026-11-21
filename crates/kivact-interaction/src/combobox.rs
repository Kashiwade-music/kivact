//! Role: Expose combobox behavior and keep its transition implementation private.
mod handle;
mod machine;
pub use handle::{ComboboxApi, ComboboxOptions, use_combobox};
