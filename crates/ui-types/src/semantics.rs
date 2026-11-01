//! Role: Portable semantics.
//! Trees must contain unique ids and exactly one rooted hierarchy.
//!
//! Bounds are laid out in content space: where a node would be if no
//! ancestor were scrolled. A scrolling node carries its offset (`scroll`)
//! and platforms move its descendants back by it, so a scroll changes one
//! node, not every node below it. A transformed node (CSS `transform`)
//! carries its own transform the same way: it maps the node's bounds and
//! everything below it, so changing it changes one node.

use crate::{NodeId, Point, Rect, SemanticsText, Transform};

/// One accessibility node, usable on any thread.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticsNode {
    /// Stable node identity.
    pub id: NodeId,
    /// Assistive-technology role.
    pub role: Role,
    /// Children in reading order.
    pub children: Vec<NodeId>,
    /// Accessible name.
    pub label: String,
    /// Accessible description.
    pub description: String,
    /// Current accessible value.
    pub value: Option<String>,
    /// Logical bounds in the window as laid out before the scroll offsets
    /// of its ancestors (see the module documentation).
    pub bounds: Rect,
    /// How far this node scrolls its content: its descendants are shown
    /// moved back by it. Zero for a node that does not scroll.
    pub scroll: Point,
    /// The node's own transform in content space (identity for most
    /// nodes): its bounds and its descendants (moved back by its `scroll`
    /// first) are shown mapped by it, then by its ancestors' scrolls and
    /// transforms. See [`SemanticsTree::window_bounds`].
    pub transform: Transform,
    /// Whether interaction is disabled.
    pub disabled: bool,
    /// Whether this node is hidden from assistive technology.
    pub hidden: bool,
    /// Whether keyboard focus is allowed.
    pub focusable: bool,
    /// Toggle state, with None for non-toggle nodes.
    pub checked: Option<bool>,
    /// Selection state.
    pub selected: bool,
    /// Expansion state.
    pub expanded: Option<bool>,
    /// Author automation identity.
    pub test_id: Option<String>,
    /// Supported accessibility actions.
    pub actions: Vec<A11yAction>,
    /// Rare control properties; absent for ordinary nodes.
    pub properties: Option<Box<SemanticsProperties>>,
}

/// Complete accessibility tree, usable on any thread.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticsTree {
    /// Root node identity.
    pub root: NodeId,
    /// Focused node, or None for window focus.
    pub focus: Option<NodeId>,
    /// All nodes; each id occurs exactly once.
    pub nodes: Vec<SemanticsNode>,
}

impl SemanticsTree {
    /// Where node `id` is shown in the window, as platforms report it: the
    /// axis-aligned bounds of its content-space `bounds` mapped by its own
    /// transform, then by each ancestor's scroll (moved back) and
    /// transform, nearest first. `None` when `id` is not in the tree.
    pub fn window_bounds(&self, id: NodeId) -> Option<Rect> {
        let node = self.nodes.iter().find(|n| n.id == id)?;
        self.window_rect(id, node.bounds)
    }

    /// Map a rectangle from `id`'s content space to logical window bounds,
    /// applying its transform and every ancestor's scroll and transform.
    /// Returns `None` when the node is absent.
    pub fn window_rect(&self, id: NodeId, rect: Rect) -> Option<Rect> {
        let node = self.nodes.iter().find(|n| n.id == id)?;
        let mut map = node.transform;
        let mut child = id;
        while let Some(parent) = self.nodes.iter().find(|n| n.children.contains(&child)) {
            map = map
                .then(Transform::translate(-parent.scroll.x, -parent.scroll.y))
                .then(parent.transform);
            child = parent.id;
        }
        Some(map.bounds(rect))
    }
}

/// The nodes of a semantics tree that may have changed since the previous
/// update; every other node stays as it was. A node no longer listed as a
/// child of any node leaves the tree with its subtree.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticsUpdate {
    /// Root node identity.
    pub root: NodeId,
    /// Focused node, or None for window focus.
    pub focus: Option<NodeId>,
    /// New or possibly changed nodes; each id occurs at most once.
    pub nodes: Vec<SemanticsNode>,
}

