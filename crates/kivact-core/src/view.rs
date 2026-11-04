//! Role: Views describe elements, control flow, input and semantics.

// Also the rest of the view machinery: the owner boundaries that wrap a
// child (`provide`, `provide_theme`), the App's root (root owner,
// environment, focus source, theme and root node), the retained tree,
// overlays and modals (the builder, the overlay index and the input policy
// the client applies while they are open), keyboard focus, the pointer's
// hover and the requests to the host (focus and platform effects).

#![allow(
    clippy::type_complexity,
    reason = "handler and effect closures are stored as boxed trait objects throughout"
)]

pub(crate) mod boundary;
pub(crate) mod common;
pub(crate) mod descriptor;
mod exit_layout;
pub(crate) mod flow;
pub(crate) mod focus;
pub(crate) mod handler_layers;
pub(crate) mod hover;
pub(crate) mod input;
pub(crate) mod modifier_keys;
pub(crate) mod modifiers;
pub(crate) mod mount;
pub(crate) mod node;
pub(crate) mod node_ref;
pub(crate) mod overlays;
pub(crate) mod owner;
pub(crate) mod part;
pub(crate) mod part_data;
pub(crate) mod requests;
pub(crate) mod root;
pub(crate) mod span;
pub(crate) mod state;
pub(crate) mod sync_hook;
pub(crate) mod text_edits;
pub(crate) mod tree;

#[doc(inline)]
pub use crate::view::flow::{
    ErrorBoundary, RowHeight, Show, ShowSome, VirtualList, dynamic, error_boundary, for_each,
    for_index, show, show_some, virtual_list,
};
#[doc(inline)]
pub use boundary::{
    ReadingDirection, current_direction, provide, provide_direction, provide_locale,
    provide_messages, provide_theme,
};
#[doc(inline)]
pub use descriptor::{IntoView, View};
#[doc(inline)]
pub use hover::mouse_position;
#[doc(inline)]
pub use input::{
    A11yEvent, ButtonPress, DismissReason, FileEvent, FilePhase, Handled, Key, KeyCode, KeyEvent,
    KeyHandlerResult, Modifiers, MouseButton, PointerButtons, PointerEvent, PointerId, PointerKind,
    PointerPhase, WheelEvent,
};
#[doc(inline)]
pub use kivact_runtime::HandlerResult;
#[doc(inline)]
pub use kivact_runtime::{IntoReactive, IntoText, IntoValue, Reactive};
#[doc(inline)]
pub use kivact_runtime::{callback, constant, selector};
#[doc(inline)]
pub use modifier_keys::current_modifiers;

/// Lets a component author take their own type as a prop.
///
/// `constant_value!(MyType)` makes a plain `MyType` acceptable wherever
/// `impl IntoValue<MyType>` is taken, next to the signals, `Value`s and
/// closures that already are. Register the type first, then pass constants
/// directly to props or call `x.into_value()` to create a read-only handle.
/// Without registration, pass a signal, `Value` or closure, or wrap the
/// constant with [`constant`] (`badge(constant(Tone::Calm))`). The type must be `Clone + PartialEq`, like every prop type.
/// List several types at once with commas,
/// and invoke the macro once per type, in the crate that defines it.
///
/// ```no_run
/// # use kivact_core as kivact;
/// use kivact::prelude::*;
/// use kivact::view::constant_value;
///
/// #[derive(Clone, Copy, PartialEq)]
/// pub enum Tone {
///     Calm,
///     Alert,
/// }
///
/// constant_value!(Tone);
///
/// pub fn badge(label: impl IntoText, tone: impl IntoValue<Tone>) -> Node {
///     let label = label.into_text_value();
///     let tone = tone.into_value();
///     node()
///         .name(label)
///         .style(move || match tone.get() {
///             Tone::Calm => style().background(ColorToken::Surface),
///             Tone::Alert => style().background(ColorToken::SurfaceAlt),
///         })
///         .child(text(label))
/// }
///
/// let status = signal(Tone::Calm);
/// let _ = (
///     badge("Saved", Tone::Calm),
///     badge("Saved", status),
///     badge("Saved", move || status.get()),
/// );
/// ```
#[doc(inline)]
pub use kivact_runtime::constant_value;

#[doc(inline)]
pub use exit_layout::ExitLayout;
#[doc(inline)]
pub use kivact_runtime::{context, try_context};
#[doc(inline)]
pub use kivact_types::{
    A11yAction, A11yRequest, CheckedState, CurrentItem, LiveRegion, Orientation, PopupKind, Role,
    SortDirection,
};
#[doc(inline)]
pub use node_ref::{NodeRef, ScrollBehavior, node_ref};
#[doc(inline)]
pub use overlays::{Overlay, OverlayAlign, OverlayViewport, Placement, overlay, overlay_at};

pub(crate) use part::Part;

pub(crate) mod editing_data;
pub(crate) mod ime_session;
