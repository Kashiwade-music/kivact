//! Role: Shared ids, geometry and small enums of the DevTools wire vocabulary.

use serde::{Deserialize, Serialize};

/// The id of a node.
pub type NodeId = u64;
/// The id of a window.
pub type WindowId = u64;
/// The id of an owner.
pub type OwnerId = u64;

/// A place in a source file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// The path of the file.
    pub file: String,
    /// The 1-based line.
    pub line: u32,
    /// The 1-based column.
    pub column: u32,
}

/// A position.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// The horizontal coordinate.
    pub x: f64,
    /// The vertical coordinate.
    pub y: f64,
}

/// A size.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Size {
    /// The width.
    pub width: f64,
    /// The height.
    pub height: f64,
}

/// A rectangle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// The horizontal position.
    pub x: f64,
    /// The vertical position.
    pub y: f64,
    /// The width.
    pub width: f64,
    /// The height.
    pub height: f64,
}

/// No parameters, or no result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Empty {}

/// An accessible role; mirrors `kivact_types::Role` variant by variant, in the same order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    /// List item.
    ListItem,
    /// Option.
    Option,
    /// Textbox.
    TextBox,
    /// Image.
    Image,
    /// Slider.
    Slider,
    /// Progress bar.
    ProgressBar,
    /// Alert.
    Alert,
    /// Status.
    Status,
    /// List.
    List,
    /// Dialog.
    Dialog,
    /// Alert dialog.
    AlertDialog,
    /// Menu.
    Menu,
    /// Menu item.
    MenuItem,
    /// Tab.
    Tab,
    /// Tab list.
    TabList,
    /// Scroll view.
    ScrollView,
    /// Canvas.
    Canvas,
    /// Link.
    Link,
    /// Toolbar.
    Toolbar,
    /// Tooltip.
    Tooltip,
    /// Radio group.
    RadioGroup,
    /// Radio button.
    RadioButton,
    /// Switch.
    Switch,
    /// Tab panel.
    TabPanel,
    /// Splitter.
    Splitter,
    /// Table.
    Table,
    /// Row.
    Row,
    /// Cell.
    Cell,
    /// Column header.
    ColumnHeader,
    /// Tree.
    Tree,
    /// Tree item.
    TreeItem,
    /// Meter.
    Meter,
    /// Spin button.
    SpinButton,
    /// Menu bar.
    MenuBar,
    /// Checkbox menu item.
    MenuItemCheckBox,
    /// Radio menu item.
    MenuItemRadio,
    /// Separator.
    Separator,
    /// Grid.
    Grid,
    /// Grid cell.
    GridCell,
    /// Row group.
    RowGroup,
    /// Navigation landmark.
    Navigation,
    /// Log.
    Log,
    /// Article.
    Article,
    /// Main landmark.
    Main,
    /// Region landmark.
    Region,
}

/// An accessibility action; mirrors `kivact_types::A11yAction` variant by variant, in the same order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum A11yAction {
    /// Activate.
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
    /// Set value.
    SetValue,
    /// Show menu.
    ShowMenu,
    /// Dismiss.
    Dismiss,
}

/// The kind of a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// The root of a window.
    Root,
    /// A box.
    Box,
    /// Text.
    Text,
    /// An image.
    Image,
    /// An overlay.
    Overlay,
    /// Contents.
    Contents,
}

/// An interactive state of a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractiveState {
    /// Hovered.
    Hovered,
    /// Pressed.
    Pressed,
    /// Focused.
    Focused,
    /// Focus visible.
    FocusVisible,
    /// Disabled.
    Disabled,
    /// Selected.
    Selected,
}

/// A mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    /// The primary button.
    Primary,
    /// The secondary button.
    Secondary,
    /// The middle button.
    Middle,
}

/// The mode of a window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowMode {
    /// A normal window.
    Normal,
    /// A tool window.
    Tool,
    /// A modal window.
    Modal,
}

/// The build kind of the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildKind {
    /// A debug build.
    Debug,
    /// A release build.
    Release,
}
