//! Role: Configure Soft tree density and ancestor guide appearance.
use crate::ItemDensity;
use kivact::state::{IntoValue, Value};

/// Soft appearance for `kivact::ui::tree`; navigation belongs to the shared tree.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TreeLook {
    // xtask-allow(state-fields): option: absent means the component's default
    pub(crate) density: Option<Value<ItemDensity>>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(crate) guides: Option<Value<bool>>,
}
impl TreeLook {
    /// Set vertical row padding, including reactive density changes.
    pub fn density(mut self, density: impl IntoValue<ItemDensity>) -> Self {
        self.density = Some(density.into_value());
        self
    }
    /// Show or hide ancestor continuation guides.
    pub fn guides(mut self, guides: impl IntoValue<bool>) -> Self {
        self.guides = Some(guides.into_value());
        self
    }
}
impl kivact::ui::Variant for TreeLook {}