/// Accessibility role of a node or scene region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Role.
pub enum Role {
    /// Generic.
    Generic,
    /// Group.
    Group,
    /// Text.
    Text,
    /// Heading.
    Heading,
    /// Button.
    Button,
    /// Checkbox.
    CheckBox,
    /// Combobox.
    ComboBox,
    /// Listbox.
    ListBox,
    /// Listitem.
    ListItem,
    /// Option.
    Option,
    /// Textbox.
    TextBox,
    /// Image.
    Image,
    /// Slider.
    Slider,
    /// Progressbar.
    ProgressBar,
    /// Alert.
    Alert,
    /// Status.
    Status,
    /// List.
    List,
    /// Dialog.
    Dialog,
    /// A dialog that interrupts with an important message and needs a response.
    AlertDialog,
    /// Menu.
    Menu,
    /// Menuitem.
    MenuItem,
    /// Tab.
    Tab,
    /// Tablist.
    TabList,
    /// Scrollview.
    ScrollView,
    /// Canvas.
    Canvas,
    /// Link.
    Link,
    /// Toolbar.
    Toolbar,
    /// Tooltip.
    Tooltip,
    /// Radiogroup.
    RadioGroup,
    /// Radiobutton.
    RadioButton,
    /// Switch.
    Switch,
    /// Tabpanel.
    TabPanel,
    /// Splitter.
    Splitter,
    /// Table.
    Table,
    /// Row.
    Row,
    /// Cell.
    Cell,
    /// Columnheader.
    ColumnHeader,
    /// Tree.
    Tree,
    /// Treeitem.
    TreeItem,
    /// Meter role.
    Meter,
    /// SpinButton role.
    SpinButton,
    /// MenuBar role.
    MenuBar,
    /// MenuItemCheckBox role.
    MenuItemCheckBox,
    /// MenuItemRadio role.
    MenuItemRadio,
    /// Separator role.
    Separator,
    /// Grid role.
    Grid,
    /// GridCell role.
    GridCell,
    /// A group of rows in a table or grid: its header, body or footer
    /// section (ARIA `rowgroup`).
    RowGroup,
    /// A landmark of links that navigate the App or the document, such as
    /// a breadcrumb or a pagination (ARIA `navigation`).
    Navigation,
}

/// Portable actions requested by assistive technology for a semantics node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum A11yAction {
    /// Native default activation (UIA Invoke or Android ACTION_CLICK).
    /// `Toggle` and `Select` declarations also accept this default action;
    /// keyboard input arrives separately through the window's key events.
    Activate,
    /// Focus.
    Focus,
    /// Increment.
    Increment,
    /// Decrement.
    Decrement,
    /// Expand.
    Expand,
    /// Collapse.
    Collapse,
    /// Toggle.
    Toggle,
    /// Select.
    Select,
    /// Scrollintoview.
    ScrollIntoView,
    /// Setvalue.
    SetValue,
    /// Showmenu.
    ShowMenu,
    /// Dismiss.
    Dismiss,
}

/// Portable LiveRegion semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LiveRegion {
    #[default]
    /// Off.
    Off,
    /// Polite.
    Polite,
    /// Assertive.
    Assertive,
}

/// Portable SortDirection semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SortDirection {
    #[default]
    /// None.
    None,
    /// Ascending.
    Ascending,
    /// Descending.
    Descending,
    /// Other.
    Other,
}

/// Portable Orientation semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Orientation {
    #[default]
    /// Unspecified.
    Unspecified,
    /// Horizontal.
    Horizontal,
    /// Vertical.
    Vertical,
}
/// Optional semantics used by advanced controls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticsProperties {
    /// Native editing text, selection and geometry. Omit for protected text.
    pub text: Option<SemanticsText>,
    /// A partially checked control. Takes precedence over `checked` in
    /// the platform's accessible toggle state.
    pub indeterminate: bool,
    /// Numeric lower bound.
    pub value_min: Option<f64>,
    /// Numeric upper bound.
    pub value_max: Option<f64>,
    /// Current numeric value.
    pub value_now: Option<f64>,
    /// Live announcement policy.
    pub live_region: Option<LiveRegion>,
    /// Sort direction.
    pub sort: SortDirection,
    /// Direction of the control.
    pub orientation: Orientation,
    /// Active descendant identity.
    pub active_descendant: Option<NodeId>,
    /// Nodes that describe this one, in order. Each names a node of the
    /// same tree.
    pub described_by: Vec<NodeId>,
    /// Nodes this one controls, in order. Each names a node of the same
    /// tree.
    pub controls: Vec<NodeId>,
    /// Nodes that label this one, in order. Each names a node of the same
    /// tree.
    pub labelled_by: Vec<NodeId>,
    /// Depth in a hierarchy, counted from 1.
    pub level: Option<u32>,
    /// Number of items in the set this node belongs to or holds.
    pub set_size: Option<u32>,
    /// Position in its set, counted from 1.
    pub pos_in_set: Option<u32>,
    /// Whether the node allows more than one selected item.
    pub multiselectable: Option<bool>,
    /// Invalid value.
    pub invalid: bool,
    /// Read-only text.
    pub read_only: bool,
    /// Password text.
    pub protected: bool,
}
