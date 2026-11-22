//! Role: Compose plain, pressable, or selectable card surfaces.
pub(crate) mod look;
pub use look::CardAction;
mod view;
pub use view::{Card, card};
