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
    pub indicator: StepIndicatorKind,
    // xtask-allow(state-fields): independent: whether the caller supplied an activation handler; any status can have one
    pub interactive: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the stepper's state for recipes, written only by its render function
    pub collapsed: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the stepper's state for recipes, written only by its render function
    pub disabled: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the stepper's state for recipes, written only by its render function
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
    pub mode: PaginationMode,
}

/// Actual entry semantics and input from its single activation part.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreadcrumbItemState {
    // xtask-allow(state-fields): projection: readonly snapshot of the stepper's state for recipes, written only by its render function
    pub current: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the stepper's state for recipes, written only by its render function
    pub input: Option<kivact_core::custom::PartState>,
}

/// Which steps remain mounted visibly when the responsive layout is compact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollapsedSteps {
    #[default]
    Current,
    All,
}

/// The actual indicator content policy selected on one step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepIndicatorKind {
    Auto,
    Number,
    None,
    Custom,
}

/// Structural pagination presentations, including an editable page number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaginationMode {
    #[default]
    Pages,
    Count,
    Compact,
    Dots,
    Input,
    None,
}

/// Actual inputs for a caller's pagination summary; no duplicate range endpoints are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationSummary {
    /// Zero-based actual page index.
    pub page: usize,
    pub count: usize,
    // xtask-allow(state-fields): projection: readonly snapshot of the stepper's state for recipes, written only by its render function
    pub total_items: Option<u64>,
    pub page_size: u32,
    // xtask-allow(state-fields): projection: readonly snapshot of the stepper's state for recipes, written only by its render function
    pub has_more: bool,
}

/// Construction-time enumeration of the numbered page window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaginationWindow {
    #[default]
    Adjacent,
    Buffered,
}
