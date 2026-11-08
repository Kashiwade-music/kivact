//! Role: Convert Kivact and engine values into the DevTools wire vocabulary.

use std::panic::Location;

use kivact_devtools_protocol as p;
use kivact_engine::contract::{KeyChord, NodeType};
use kivact_runtime::Stage;
use kivact_style::internal::ForcedStates;
use kivact_types::{A11yAction, A11yRequest, Key, Modifiers, MouseButton};

use kivact_types::{Rect, WindowId};

/// The protocol role of a Kivact role.
pub(crate) fn role(role: kivact_types::Role) -> p::Role {
    use kivact_types::Role as R;
    match role {
        R::Generic => p::Role::Generic,
        R::Group => p::Role::Group,
        R::Text => p::Role::Text,
        R::Heading => p::Role::Heading,
        R::Button => p::Role::Button,
        R::CheckBox => p::Role::CheckBox,
        R::ComboBox => p::Role::ComboBox,
        R::ListBox => p::Role::ListBox,
        R::ListItem => p::Role::ListItem,
        R::Option => p::Role::Option,
        R::TextBox => p::Role::TextBox,
        R::Image => p::Role::Image,
        R::Slider => p::Role::Slider,
        R::ProgressBar => p::Role::ProgressBar,
        R::Alert => p::Role::Alert,
        R::Status => p::Role::Status,
        R::List => p::Role::List,
        R::Dialog => p::Role::Dialog,
        R::AlertDialog => p::Role::AlertDialog,
        R::Menu => p::Role::Menu,
        R::MenuItem => p::Role::MenuItem,
        R::Tab => p::Role::Tab,
        R::TabList => p::Role::TabList,
        R::ScrollView => p::Role::ScrollView,
        R::Canvas => p::Role::Canvas,
        R::Link => p::Role::Link,
        R::Toolbar => p::Role::Toolbar,
        R::Tooltip => p::Role::Tooltip,
        R::RadioGroup => p::Role::RadioGroup,
        R::RadioButton => p::Role::RadioButton,
        R::Switch => p::Role::Switch,
        R::TabPanel => p::Role::TabPanel,
        R::Splitter => p::Role::Splitter,
        R::Table => p::Role::Table,
        R::Row => p::Role::Row,
        R::Cell => p::Role::Cell,
        R::ColumnHeader => p::Role::ColumnHeader,
        R::Tree => p::Role::Tree,
        R::TreeItem => p::Role::TreeItem,
        R::Meter => p::Role::Meter,
        R::SpinButton => p::Role::SpinButton,
        R::MenuBar => p::Role::MenuBar,
        R::MenuItemCheckBox => p::Role::MenuItemCheckBox,
        R::MenuItemRadio => p::Role::MenuItemRadio,
        R::Separator => p::Role::Separator,
        R::Grid => p::Role::Grid,
        R::GridCell => p::Role::GridCell,
        R::RowGroup => p::Role::RowGroup,
        R::Navigation => p::Role::Navigation,
        R::Log => p::Role::Log,
        R::Article => p::Role::Article,
        R::Main => p::Role::Main,
        R::Region => p::Role::Region,
    }
}

