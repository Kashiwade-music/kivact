//! Role: Mounting writes a host node's first properties and listener flags.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::bridge::PropValue;
use kivact_wire::{NodeId, NodeType};

use super::editing;
use super::facts::{LocalPaintSource, NodeFacts};
use crate::view::common::Common;
use crate::view::descriptor::Mcx;
use crate::view::state::ViewState;

/// Stage 3: decide the listener flags and what the style and draw stages
/// need, then write the node's first properties (test id, role, focusable,
/// hit_testable, actions, listeners, intrinsic size, text editing). This is
/// the only stage that borrows the engine tree for its writes at once.
pub(super) fn write(
    cx: &Mcx<'_>,
    id: NodeId,
    kind: NodeType,
    c: &mut Common,
    actions: Vec<crate::view::A11yAction>,
    at: &'static Location<'static>,
) -> NodeFacts {
    let listen_file = !c.handlers.file.is_empty();
    let listen_pointer = !c.handlers.pointer.is_empty();
    let listen_wheel = !c.handlers.wheel.is_empty();
    let has_style = !c.style.is_empty();
    // A focus stop gets the theme's focus ring and disabled look (see
    // `Style::compile`), styled or not.
    let focus_stop = c.focusable == Some(true)
        || c.text_editing.is_some()
        || c.semantics.as_ref().is_some_and(|s| s.focusable.is_some());
    let has_local_paint = has_style && kind == NodeType::Box && c.text_editing.is_none();
    let has_explicit_draw = c.draw.is_some() || c.draw_over.is_some();
    let has_draw = has_explicit_draw || has_local_paint;
    let local_paint = has_local_paint.then(|| LocalPaintSource {
        value: Rc::new(RefCell::new(None)),
        revision: cx.owner.trigger(at),
    });
    {
        let writer = cx.rt.running_effect();
        let mut tree = ViewState::tree_mut(cx.rt);
        if let Some(t) = c.test_id.take() {
            tree.update(id, PropValue::TestId(t.to_string()), writer);
        }
        if let Some(n) = tree.get_mut(id) {
            n.at = Some(at);
            n.handlers = Some(Box::new(std::mem::take(&mut c.handlers)));
        }
        if let Some(r) = c.role {
            tree.update(id, PropValue::Role(r), writer);
        }
        if let Some(f) = c.focusable {
            tree.update(id, PropValue::Focusable(f), writer);
        }
        if let Some(h) = c.hit_testable {
            tree.update(id, PropValue::HitTestable(h), writer);
        }
        if !actions.is_empty() {
            tree.update(id, PropValue::Actions(actions), writer);
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
        if let Some(i) = c.intrinsic {
            tree.update(id, PropValue::Intrinsic(i), writer);
        }
        if has_explicit_draw {
            tree.set_observe(id, true);
        }
    }
    if let Some(editing) = c.text_editing.take() {
        editing::bind(cx, id, *editing, at);
    }
    NodeFacts {
        kind,
        focus_stop,
        has_draw,
        local_paint,
    }
}
