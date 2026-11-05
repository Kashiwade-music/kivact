//! Role: Define passive semantic status and responsive step recipe projections.
use kivact_core::view::Orientation;

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
    pub orientation: Orientation,
    pub collapsed: bool,
}
