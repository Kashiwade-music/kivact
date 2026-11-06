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
pub(crate) mod descriptor;
pub(crate) mod element_data;
mod exit_layout;
pub(crate) mod flow;
pub(crate) mod focus;
pub(crate) mod handler_layers;
pub(crate) mod hover;
pub(crate) mod input;
pub(crate) mod modifier_keys;
pub(crate) mod modifiers;
pub(crate) mod mount;
mod node_draw;
mod node_modifiers;
pub(crate) mod node_ref;
mod node_status;
pub(crate) mod node_status_data;
pub(crate) mod overlays;
pub(crate) mod owner;
pub(crate) mod part;
pub(crate) mod part_data;
pub(crate) mod ref_data;
pub(crate) mod requests;
pub(crate) mod root;
pub(crate) mod semantics_data;
pub(crate) mod snapshots;
pub(crate) mod span;
pub(crate) mod state;
pub(crate) mod sync_hook;
pub(crate) mod text_edits;
pub(crate) mod tree;

#[doc(inline)]
pub use crate::view::flow::{
    ErrorBoundary, ListSource, RowHeight, Show, ShowSome, VirtualList, dynamic, error_boundary,
    for_each, for_index, show, show_some, virtual_list,
};
#[doc(inline)]
pub use boundary::{ReadingDirection, component, provide_direction, use_direction};
#[doc(inline)]
pub use descriptor::{IntoView, IntoViewFn, View, ViewFn};
pub use element_data::Component;
#[doc(inline)]
pub use hover::pointer_position;
#[doc(inline)]
pub use input::{
    A11yEvent, ButtonPress, Composition, DismissReason, FileEvent, FilePhase, Handled,
    InputPurpose, Key, KeyCode, KeyEvent, Modifiers, MouseButton, PointerButtons, PointerEvent,
    PointerId, PointerKind, PointerPhase, WheelEvent,
};
#[doc(inline)]
pub use kivact_runtime::HandlerResult;
pub use kivact_types::Presence;
#[doc(inline)]
pub use modifier_keys::modifiers;

#[doc(inline)]
pub use exit_layout::ExitLayout;
pub use kivact_types::ImageStatus;
#[doc(inline)]
pub use kivact_types::{
    A11yAction, A11yRequest, CheckedState, CurrentItem, LiveRegion, Orientation, PopupKind, Role,
    SortDirection,
};
#[doc(inline)]
pub use node_ref::{NodeRef, ScrollBehavior, node_ref};
pub use node_status_data::ScrollState;
#[doc(inline)]
pub use overlays::{Overlay, OverlayAlign, Placement, overlay, overlay_at};

pub(crate) use part::Part;

pub(crate) mod editing_data;
pub(crate) mod ime_session;

mod children;
mod edit_phase;
pub(crate) mod elements;

pub use children::IntoChildren;
pub use edit_phase::EditPhase;
pub use elements::{
    ImageFit, Node, NodeBuilder, Scroll, ScrollAxes, Span, SurfaceSize, Viewport, canvas, column,
    grid, icon, image, node, rich_text, row, scroll, spacer, span, stack, text, viewport,
};
pub use kivact_style::asset::{Asset, AssetBytes};

#[doc(hidden)]
pub use flow::ListInput;

#[doc(hidden)]
pub use elements::ViewportTransform;
