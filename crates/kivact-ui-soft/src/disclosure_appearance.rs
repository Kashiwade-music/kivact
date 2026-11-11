//! Role: Supply Neutral disclosure appearance options to standard component recipes.
/// Place the disclosure chevron before or after the title in reading order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChevronPosition {
    /// Before the title, following the current reading direction.
    Start,
    /// After the title (default).
    #[default]
    End,
}
/// Vertical padding for disclosure triggers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisclosureDensity {
    /// Use the theme's smallest spacing step.
    Compact,
    /// Use the theme's standard spacing step (default).
    #[default]
    Balanced,
    /// Use the theme's larger spacing step.
    Spacious,
}
/// Visual options for `kivact::ui::disclosure` and `disclosure_group`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisclosureVariant {
    /// Chevron placement; defaults to the trailing side.
    pub chevron: ChevronPosition,
    // xtask-allow(state-fields): option: theme-author padding choice; absent uses Balanced when dividers is set and no padding otherwise.
    /// Padding override; `None` uses Balanced with dividers and no padding otherwise.
    pub density: Option<DisclosureDensity>,
    // xtask-allow(state-fields): option: theme-author choice; false draws no separator between grouped panels.
    /// Draw a separator before each grouped panel after the first; false by default.
    pub dividers: bool,
}
impl kivact::ui::Variant for DisclosureVariant {}
