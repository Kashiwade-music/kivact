//! Role: These headless interactions combine bindings and owner lifetimes.
//!
//! Pure policies live below the headless controllers. Mounting, focus and
//! input use Kivact's existing native tree and overlay authorities.
//!
//! These are the parts component authors build on; applications use
//! `kivact::ui` and the UI kits. A behavior draws nothing: it returns a Copy
//! handle whose parts the caller attaches with `.part(..)` to nodes or canvas
//! regions it draws itself. The official kits use only this public API.
//!
//! Naming follows react-aria and the Ark UI and Radix anatomies. A
//! constructor is `use_*` (`use_dialog`, `use_menu`, `use_list`). The floating
//! or collapsible body is `content()`, a list's row is `item(key)` and its
//! container `list()`, a table has `grid()`, `header`, `row` and `cell`, the
//! zoom and pan surface is `viewport()`, a slider has `thumb()` and
//! `track(axis)`, a number field `input()`, and a one-element behavior
//! `control()`. The open query is `shown()` and the commands are `open()`,
//! `close()` and `toggle()` on a Copy `self`. State a caller may own is a
//! builder of that name taking a `Binding` (`sort(binding)`,
//! `widths(binding)`, `anchor(binding)`), read back with `<name>_value()`.
//!
//! Opening and closing is one chain: a disclosure edits the requested open
//! state, a popup adds the overlay's focus and dismissal rules to it, and a
//! tooltip and a dialog add their own rules to a popup. A delayed shown state
//! is a [`Delayed`]. The content of a
//! popup, tooltip or dialog is mounted with the free function
//! `show(handle.shown(), factory)`, in a fresh owner on every opening. A menu
//! and a command palette keep their own `show`: a menu's top level builds its
//! content again when a request changes between the pointer and the anchor
//! origin, and a submenu mounts its level in a new owner while it is open.

#![forbid(unsafe_code)]

mod announcer;
mod axis;
mod calendar;
mod combobox;
mod content_reveal;
mod delayed;
mod dialog;
mod disclosure;
mod drag;
mod drag_settle;
mod drag_velocity;
mod edit;
mod focus_return;
mod follow_end;
mod hover_corridor;
mod list;
mod menu;
mod menubar;
mod palette;
mod policy;
mod popup;
mod press;
mod range_scalar;
mod roving;
mod scalar;
mod sealed;
mod selection;
pub use selection::{IntoSelection, Selection};
mod select;
mod space_pan;
mod splitter;
mod table;
mod tabs;
mod text_edit;
mod toaster;
mod toggle;
mod tone;
mod tooltip;
mod tree;
mod zoom_pan;

pub use announcer::{Announcer, announcer};
pub use calendar::{
    CalendarDayState, CalendarGrid, CalendarPolicy, CalendarSelection, CalendarSpan,
    use_calendar_grid,
};
pub use combobox::{Combobox, use_combobox};
pub use content_reveal::{ContentReveal, use_content_reveal};
pub use delayed::{Delayed, DelayedPhase, Delays, use_delayed};
pub use dialog::{DialogApi, DialogKind, Dismiss, use_dialog};
pub use disclosure::{Disclosure, DisclosureState, use_disclosure};
pub use drag::{Drag, DragCoordinates, DragPhase, DragStart, DragState, DragUpdate, use_drag};
pub use drag_settle::{DragAxis, nearest_snap, project_release, pull_to_snap, resist_overshoot};
pub use drag_velocity::DragVelocity;
pub use edit::EditSession;
pub use focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason, ReasonSet};
pub use follow_end::{FollowEnd, use_follow_end};
pub use hover_corridor::{use_hover_corridor, use_hover_surface};
pub use list::{
    List, ListActivation, ListBoundary, ListCommandModifier, ListGesture, ListItem, ListItemState,
    ListMissingAnchor, ListMode, ListPolicy, ListRangeMode, ListRangePolicy, ListSelection,
    use_list,
};
pub use menu::{
    DisabledMenuFocus, MenuApi, MenuCheck, MenuEntry, MenuItemKind, MenuItemState, MenuOrigin,
    MenuPress, MenuRow, SubmenuTiming, use_menu,
};
pub use menubar::{Menubar, use_menubar};
pub use palette::{Command, CommandGroup, CommandPalette, filter_commands, use_command_palette};
pub use policy::grid::GridMove;
pub use policy::scalar::{ScalarRange, ScalarRangeError};
pub use popup::{Popup, PopupState, use_popup};
pub use range_scalar::{RangeScalar, RangeThumb, use_range_scalar};
pub use scalar::{Scalar, ScalarState, TouchSliderGesture, use_scalar};
pub use select::{SelectApi, use_select};
pub use space_pan::{SpacePan, use_space_pan};
pub use splitter::{SplitterApi, SplitterState, use_splitter};
pub use table::{
    ColumnDef, FilterValue, SortOrder, Table, TableCellState, TableFocus, TableHeaderState,
    TableModel, TableRowState, use_table, use_table_model,
};
pub use tabs::{TabListState, TabState, TabsApi, use_tabs};
pub use toaster::{Toast, ToastId, ToastOptions, ToastState, Toaster, toaster};
pub use toggle::{
    ButtonApi, CheckboxApi, Toggle, ToggleState, use_button, use_checkbox, use_switch,
    use_toggle_button,
};
pub use tone::Tone;
pub use tooltip::{TooltipApi, TooltipConfig, use_tooltip, use_tooltip_when};
pub use tree::{Tree, TreeBranch, TreeItemState, TreeNode, TreeRow, flatten_tree_rows, use_tree};

pub use text_edit::{TextEdit, text_edit_policy, use_text_area, use_text_edit};

pub use zoom_pan::{ViewTransform, ZoomPan, use_zoom_pan};

mod inline_edit;
pub use inline_edit::{InlineEdit, use_inline_edit};
