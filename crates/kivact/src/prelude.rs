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
    CheckedState, EditPhase, Handled, InputPurpose, IntoView, Key, KeyEvent, Modifiers, Node,
    NodeBuilder, NodeRef, Orientation, Placement, PointerEvent, Role, RowHeight, ScrollAxes, View,
    WindowMode, WindowPlacement, canvas, column, component, drawable, dynamic, error_boundary,
    for_each, for_index, grid, icon, image, node, node_ref, row, scroll, show, show_some, spacer,
    stack, text, virtual_list, window,
};

pub use crate::style::{
    Align, Color, ColorToken, Justify, Style, Theme, ThemeColor, ThemeMode, pct, provide_theme,
    rgb, style,
};

pub use crate::style::{use_theme, use_tokens};

pub use crate::motion::{Easing, Spring, Transition, animated, frame_loop, timeline};

pub use crate::draw::Transform;

pub use crate::task::ActionMode;

pub use crate::task::{Action, AsyncState, Resource, action, blocking, resource, spawn};

pub use crate::i18n::message;

pub use crate::interaction::{DialogMode, DismissOn};

pub use crate::interaction::{IntoSelection, Selection};

pub use crate::interaction::{ListItem, MenuEntry, Toast, Tone, toaster};

pub use crate::ui::{
    DataColumn, PageItem, StepStatus, StepperItem, breadcrumb, button, button_group, checkbox,
    combobox, data_table, dialog, field, list, menu, navigation_menu, pagination, radio_group,
    select, selection_field, slider, stepper, switch, tabs, text_area, text_input, toast_region,
    toggle_group, toolbar, tooltip,
};

pub use crate::ui::async_view;

pub use crate::ui::{calendar, date_input, date_range_input, date_time_input, time_input};
pub use crate::ui::{
    item_list, item_row, table, table_body, table_cell, table_footer, table_head, table_header,
    table_row, tree,
};
