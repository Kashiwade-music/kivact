//! Role: Everyday types and functions used by the introductory guides.
//!
//! The facade adds standard UI functions and `binding` to these core exports.
//! Drawing values, optional context lookup and specialized services stay in
//! their defining modules. The public-name inventory fixes the complete list.

pub use crate::control::{
    Node, NodeBuilder, column, grid, icon, image, node, row, scroll, spacer, stack, text,
};
pub use crate::task::{
    Action, ActionPolicy, AsyncState, Resource, action, blocking, resource, spawn,
};
pub use crate::view::{
    IntoText, IntoValue, IntoView, Key, NodeRef, Orientation, Role, View, context, dynamic,
    for_each, for_index, node_ref, provide, show, show_some, virtual_list,
};
pub use crate::{
    App, Signal, UiError, Value, Var, effect, memo, on_cleanup, on_mount, signal, var,
};
pub use kivact_style::{Align, ColorToken, Justify, Theme, ThemeColor, pct, rgb, style};
