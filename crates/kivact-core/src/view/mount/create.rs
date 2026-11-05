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

// xtask-allow(state-fields): Independent resolved node properties are consumed by the initial publication stage.
/// The node's first write.
pub(super) struct FirstWrite {
    pub(super) exit_layout: crate::view::ExitLayout,
    pub(super) exit_complete: Option<kivact_runtime::Handler<()>>,
    pub(super) focusable_when_disabled: Option<InputProp<bool>>,
    pub(super) roving_focus: Option<InputProp<kivact_engine::contract::RovingFocus>>,
    pub(super) hit_testable: Option<InputProp<bool>>,
    /// The accessibility actions the node offers, its own and its parts'.
    /// The final handlers (routed through the parts' defaults when the node has parts).
    pub(super) handlers: Handlers,
    pub(super) intrinsic: Option<IntrinsicSize>,
    /// The node has an explicit draw layer: its layout is observed for the layer's size.
    pub(super) observe_layout: bool,
    pub(super) editing: Option<EditorProps>,
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
            n.exit_complete = exit_complete;
            n.handlers = Some(Box::new(handlers));
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
