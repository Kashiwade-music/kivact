//! Role: Store one canonical node semantics value and map its relation targets.

use super::{
    A11yAction, CheckedState, CurrentItem, Finite, LiveRegion, Orientation, PopupKind, Role,
    SortDirection,
};
use crate::SemanticsText;

/// The semantics of one node, including ordinary and advanced properties.
/// `R` names a relation target: FocusTarget in core, NodeId in the engine,
/// and SemanticsId in the backend. Optional values preserve an unstated value.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Semantics<R> {
    /// Accessible role; None leaves the role unstated.
    pub role: Option<Role>,
    /// Accessible name.
    pub name: Option<String>,
    /// Additional accessible description.
    pub description: Option<String>,
    /// Accessible textual value.
    pub value_text: Option<String>,
    /// Numeric lower bound.
    pub value_min: Option<Finite>,
    /// Numeric upper bound.
    pub value_max: Option<Finite>,
    /// Current numeric value.
    pub value_now: Option<Finite>,
    /// Checked state; None for a node that is not a toggle.
    pub checked: Option<CheckedState>,
    /// Selection state; None leaves it unstated.
    pub selected: Option<bool>,
    /// Expansion state; None leaves it unstated.
    pub expanded: Option<bool>,
    /// Whether interaction is disabled.
    pub disabled: bool,
    /// Whether this node is hidden from assistive technology.
    pub hidden: bool,
    /// Whether keyboard focus is allowed.
    pub focusable: bool,
    /// Whether the accessible value is invalid; None leaves it unstated.
    pub invalid: Option<bool>,
    /// Whether text is protected and must omit its text snapshot.
    pub protected: bool,
    /// Supported actions; None leaves the action list unstated.
    pub actions: Option<Vec<A11yAction>>,
    /// Author automation identity used by screen queries.
    pub test_id: Option<String>,
    /// Live announcement policy.
    pub live_region: Option<LiveRegion>,
    /// Sort direction.
    pub sort: SortDirection,
    /// Direction of the control.
    pub orientation: Orientation,
    /// Active descendant; publication validates visibility, role and containment.
    pub active_descendant: Option<R>,
    /// Controlled nodes, in declaration order.
    pub controls: Vec<R>,
    /// Describing nodes, in declaration order.
    pub described_by: Vec<R>,
    /// Labelling nodes, in declaration order.
    pub labelled_by: Vec<R>,
    /// Total table rows, including header rows; None means unknown or unstated.
    pub row_count: Option<u32>,
    /// Total table columns, including hidden columns; None means unknown or unstated.
    pub column_count: Option<u32>,
    /// Logical table row, counted from one; zero is not published.
    pub row_index: Option<u32>,
    /// Logical table column, counted from one; zero is not published.
    pub column_index: Option<u32>,
    /// Hierarchy depth, counted from one; zero is not published.
    pub level: Option<u32>,
    /// Number of items in the set; zero denotes an empty set.
    pub set_size: Option<u32>,
    /// Position counted from one; invalid positions are not published.
    pub pos_in_set: Option<u32>,
    /// Whether more than one item may be selected; None leaves it unstated.
    pub multiselectable: Option<bool>,
    /// Whether assistive technology should wait for updates to finish.
    pub busy: Option<bool>,
    /// Whether a value is required; None leaves it unstated.
    pub required: Option<bool>,
    /// Whether the value is read-only; None leaves it unstated.
    pub read_only: Option<bool>,
    /// The kind of popup this control opens.
    pub has_popup: Option<PopupKind>,
    /// The meaning of the current item in a related set.
    pub current_item: Option<CurrentItem>,
    /// Custom or native accessible text in unscrolled content coordinates.
    pub text: Option<SemanticsText>,
}

impl<R> Default for Semantics<R> {
    fn default() -> Self {
        Self {
            role: None,
            name: None,
            description: None,
            value_text: None,
            value_min: None,
            value_max: None,
            value_now: None,
            checked: None,
            selected: None,
            expanded: None,
            disabled: false,
            hidden: false,
            focusable: false,
            invalid: None,
            protected: false,
            actions: None,
            test_id: None,
            live_region: None,
            sort: Default::default(),
            orientation: Default::default(),
            active_descendant: None,
            controls: Vec::new(),
            described_by: Vec::new(),
            labelled_by: Vec::new(),
            row_count: None,
            column_count: None,
            row_index: None,
            column_index: None,
            level: None,
            set_size: None,
            pos_in_set: None,
            multiselectable: None,
            busy: None,
            required: None,
            read_only: None,
            has_popup: None,
            current_item: None,
            text: None,
        }
    }
}

impl<R> Semantics<R> {
    /// Map relation targets, omitting targets the mapper rejects.
    /// The mapper visits active_descendant, controls, described_by and labelled_by
    /// in that order, preserving each list's order and duplicate targets.
    /// Every non-relation value, including accessible text, is moved unchanged.
    pub fn map_relations<S>(self, mut f: impl FnMut(R) -> Option<S>) -> Semantics<S> {
        Semantics {
            role: self.role,
            name: self.name,
            description: self.description,
            value_text: self.value_text,
            value_min: self.value_min,
            value_max: self.value_max,
            value_now: self.value_now,
            checked: self.checked,
            selected: self.selected,
            expanded: self.expanded,
            disabled: self.disabled,
            hidden: self.hidden,
            focusable: self.focusable,
            invalid: self.invalid,
            protected: self.protected,
            actions: self.actions,
            test_id: self.test_id,
            live_region: self.live_region,
            sort: self.sort,
            orientation: self.orientation,
            active_descendant: self.active_descendant.and_then(&mut f),
            controls: self.controls.into_iter().filter_map(&mut f).collect(),
            described_by: self.described_by.into_iter().filter_map(&mut f).collect(),
            labelled_by: self.labelled_by.into_iter().filter_map(&mut f).collect(),
            row_count: self.row_count,
            column_count: self.column_count,
            row_index: self.row_index,
            column_index: self.column_index,
            level: self.level,
            set_size: self.set_size,
            pos_in_set: self.pos_in_set,
            multiselectable: self.multiselectable,
            busy: self.busy,
            required: self.required,
            read_only: self.read_only,
            has_popup: self.has_popup,
            current_item: self.current_item,
            text: self.text,
        }
    }
}
