//! Role: Mounting wires a node's text editing callbacks and binds its editing data.

use std::rc::Rc;

use kivact_engine::bridge::PropValue;

use crate::view::common::Common;
use crate::view::descriptor::Mcx;
use kivact_wire::NodeId;
use std::panic::Location;

/// Stage 1: compose `on_edit` and `on_selection` with the handlers that were
/// registered before them (the earlier handler runs first, and an error from
/// it stops the callback). `on_selection` also makes the host report
/// selection sessions.
pub(super) fn compose_callbacks(c: &mut Common) {
    if let Some(editing) = c.text_editing.as_mut()
        && let Some(handler) = editing.on_edit.take()
    {
        let previous = c.handlers.text.take();
        c.handlers.text = Some(Rc::new(move |event| {
            if let Some(previous) = &previous {
                previous(event.clone())?;
            }
            handler(event)
        }));
    }
    if let Some(editing) = c.text_editing.as_mut()
        && let Some(handler) = editing.on_selection.take()
    {
        editing.reports_session = true;
        let previous = c.handlers.selection.take();
        c.handlers.selection = Some(Rc::new(move |session| {
            if let Some(previous) = &previous {
                previous(session.clone())?;
            }
            handler(session)
        }));
    }
}

/// Write the editing payload of a text-editing node and bind its values.
pub(super) fn bind(
    cx: &Mcx<'_>,
    id: NodeId,
    editing: crate::view::editing_data::TextEditing,
    at: &'static Location<'static>,
) {
    use kivact_editing::{EditBehavior, EditOutcome};
    use kivact_wire::EditingData;
    // The owner keeps the author's function; the engine, which may drop
    // its copy while its tree is borrowed, calls it through a `SyncHook`
    // (a panic leaves the request unhandled, the engine takes back what it
    // changed, and the panic is reported once after the engine lets go of
    // its model).
    let behavior = editing.behavior.map(|f| {
        let (hook, hooked) =
            crate::view::sync_hook::SyncHook::new(Rc::downgrade(cx.rt), cx.owner, at, (), f);
        cx.owner.on_cleanup(move || drop(hooked));
        EditBehavior::new(move |state, request, context| {
            hook.call(|(), f| f(state, request, context))
                .unwrap_or(EditOutcome::NotEditing)
        })
    });
    cx.update(
        id,
        PropValue::Editing(Some(EditingData {
            behavior,
            report_session: editing.reports_session.then_some(true),
            ..EditingData::default()
        })),
    );
    if let Some(bind) = editing.highlight {
        bind(crate::view::editing_data::HighlightAt {
            rt: cx.rt.clone(),
            owner: cx.owner,
            theme: cx.theme(),
            id,
            at,
        });
    }
    cx.update(id, PropValue::Multiline(editing.multiline));
    if editing.report_text {
        cx.update(id, PropValue::ReportText(true));
    }
    cx.bind(id, editing.value, PropValue::InputValue, at);
    if let Some(keys) = editing.forward_keys {
        cx.bind(id, keys, PropValue::ForwardKeys, at);
    }
    if let Some(value) = editing.password {
        cx.bind(id, value, PropValue::Password, at);
    }
    if let Some(value) = editing.read_only {
        cx.bind(id, value, PropValue::ReadOnly, at);
    }
    if let Some(value) = editing.max_length {
        cx.bind(id, value, PropValue::MaxLength, at);
    }
    if let Some(value) = editing.placeholder {
        cx.bind(id, value, PropValue::Placeholder, at);
    }
}

/// Stage 8: bind the IME session of a node that edits its own text. It
/// follows the draw stage, so a session read in the same turn sees what the
/// node's drawing made (a caret rectangle taken from the paragraph it drew).
pub(super) fn bind_session(
    cx: &Mcx<'_>,
    id: NodeId,
    session: crate::view::ime_session::ImeSession,
    at: &'static Location<'static>,
) {
    use kivact_wire::ImeSession;
    let weak = cx.weak();
    cx.effect(kivact_reactive::EffectKind::Render, at, move || {
        let value = ImeSession {
            text: session.text.get(),
            selection: session.selection.get(),
            caret: session.caret.get(),
            password: session.password,
        };
        if let Some(rt) = weak.upgrade() {
            crate::view::state::ViewState::tree_mut(&rt).update(
                id,
                PropValue::ImeSession(value),
                rt.running_effect(),
            );
        }
    });
}
