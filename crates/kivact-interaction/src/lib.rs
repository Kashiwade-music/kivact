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
//! `control()`. The open query is `is_open()` and the commands are `open()`,
//! `close()` and `toggle()` on a Copy `self`. State a caller may own is a
//! builder of that name taking a `Binding` (`sort(binding)`,
//! `widths(binding)`, `anchor(binding)`), read back with `<name>_value()`.
//!
//! Opening and closing is one chain: a disclosure edits the requested open
//! state, a popup adds the overlay's focus and dismissal rules to it, and a
//! tooltip and a dialog add their own rules to a popup. A delayed shown state
//! is a [`DelayedApi`]. The content of a
//! popup, tooltip or dialog is mounted with the free function
//! `show(handle.is_open(), factory)`, in a fresh owner on every opening. A menu
//! and a command palette keep their own `show`: a menu's top level builds its
//! content again when a request changes between the pointer and the anchor
//! origin, and a submenu mounts its level in a new owner while it is open.

#![forbid(unsafe_code)]

mod announcer;
mod axis;
mod calendar;
mod combobox;
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
mod reorder;
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
    CalendarDayState, CalendarGridApi, CalendarOptions, CalendarSelection, CalendarSpan,
    use_calendar_grid,
};
pub use combobox::{ComboboxApi, use_combobox};
pub use delayed::{DelayedApi, DelayedPhase, Delays, use_delayed};
pub use dialog::{DialogApi, DialogMode, DismissOn, use_dialog};
pub use disclosure::{DisclosureApi, DisclosureState, use_disclosure};
pub use drag::{DragApi, DragCoordinates, DragPhase, DragStart, DragState, DragUpdate, use_drag};
pub use drag_settle::{DragAxis, nearest_snap, project_release, pull_to_snap, resist_overshoot};
pub use drag_velocity::DragVelocity;
pub use edit::EditSession;
pub use focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason, ReasonSet};
pub use follow_end::{FollowEndApi, use_follow_end};
pub use hover_corridor::{
    HoverCorridorApi, HoverSurfaceApi, use_hover_corridor, use_hover_surface,
};
pub use list::{
    ListActivation, ListApi, ListBoundary, ListCommandModifier, ListGesture, ListItem,
    ListItemState, ListMissingAnchor, ListMode, ListOptions, ListRangeMode, ListRangeOptions,
    use_list,
};
pub use menu::{
    DisabledMenuFocus, MenuApi, MenuCheck, MenuEntry, MenuItemKind, MenuItemState, MenuOrigin,
    MenuPress, MenuRow, SubmenuTiming, use_menu,
};
pub use menubar::{MenubarApi, use_menubar};
pub use palette::{Command, CommandGroup, CommandPaletteApi, filter_commands, use_command_palette};
pub use policy::grid::GridMove;
pub use policy::scalar::{ScalarRange, ScalarRangeError};
pub use popup::{PopupApi, PopupState, use_popup};
pub use range_scalar::{RangeScalarApi, RangeThumb, use_range_scalar};
pub use reorder::Reorder;
pub use scalar::{ScalarApi, ScalarState, TouchSliderGesture, use_scalar};
pub use select::{SelectApi, use_select};
pub use space_pan::{SpacePanApi, use_space_pan};
pub use splitter::{SplitterApi, SplitterState, use_splitter};
pub use table::{
    ColumnDef, DataTableApi, DataTableModelApi, FilterValue, SortOrder, TableCellState, TableFocus,
    TableHeaderState, TableRowState, use_data_table, use_data_table_model,
};
pub use tabs::{TabListState, TabState, TabsApi, use_tabs};
pub use toaster::{Toast, ToastId, ToastOptions, ToastState, Toaster, toaster};
pub use toggle::{
    ButtonApi, CheckboxApi, SwitchApi, ToggleButtonApi, ToggleState, use_button, use_checkbox,
    use_switch, use_toggle_button,
};
pub use tone::Tone;
pub use tooltip::{TooltipApi, TooltipTiming, use_tooltip};
pub use tree::{
    TreeApi, TreeBranch, TreeItemState, TreeNode, TreeRow, flatten_tree_rows, use_tree,
};

pub use text_edit::{TextEditApi, text_edit_policy, use_text_area, use_text_edit};

pub use zoom_pan::{ViewTransform, ZoomPanApi, use_zoom_pan};

mod inline_edit;
mod interaction_policy;
mod options_cell;
pub use inline_edit::{InlineEditApi, use_inline_edit};

pub use interaction_policy::{
    ButtonPolicy, DisabledButtonFocus, InteractionPolicy, MenuPolicy, PressDrift, SelectPolicy,
    SliderPolicy, SliderTouchMotion, SwitchDrag, SwitchPolicy, ToastPolicy, ToastSwipe,
};

mod menu_policy;
mod tooltip_policy;
