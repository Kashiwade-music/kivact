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
//! `control()`.
//!
//! Every floating or collapsible surface (disclosure, popup, dialog, menu,
//! select, combobox, command palette, hover card, navigation panel) takes the
//! caller's `open: Signal<bool>` as a required argument, and that signal is
//! its only open state. `is_open()` reads it; `show()`, `hide()` and
//! `toggle()` write it on a Copy `self`: `show` and `hide` change nothing when
//! the surface already is so, and `toggle` inverts. A kind of surface is a
//! suffix (`show_sheet()`). A toaster's `dismiss(id)` removes one toast from
//! its queue; it is not a show/hide command. The parts are `trigger()` and
//! `content()`. A surface's content mounts with `show(api.is_open(), ..)`
//! around an `overlay(..)` carrying `content()`, or in one call with
//! `api.surface(placement, content)`, which anchors the overlay at the
//! trigger part and mounts the content in a fresh owner on every opening (a
//! menu's top level again when its origin changes between the pointer and the
//! anchor). A popup adds the overlay's focus and dismissal rules to a
//! disclosure; a dialog, a menu, a select and a navigation panel add theirs to
//! a popup. A delayed shown state is a [`DelayedApi`]. Other state a caller
//! may own is a builder of that name taking a `Signal` (`sort(signal)`,
//! `widths(signal)`), read back with `<name>_value()`.

#![forbid(unsafe_code)]

mod announcer;
mod axis;
mod calendar;
mod collection;
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
mod history;
mod hover_corridor;
mod list;
mod menu;
mod menubar;
mod month_swipe;
mod palette;
mod policy;
mod popup;
mod press;
mod range_scalar;
mod reorder;
mod reorder_drag;
mod roving;
mod scalar;
mod sealed;
mod selection;
mod sheet_tap;
mod swipe_dismiss;
mod switch_drag;
pub use selection::{IntoSelection, Selection};
mod date_field;
mod draft;
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

pub use announcer::{Announcement, Announcer, use_announcer};
pub use calendar::{
    CalendarDayState, CalendarGridApi, CalendarOptions, CalendarSelection, CalendarSpan,
    DaySelection, use_calendar_grid,
};
pub use combobox::{ComboboxApi, ComboboxOptions, KnownOptions, known_options, use_combobox};
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
    DisabledMenuFocus, MenuApi, MenuCheck, MenuEntry, MenuItemDisplay, MenuItemKind, MenuItemState,
    MenuOrigin, MenuRow, PressOver, SubmenuTiming, use_menu,
};
pub use menubar::{MenubarApi, use_menubar};
pub use palette::{Command, CommandGroup, CommandPaletteApi, filter_commands, use_command_palette};
pub use policy::grid::GridMove;
pub use policy::scalar::{ScalarRange, ScalarRangeError, ScalarStep};
pub use policy::scalar_mapping::ScalarMapping;
pub use popup::{PopupApi, PopupState, use_popup};
pub use range_scalar::{RangeScalarApi, RangeThumb, use_range_scalar};
pub use reorder::Reorder;
pub use reorder_drag::{ReorderDragApi, ReorderPlace, use_reorder_drag};
pub use scalar::{ScalarApi, ScalarState, TouchSliderGesture, use_scalar};
pub use select::{SelectApi, use_select};
pub use sheet_tap::{SheetTapApi, use_sheet_tap};
pub use space_pan::{SpacePanApi, use_space_pan};
pub use splitter::{SplitterApi, SplitterState, use_splitter};
pub use swipe_dismiss::{
    SwipeDismissApi, SwipeDismissPolicy, SwipeInputs, SwipePhase, use_swipe_dismiss,
};
pub use switch_drag::{Lift, SwitchDragApi, SwitchTrack, ThumbDrag, TrackInput, use_switch_drag};
pub use table::{
    ColumnDef, DataTableApi, DataTableModelApi, FilterValue, HeaderSort, SortOrder, TableCellState,
    TableFocus, TableHeaderState, TableRowState, use_data_table, use_data_table_model,
};
pub use tabs::{TabListState, TabState, TabsApi, use_tabs};
pub use toaster::{Toast, ToastId, ToastOptions, ToastState, ToasterApi, use_toaster};
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

pub use date_field::{
    DateFieldApi, DateFieldOptions, DatePanel, DatePicker, DatePickerApi, TextCheck, TypedCheck,
    TypedText, use_date_field, use_date_picker,
};
pub use draft::{DraftApi, DraftCommit, DraftOptions, use_draft};
pub use history::{HistoryApi, use_history};
pub use month_swipe::{MonthSwipeApi, MonthSwipePhase, use_month_swipe};

mod inline_edit;
mod interaction_policy;
mod options_cell;
pub use inline_edit::{InlineEditApi, use_inline_edit};

pub use interaction_policy::{
    ButtonPolicy, DisabledButtonFocus, InteractionPolicy, MenuPolicy, ModifierKey, PressDrift,
    ScrubPolicy, SelectPolicy, SliderPolicy, SliderTouchMotion, SwitchPolicy, ToastPolicy,
    ToastSwipe,
};

mod menu_policy;
mod tooltip_policy;

mod carousel;
pub use carousel::{CarouselApi, CarouselState, Snap, use_carousel};

mod dropzone;
pub use dropzone::{DropzoneApi, DropzoneState, FileInfo, Validity, use_dropzone};
mod copy_feedback;
pub use copy_feedback::{CopyFeedbackApi, use_copy_feedback};

mod disclosure_group;
pub use disclosure_group::{DisclosureGroupApi, use_disclosure_group};

mod surface_policy;
pub use surface_policy::{DrawerPolicy, DrawerSwipe};

mod hover_card;
pub use hover_card::{HoverCardApi, HoverCardOpenedBy, HoverCardState, use_hover_card};
mod navigation_menu;
mod surface;
pub use navigation_menu::{NavOpen, NavigationMenuApi, NavigationPanelApi, use_navigation_menu};
