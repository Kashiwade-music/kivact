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
    pub count: usize,
    pub orientation: Orientation,
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
/// Actual link input and independent description-slot availability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavigationLinkState {
    pub input: kivact_core::custom::PartState,
    pub has_description: bool,
}

/// Whether this actual navigation trigger's existing panel is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationTriggerState {
    pub open: bool,
}

/// A cell in a pagination bar; page indices are zero based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PageItem {
    First,
    Previous,
    Page(usize),
    Ellipsis,
    Next,
    Last,
}

/// Actual page cell and selection supplied to the cell recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationItemState {
    pub current: bool,
    pub item: PageItem,
}

// xtask-allow(state-fields): Read-only independent current-page semantics and the actual entry part input; no stored interaction or reconciliation flags.
/// Actual entry semantics and input from its single activation part.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreadcrumbItemState {
    pub current: bool,
    pub input: Option<kivact_core::custom::PartState>,
}
