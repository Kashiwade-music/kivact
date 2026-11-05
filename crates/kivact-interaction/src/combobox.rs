//! Role: Expose combobox behavior and keep its transition implementation private.
mod handle;
mod machine;
pub use handle::{ComboboxApi, ComboboxMode, use_combobox};
