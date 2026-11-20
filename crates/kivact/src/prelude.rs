//! Role: Everyday names: what the introductory guides use without a path.
//!
//! Everything else is imported from its module (`kivact::ui::data_table`,
//! `kivact::state::Machine`, ...). No name is exported from two modules.

pub use crate::{App, Prop, UiError, bind, component};

pub use crate::state::{
    Callback, Signal, Trigger, Value, Var, binding, callback, effect, effect_on, memo, on_cleanup,
    on_mount, provide, signal, trigger, untrack, use_context, var,
};

pub use crate::view::{
    CheckedState, Element, Handled, IntoView, Key, KeyEvent, MouseButton, NodeRef, PointerEvent,
    PointerPhase, Role, View, ViewFn, WheelEvent, canvas, column, dynamic, error_boundary,
    for_each, for_index, grid, icon, image, node, node_ref, row, scroll, show, show_some, spacer,
    stack, text, virtual_list, window,
};

pub use crate::style::{
    Align, Color, Cursor, Justify, Overflow, Position, Style, TextAlign, Theme, ThemeMode, em, pct,
    provide_theme, rgb, style, use_theme,
};

pub use crate::draw::{Draw, Path, Shadow, Stroke};

pub use crate::geometry::{Point, Rect, Size};

pub use crate::task::{Action, AsyncState, Resource, action, resource, sleep, spawn};

pub use crate::ui::{
    ButtonVariant, ControlSize, async_view, button, checkbox, dialog, field, list, menu,
    radio_group, select, slider, switch, tabs, text_area, text_input, toast_region, tooltip,
};

pub use crate::interaction::{ListItem, MenuEntry, Toast};

pub use crate::i18n::message;
