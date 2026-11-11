//! Role: Mounting writes a host node's first properties and listener flags.

use std::panic::Location;

use kivact_engine::api::PropValue;
use kivact_runtime::Prop as InputProp;
use kivact_types::{IntrinsicSize, NodeId};

use super::editing;
use crate::view::descriptor::Mcx;
use crate::view::editing_data::EditorProps;
use crate::view::state::ViewState;
use crate::view::tree::Handlers;

/// The node's first write.
pub(super) struct FirstWrite {
    pub(super) exit_layout: crate::view::ExitLayout,
    pub(super) exit_complete: Option<kivact_runtime::Handler<()>>,
    // xtask-allow(state-fields): option: a builder input carried to mount; absent means not focusable while disabled
    pub(super) focusable_when_disabled: Option<InputProp<bool>>,
    // xtask-allow(state-fields): option: a builder input carried to mount; absent means no roving focus
    pub(super) roving_focus: Option<InputProp<kivact_engine::contract::RovingFocus>>,
    // xtask-allow(state-fields): option: a builder input carried to mount; absent means the node hit-tests
    pub(super) hit_testable: Option<InputProp<bool>>,
    /// The accessibility actions the node offers, its own and its parts'.
    /// The final handlers (routed through the parts' defaults when the node has parts).
    pub(super) handlers: Handlers,
    // xtask-allow(state-fields): option: a builder input carried to mount; absent means none
    pub(super) intrinsic: Option<IntrinsicSize>,
    // xtask-allow(state-fields): option: set once by finish_resolve when the node has a draw, draw_over or draw_with_layout layer; false means the layout is not observed for a layer's size
    /// The node has an explicit draw layer: its layout is observed for the layer's size.
    pub(super) observe_layout: bool,
    // xtask-allow(state-fields): option: a builder input carried to mount; absent means the node does not edit
    pub(super) editing: Option<EditorProps>,
}

/// Whether `handlers` is the default record: the node declared no handler.
fn is_default_handlers(handlers: &Handlers) -> bool {
    handlers.activate.is_none()
        && handlers.secondary_context.is_none()
        && handlers.text.is_none()
        && handlers.selection.is_none()
        && handlers.edit_phase.is_none()
        && handlers.image_error.is_none()
        && handlers.scroll.is_none()
        && handlers.layout.is_none()
        && handlers.a11y.is_empty()
        && handlers.key.is_empty()
        && handlers.file.is_empty()
        && handlers.key_up.is_empty()
        && handlers.pointer.is_empty()
        && handlers.wheel.is_empty()
        && handlers.ime.is_empty()
}

/// Stage 1: decide the listener flags from the final handlers, then write the node's first
/// properties (test id, role, focusable, hit_testable, actions, listeners, intrinsic size, text
/// editing). This is the only stage that borrows the engine tree for its writes at once.
pub(super) fn write(cx: &Mcx<'_>, id: NodeId, first: FirstWrite, at: &'static Location<'static>) {
    let FirstWrite {
        exit_layout,
        exit_complete,
        focusable_when_disabled,
        roving_focus,
        hit_testable,
        handlers,
        intrinsic,
        observe_layout,
        editing,
    } = first;
    let listen_file = !handlers.file.is_empty();
    let listen_pointer = !handlers.pointer.is_empty() || handlers.secondary_context.is_some();
    let listen_wheel = !handlers.wheel.is_empty();
    // The engine reports the selection sessions of a node that has a session handler.
    let report_session = handlers.selection.is_some();
    {
        let writer = cx.rt.running_effect();
        let mut tree = ViewState::tree_mut(cx.rt);
        if let Some(n) = tree.get_mut(id) {
            n.at = Some(at);
            n.exit_layout = exit_layout;
            if exit_complete.is_some() {
                n.rare_mut().exit_complete = exit_complete;
            }
            // A node that declared no handler keeps no record of them: an
            // observer added later creates it (`ViewNodes::handlers_mut`).
            if !is_default_handlers(&handlers) {
                n.handlers = Some(Box::new(handlers));
            }
        }
        if listen_file {
            tree.update(id, PropValue::ListenFile(true), writer);
        }
        if listen_pointer {
            tree.update(id, PropValue::ListenPointer(true), writer);
        }
        if listen_wheel {
            tree.update(id, PropValue::ListenWheel(true), writer);
        }
        if let Some(i) = intrinsic {
            tree.update(id, PropValue::Intrinsic(i), writer);
        }
        if observe_layout {
            tree.set_observe(id, true);
        }
    }
    if let Some(f) = roving_focus {
        cx.bind(id, f, PropValue::RovingFocus, at);
    }
    if let Some(f) = hit_testable {
        cx.bind(id, f, PropValue::HitTestable, at);
    }
    if let Some(f) = focusable_when_disabled {
        cx.bind(id, f, PropValue::FocusableWhenDisabled, at);
    }
    if let Some(props) = editing {
        editing::bind(cx, id, props, report_session, at);
    }
}
