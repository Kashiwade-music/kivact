//! Role: Configure Soft tree density and ancestor guide appearance.
use crate::ItemDensity;
use kivact::state::{IntoValue, Value};

// xtask-allow(state-fields): Independent read-only visual inputs, without tree interaction state.
/// Soft appearance for `kivact::ui::tree`; navigation belongs to the shared tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeLook {
    pub(crate) density: Value<ItemDensity>,
    pub(crate) guides: Value<bool>,
}
impl Default for TreeLook {
    fn default() -> Self {
        Self {
            density: ItemDensity::Balanced.into_value(),
            guides: true.into_value(),
        }
    }
}
impl TreeLook {
    /// Set vertical row padding, including reactive density changes.
    pub fn density(mut self, density: impl IntoValue<ItemDensity>) -> Self {
        self.density = density.into_value();
        self
    }
    /// Show or hide ancestor continuation guides.
    pub fn guides(mut self, guides: impl IntoValue<bool>) -> Self {
        self.guides = guides.into_value();
        self
    }
}
impl kivact::ui::Variant for TreeLook {}
