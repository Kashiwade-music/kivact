//! Role: Everyday names used by the introductory guides.

pub use crate::{App, UiError};

pub use crate::state::use_context;

pub use crate::state::{
    Binding, Callback, IntoText, IntoValue, Signal, Value, Var, binding, callback, constant_value,
    effect, effect_on, memo, on_cleanup, on_mount, provide, signal, untrack, var,
};

pub use crate::state::{IntoBinding, ItemKey, PropValue, bind};

pub use crate::geometry::{Point, Size};

pub use crate::view::Component;

pub use crate::view::{
    CheckedState, EditPhase, Handled, IntoView, Key, KeyEvent, Modifiers, Node, NodeBuilder,
    NodeRef, Orientation, Placement, PointerEvent, Role, RowHeight, ScrollAxes, View, canvas,
    column, dynamic, error_boundary, for_each, for_index, grid, icon, image, node, node_ref, row,
    scroll, show, show_some, spacer, stack, text, virtual_list,
};

pub use crate::style::{
    Align, Color, ColorToken, Justify, Style, Theme, ThemeColor, ThemeMode, pct, provide_theme,
    rgb, style,
};

pub use crate::style::{use_theme, use_tokens};

pub use crate::task::ActionMode;

pub use crate::task::{Action, AsyncState, Resource, action, blocking, resource, spawn};

pub use crate::i18n::message;

pub use crate::interaction::{DialogMode, DismissOn};

pub use crate::interaction::{IntoSelection, Selection};

pub use crate::interaction::{ListItem, MenuEntry, Toast, Tone, toaster};

pub use crate::ui::{
    DataColumn, button, button_group, checkbox, combobox, data_table, dialog, field, list, menu, radio_group,
    select, slider, switch, tabs, text_area, text_input, toast_region, toggle_group, toolbar,
    tooltip,
};

pub use crate::ui::async_view;
