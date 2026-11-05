//! Role: Describe Soft appearances consumed by canonical chat recipes.
use kivact::state::{IntoValue, Value};
/// Spacing of a Soft conversation surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatDensity {
    /// Tight conversation spacing.
    Compact,
    /// Balanced conversation spacing, the default.
    #[default]
    Balanced,
    /// Roomy conversation spacing.
    Spacious,
}
/// Bubble background treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BubbleVariant {
    /// Filled message bubbles, the default.
    #[default]
    Filled,
    /// Unfilled message bubbles.
    Ghost,
}
/// Elevation of a composer surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComposerElevation {
    /// Composer without a shadow.
    None,
    /// Low composer shadow, the default.
    #[default]
    Low,
}
/// System message framing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemMessageVariant {
    /// Plain system message framing, the default.
    #[default]
    Default,
    /// System message with divider framing.
    Divider,
}
/// Read-only reactive visual choices for standard chat builders.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatAppearance {
    /// Read-only reactive conversation spacing.
    pub density: Value<ChatDensity>,
    /// Read-only reactive bubble treatment.
    pub bubble: Value<BubbleVariant>,
    /// Read-only reactive composer shadow.
    pub elevation: Value<ComposerElevation>,
    /// Read-only reactive system-message framing.
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
