//! Role: Passive actual input facts for navigation link recipes.
/// Actual link input and independent description-slot availability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavigationLinkState {
    pub input: kivact_core::custom::PartState,
    pub has_description: bool,
}

/// Semantic status attached to a step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    #[default]
    Default,
    Accent,
    Success,
    Warning,
    Error,
}
/// Recipe state for an individual step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperItemState {
    pub index: usize,
    pub count: usize,
    pub orientation: kivact_core::view::Orientation,
    pub current: usize,
    pub status: StepStatus,
    // xtask-allow(state-fields): Derived width fit shared by the root and all recipe parts.
    pub collapsed: bool,
    // xtask-allow(state-fields): Independent caller availability projected into a passive recipe snapshot.
    pub disabled: bool,
    // xtask-allow(state-fields): Independent optional-step metadata projected into a passive recipe snapshot.
    pub optional: bool,
}
/// Actual responsive layout projected from the mounted root and caller data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperState {
    pub current: usize,
    pub count: usize,
    pub orientation: kivact_core::view::Orientation,
    pub collapsed: bool,
}

// xtask-allow(state-fields): Read-only independent current-page semantics and the actual entry part input; no stored interaction or reconciliation flags.
/// Actual entry semantics and input from its single activation part.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreadcrumbItemState {
    pub current: bool,
    pub input: Option<kivact_core::custom::PartState>,
}

/// Which steps remain mounted visibly when the responsive layout is compact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollapsedSteps {
    #[default]
    Current,
    All,
}
