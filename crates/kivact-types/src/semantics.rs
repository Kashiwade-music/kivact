//! Role: Roles, actions and ARIA values of accessible nodes.

/// Accessibility role of a node or scene region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Role.
#[non_exhaustive]
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
#[non_exhaustive]
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
