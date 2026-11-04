//! Role: Roles, actions and ARIA values of accessible nodes.

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
    /// A region that receives a chronological stream of updates.
    Log,
    /// A self-contained article or message.
    Article,
    /// The primary content landmark of the application.
    Main,
    /// A named section of the application.
    Region,
}

/// The meaning of the current item in a related set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CurrentItem {
    /// A current item without a more specific meaning.
    True,
    /// The current page.
    Page,
    /// The current step of a process.
    Step,
    /// The current location.
    Location,
    /// The current date.
    Date,
    /// The current time.
    Time,
}

/// The kind of popup an accessible control opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PopupKind {
    /// A menu.
    Menu,
    /// A list of options.
    ListBox,
    /// A tree of options.
    Tree,
    /// A grid.
    Grid,
    /// A dialog.
    Dialog,
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

/// The input that requested a dismissal. Unmounting an overlay does not
/// produce one; applications close it explicitly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DismissReason {
    /// A pointer button was pressed outside the overlay and its anchor.
    OutsidePress,
    /// An unmodified, unhandled Escape reached this overlay.
    Escape,
    /// Assistive technology requested the offered Dismiss action.
    Accessibility,
}

/// What a node or drawn region is asked to do: an [`A11yAction`] with the
/// data only some actions carry. Assistive technology sends all of them; a
/// `Dismiss` also comes from an outside press or an Escape (its
/// [`DismissReason`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum A11yRequest {
    /// Native default activation.
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
    /// Scroll into view.
    ScrollIntoView,
    /// Set this text (a number arrives as its text): parsing, range and
    /// validity are the component's.
    SetValue(String),
    /// Show the context menu.
    ShowMenu,
    /// Close, for this reason.
    Dismiss(DismissReason),
}

impl A11yRequest {
    /// The action this request asks for.
    pub fn action(&self) -> A11yAction {
        match self {
            Self::Activate => A11yAction::Activate,
            Self::Focus => A11yAction::Focus,
            Self::Increment => A11yAction::Increment,
            Self::Decrement => A11yAction::Decrement,
            Self::Expand => A11yAction::Expand,
            Self::Collapse => A11yAction::Collapse,
            Self::Toggle => A11yAction::Toggle,
            Self::Select => A11yAction::Select,
            Self::ScrollIntoView => A11yAction::ScrollIntoView,
            Self::SetValue(_) => A11yAction::SetValue,
            Self::ShowMenu => A11yAction::ShowMenu,
            Self::Dismiss(_) => A11yAction::Dismiss,
        }
    }

    /// The request assistive technology makes for a fieldless `action`: `None`
    /// for `SetValue` (it carries a value); `Dismiss` is
    /// `Dismiss(DismissReason::Accessibility)`.
    pub fn from_action(action: A11yAction) -> Option<A11yRequest> {
        Some(match action {
            A11yAction::Activate => Self::Activate,
            A11yAction::Focus => Self::Focus,
            A11yAction::Increment => Self::Increment,
            A11yAction::Decrement => Self::Decrement,
            A11yAction::Expand => Self::Expand,
            A11yAction::Collapse => Self::Collapse,
            A11yAction::Toggle => Self::Toggle,
            A11yAction::Select => Self::Select,
            A11yAction::ScrollIntoView => Self::ScrollIntoView,
            A11yAction::SetValue => return None,
            A11yAction::ShowMenu => Self::ShowMenu,
            A11yAction::Dismiss => Self::Dismiss(DismissReason::Accessibility),
        })
    }
}

/// Whether a check box, a switch, a toggle button or a radio item is on, as
/// ARIA's `aria-checked` (`true`, `false`, `mixed`) and Radix's `data-state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CheckedState {
    /// Off.
    #[default]
    Unchecked,
    /// On.
    Checked,
    /// Partially checked (HTML's `indeterminate`), whatever the binary value
    /// is.
    Indeterminate,
}