/// The Kivact role of a protocol role.
pub(crate) fn role_from(role: p::Role) -> kivact_types::Role {
    use kivact_types::Role as R;
    match role {
        p::Role::Generic => R::Generic,
        p::Role::Group => R::Group,
        p::Role::Text => R::Text,
        p::Role::Heading => R::Heading,
        p::Role::Button => R::Button,
        p::Role::CheckBox => R::CheckBox,
        p::Role::ComboBox => R::ComboBox,
        p::Role::ListBox => R::ListBox,
        p::Role::ListItem => R::ListItem,
        p::Role::Option => R::Option,
        p::Role::TextBox => R::TextBox,
        p::Role::Image => R::Image,
        p::Role::Slider => R::Slider,
        p::Role::ProgressBar => R::ProgressBar,
        p::Role::Alert => R::Alert,
        p::Role::Status => R::Status,
        p::Role::List => R::List,
        p::Role::Dialog => R::Dialog,
        p::Role::AlertDialog => R::AlertDialog,
        p::Role::Menu => R::Menu,
        p::Role::MenuItem => R::MenuItem,
        p::Role::Tab => R::Tab,
        p::Role::TabList => R::TabList,
        p::Role::ScrollView => R::ScrollView,
        p::Role::Canvas => R::Canvas,
        p::Role::Link => R::Link,
        p::Role::Toolbar => R::Toolbar,
        p::Role::Tooltip => R::Tooltip,
        p::Role::RadioGroup => R::RadioGroup,
        p::Role::RadioButton => R::RadioButton,
        p::Role::Switch => R::Switch,
        p::Role::TabPanel => R::TabPanel,
        p::Role::Splitter => R::Splitter,
        p::Role::Table => R::Table,
        p::Role::Row => R::Row,
        p::Role::Cell => R::Cell,
        p::Role::ColumnHeader => R::ColumnHeader,
        p::Role::Tree => R::Tree,
        p::Role::TreeItem => R::TreeItem,
        p::Role::Meter => R::Meter,
        p::Role::SpinButton => R::SpinButton,
        p::Role::MenuBar => R::MenuBar,
        p::Role::MenuItemCheckBox => R::MenuItemCheckBox,
        p::Role::MenuItemRadio => R::MenuItemRadio,
        p::Role::Separator => R::Separator,
        p::Role::Grid => R::Grid,
        p::Role::GridCell => R::GridCell,
        p::Role::RowGroup => R::RowGroup,
        p::Role::Navigation => R::Navigation,
        p::Role::Log => R::Log,
        p::Role::Article => R::Article,
        p::Role::Main => R::Main,
        p::Role::Region => R::Region,
    }
}

/// The wire location of a caller location.
pub(crate) fn location(at: &'static Location<'static>) -> p::SourceLocation {
    p::SourceLocation {
        file: at.file().to_owned(),
        line: at.line(),
        column: at.column(),
    }
}

/// The wire rectangle of a Kivact rectangle.
pub(crate) fn rect(rect: Rect) -> p::Rect {
    p::Rect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

/// The wire kind of a host node type.
#[allow(
    dead_code,
    reason = "the DevTools tree conversion (devtools::tree) maps engine kinds inline; this shared conversion has no caller among T12.1's files"
)]
pub(crate) fn node_kind(kind: NodeType) -> p::NodeKind {
    match kind {
        NodeType::Root => p::NodeKind::Root,
        NodeType::Box => p::NodeKind::Box,
        NodeType::Text => p::NodeKind::Text,
        NodeType::Image => p::NodeKind::Image,
        NodeType::Overlay => p::NodeKind::Overlay,
    }
}

/// The number of a window id.
pub(crate) fn window(window: WindowId) -> u64 {
    window.get()
}

/// The window id with number `raw`, if there is one.
pub(crate) fn window_from(raw: u64) -> Option<WindowId> {
    WindowId::new(raw)
}

/// The Kivact mouse button of a protocol button.
pub(crate) fn mouse_button(button: p::MouseButton) -> MouseButton {
    match button {
        p::MouseButton::Primary => MouseButton::Primary,
        p::MouseButton::Secondary => MouseButton::Secondary,
        p::MouseButton::Middle => MouseButton::Middle,
    }
}

/// The Kivact action of a protocol action.
pub(crate) fn a11y_action(action: p::A11yAction) -> A11yAction {
    match action {
        p::A11yAction::Activate => A11yAction::Activate,
        p::A11yAction::Focus => A11yAction::Focus,
        p::A11yAction::Increment => A11yAction::Increment,
        p::A11yAction::Decrement => A11yAction::Decrement,
        p::A11yAction::Expand => A11yAction::Expand,
        p::A11yAction::Collapse => A11yAction::Collapse,
        p::A11yAction::Toggle => A11yAction::Toggle,
        p::A11yAction::Select => A11yAction::Select,
        p::A11yAction::ScrollIntoView => A11yAction::ScrollIntoView,
        p::A11yAction::SetValue => A11yAction::SetValue,
        p::A11yAction::ShowMenu => A11yAction::ShowMenu,
        p::A11yAction::Dismiss => A11yAction::Dismiss,
    }
}

