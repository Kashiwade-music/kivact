//! Role: Supply Neutral disclosure appearance options to standard component recipes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChevronPosition {
    Start,
    #[default]
    End,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisclosureDensity {
    Compact,
    #[default]
    Balanced,
    Spacious,
}
// xtask-allow(state-fields): Constant, independent theme appearance options; stores no disclosure or group lifecycle state.
/// Visual options for `kivact::ui::disclosure` and `disclosure_group`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisclosureVariant {
    pub chevron: ChevronPosition,
    pub density: Option<DisclosureDensity>,
    pub dividers: bool,
}
impl kivact::ui::Variant for DisclosureVariant {}
