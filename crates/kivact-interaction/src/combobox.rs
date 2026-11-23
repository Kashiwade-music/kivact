//! Role: Expose combobox behavior and keep its transition implementation private.
mod handle;
mod known;
mod machine;
pub use handle::{ComboboxApi, ComboboxOptions, use_combobox};
pub use known::{KnownOptions, known_options};
