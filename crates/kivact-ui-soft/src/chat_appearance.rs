//! Role: Describe Soft appearances consumed by canonical chat recipes.
use kivact::state::{IntoValue, Value};
/// Spacing of a Soft conversation surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatDensity {
    Compact,
    #[default]
    Balanced,
    Spacious,
}
/// Bubble background treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BubbleVariant {
    #[default]
    Filled,
    Ghost,
}
/// Elevation of a composer surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComposerElevation {
    None,
    #[default]
    Low,
}
/// System message framing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemMessageVariant {
    #[default]
    Default,
    Divider,
}
/// Read-only reactive visual choices for standard chat builders.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatAppearance {
    pub density: Value<ChatDensity>,
    pub bubble: Value<BubbleVariant>,
    pub elevation: Value<ComposerElevation>,
    pub system: Value<SystemMessageVariant>,
}
impl Default for ChatAppearance {
    fn default() -> Self {
        Self {
            density: ChatDensity::Balanced.into_value(),
            bubble: BubbleVariant::Filled.into_value(),
            elevation: ComposerElevation::Low.into_value(),
            system: SystemMessageVariant::Default.into_value(),
        }
    }
}
impl kivact::ui::Variant for ChatAppearance {}
impl kivact::ui::Variant for ChatDensity {}
impl kivact::ui::Variant for BubbleVariant {}
impl kivact::ui::Variant for ComposerElevation {}
impl kivact::ui::Variant for SystemMessageVariant {}
kivact::state::constant_value!(
    ChatDensity,
    BubbleVariant,
    ComposerElevation,
    SystemMessageVariant
);
