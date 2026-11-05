//! Role: Provide immutable direct-child list positions and appearance defaults.
use crate::collection_values::{ItemListMarker, ItemPosition};
use crate::variants::Variants;
use kivact_core::state::{IntoValue, Value};
#[derive(Clone)]
pub(super) struct ItemContext {
    pub position: Option<ItemPosition>,
    pub marker: Value<ItemListMarker>,
    pub start: Value<u32>,
    pub variants: Variants,
}
impl Default for ItemContext {
    fn default() -> Self {
        Self {
            position: None,
            marker: ItemListMarker::None.into_value(),
            start: 1u32.into_value(),
            variants: Variants::default(),
        }
    }
}
pub(super) fn current() -> ItemContext {
    kivact_core::state::try_use_context::<ItemContext>()
        .map(|value| value.get())
        .unwrap_or_default()
}
