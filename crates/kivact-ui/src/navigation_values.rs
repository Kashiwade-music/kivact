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
    // xtask-allow(state-fields): independent: whether Stepper::on_step_press was given, the same for every step; a disabled, optional or collapsed step keeps it, and disabled alone refuses the press
    pub interactive: bool,
    // xtask-allow(state-fields): independent: copied from StepperState::collapsed, the whole stepper's responsive layout; disabled and optional steps appear in either layout
    pub collapsed: bool,
    // xtask-allow(state-fields): independent: StepperItem::disabled; a disabled step may also be optional and may have an activation handler on the stepper
    pub disabled: bool,
    // xtask-allow(state-fields): independent: StepperItem::optional only marks the label; it combines with disabled and either layout
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

/// What a breadcrumb entry is, with its link's actual input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BreadcrumbItemState {
    /// The current page's label (`aria-current="page"`); not a link.
    Current,
    /// A label the caller gave no activation.
    Label,
    /// A link and the actual input of its activation part.
    Link(kivact_core::custom::PartState),
}
impl BreadcrumbItemState {
    /// Whether this is the current page's entry.
    pub fn is_current(&self) -> bool {
        matches!(self, Self::Current)
    }
    /// The link's actual input; `None` for the current page and plain labels.
    pub fn input(&self) -> Option<kivact_core::custom::PartState> {
        match self {
            Self::Link(input) => Some(*input),
            Self::Current | Self::Label => None,
        }
    }
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
    // xtask-allow(state-fields): option: Pagination::total_items; absent when the caller does not know the total
    pub total_items: Option<u64>,
    pub page_size: u32,
    // xtask-allow(state-fields): independent: Pagination::has_more for remote paging; the caller reports more pages with or without a known total
    pub has_more: bool,
}

/// Construction-time enumeration of the numbered page window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaginationWindow {
    #[default]
    Adjacent,
    Buffered,
}