/// The Kivact accessibility request of a protocol action and `SetValue` value.
pub(crate) fn a11y_request(
    action: p::A11yAction,
    value: Option<String>,
) -> Result<A11yRequest, p::RpcError> {
    if action == p::A11yAction::SetValue {
        return value
            .map(A11yRequest::SetValue)
            .ok_or_else(|| p::RpcError::new(p::code::INVALID_PARAMS, "set_value needs a value"));
    }
    A11yRequest::from_action(a11y_action(action))
        .ok_or_else(|| p::RpcError::new(p::code::INVALID_PARAMS, "unknown accessibility action"))
}

/// The forced states of a protocol interactive state.
pub(crate) fn forced_state(state: p::InteractiveState) -> ForcedStates {
    match state {
        p::InteractiveState::Hovered => ForcedStates::HOVER,
        p::InteractiveState::Pressed => ForcedStates::PRESSED,
        p::InteractiveState::Focused => ForcedStates::FOCUS,
        p::InteractiveState::FocusVisible => ForcedStates::FOCUS_VISIBLE,
        p::InteractiveState::Disabled => ForcedStates::DISABLED,
        p::InteractiveState::Selected => ForcedStates::SELECTED,
    }
}

/// The protocol stage of a runtime stage.
pub(crate) fn turn_stage(stage: Stage) -> p::TurnStage {
    match stage {
        // Setup runs no turn; it names the first stage for the exhaustive match.
        Stage::Setup | Stage::Begin => p::TurnStage::Begin,
        Stage::Reports => p::TurnStage::Reports,
        Stage::Dispatch => p::TurnStage::Dispatch,
        Stage::Async => p::TurnStage::Async,
        Stage::Commit => p::TurnStage::Commit,
        Stage::Mounts => p::TurnStage::Mounts,
        Stage::UserEffects => p::TurnStage::UserEffects,
        Stage::Output => p::TurnStage::Output,
    }
}

/// The key named by the design 4.2.7 rules, if it names one.
pub(crate) fn parse_key(text: &str) -> Option<Key> {
    if text.chars().count() == 1 {
        return Some(Key::Char(text.chars().next()?));
    }
    if let Some(number) = text.strip_prefix('F')
        && let Ok(number) = number.parse::<u8>()
        && (1..=24).contains(&number)
    {
        return Some(Key::F(number));
    }
    Some(match text {
        "Enter" => Key::Enter,
        "Tab" => Key::Tab,
        "Space" => Key::Space,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Escape" => Key::Escape,
        "ArrowUp" => Key::ArrowUp,
        "ArrowDown" => Key::ArrowDown,
        "ArrowLeft" => Key::ArrowLeft,
        "ArrowRight" => Key::ArrowRight,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "ContextMenu" => Key::ContextMenu,
        "Unidentified" => Key::Unidentified,
        _ => return None,
    })
}

/// The chord named by the design 4.2.7 rules, if it names one.
pub(crate) fn parse_chord(text: &str) -> Option<KeyChord> {
    let mut modifiers = Modifiers::default();
    let mut parts = text.split('+');
    let key = parts.next_back()?;
    for part in parts {
        match part {
            "Ctrl" => modifiers.ctrl = true,
            "Shift" => modifiers.shift = true,
            "Alt" => modifiers.alt = true,
            "Meta" => modifiers.meta = true,
            _ => return None,
        }
    }
    let mut key = parse_key(key)?;
    if let Key::Char(c) = key {
        key = Key::Char(c.to_ascii_lowercase());
    }
    Some(KeyChord { key, modifiers })
}

/// Whether a file is app code rather than Kivact or Arvane (design 4.2.4).
pub(crate) fn app_code(file: &str) -> bool {
    !file
        .split(['/', '\\'])
        .any(|part| part == "kivact" || part.starts_with("kivact-") || part.starts_with("arvane"))
}