impl CheckedState {
    /// The state of a binary value.
    pub fn from_bool(checked: bool) -> Self {
        if checked {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }

    /// Whether it is [`Checked`](Self::Checked).
    pub fn is_checked(self) -> bool {
        self == Self::Checked
    }
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
///
/// A node keeps them in one box (`NodeSemantics::accessibility`) that only
/// the nodes using them allocate: numeric values, live regions, sorting,
/// orientation, the active descendant, the relations, list and tree
/// positions. The engine hands
/// them to Arvane's snapshot and native validation as they are, except
/// what it must not publish (see the fields).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticsProperties {
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
    /// Active descendant identity. Published only while it is shown, has a
    /// role and lies in this node's subtree or in that of a node it
    /// `controls` (this node included).
    pub active_descendant: Option<super::FocusTarget>,
    /// Nodes that describe this one (`aria-describedby`), in order.
    /// Like the other relations, a target is published only while it
    /// exists, is attached and has a role (a hidden one stays).
    pub described_by: Vec<super::FocusTarget>,
    /// Nodes this one controls (`aria-controls`), in order.
    pub controls: Vec<super::FocusTarget>,
    /// Nodes that label this one (`aria-labelledby`), in order.
    pub labelled_by: Vec<super::FocusTarget>,
    /// Depth in a hierarchy, counted from 1 (`aria-level`); `0` is not
    /// published.
    pub level: Option<u32>,
    /// Number of items in the set this node belongs to or holds
    /// (`aria-setsize`); `0` is published, an empty set.
    pub set_size: Option<u32>,
    /// Position in its set, counted from 1 (`aria-posinset`); `0`, or a
    /// position past `set_size`, is not published.
    pub pos_in_set: Option<u32>,
    /// Whether the node allows more than one selected item
    /// (`aria-multiselectable`). `None` leaves it unstated.
    pub multiselectable: Option<bool>,
    /// Whether assistive technology should wait for updates to finish.
    pub busy: Option<bool>,
    /// Whether a value is required.
    pub required: Option<bool>,
    /// Whether assistive technology should report the value as read-only.
    pub read_only: Option<bool>,
    /// The type of popup this control opens.
    pub has_popup: Option<PopupKind>,
    /// The meaning of the current item in a related set.
    pub current_item: Option<CurrentItem>,
    /// The text of a node that draws and edits its own (not a native
    /// editor), as runs for assistive technology, in the node's own
    /// coordinates. The engine moves the rectangles into the window's and
    /// gives each run a platform identity of its own (a run's `id` is its
    /// key in this node). Dropped when [`check_text`] finds it invalid.
    pub text: Option<crate::SemanticsText>,
}

/// Why `text` cannot be published as a node's text runs, if it cannot: at
/// least one run, finite non-negative rectangles, carets inside their runs
/// and a selection and caret inside the text.
pub fn check_text(text: &crate::SemanticsText) -> Result<(), &'static str> {
    let finite = |r: crate::Rect| {
        [r.x, r.y, r.width, r.height, r.x + r.width, r.y + r.height]
            .into_iter()
            .all(f32::is_finite)
            && r.width >= 0.0
            && r.height >= 0.0
    };
    if text.runs.is_empty() {
        return Err("it has no run (empty text is one empty run)");
    }
    if !finite(text.caret) {
        return Err("its caret rectangle is not finite or has a negative size");
    }
    let mut length = 0u32;
    for run in text.runs.iter() {
        let count = u32::try_from(run.value.chars().count()).map_err(|_| "a run is too long")?;
        if !finite(run.bounds) || run.carets.iter().any(|c| !finite(c.rect)) {
            return Err("a run's rectangle is not finite or has a negative size");
        }
        if run.carets.iter().any(|c| c.offset > count) {
            return Err("a caret lies past the end of its run");
        }
        length = length.checked_add(count).ok_or("the text is too long")?;
    }
    let end = text
        .start
        .checked_add(length)
        .ok_or("the text is too long")?;
    let inside = |at: u32| (text.start..=end).contains(&at);
    if !inside(text.selection.0) || !inside(text.selection.1) {
        return Err("the selection lies outside the runs");
    }
    let mut ids = std::collections::HashSet::new();
    if !text.runs.iter().all(|run| ids.insert(run.id)) {
        return Err("two runs have the same id");
    }
    Ok(())
}
