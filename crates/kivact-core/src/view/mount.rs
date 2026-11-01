//! Role: Mounting turns descriptions into retained host nodes.
//!
//! Mounting: descriptions become retained nodes plus one render effect per
//! dynamic prop. Each render effect only computes its value and hands it to
//! the engine writer, which records only values that differ.
//!
//! A builder is an unmounted description, and a `View` runs once, when it
//! mounts. `mount_host` passes the typed properties and the payload straight
//! to the engine's create and makes a Render effect for each dynamic value
//! that calls a comparing setter (an equal value writes nothing). Core keeps
//! no unsent prop or scene and no index of creates or prop sets; a row
//! without a parent yet is made as a detached engine root. While native
//! edits wait, the controlled value is kept from overwriting them by the
//! count of `TextChanged` events still to deliver (`crate::view::tree`).

use std::panic::Location;

mod attachments;
mod create;
mod diagnostics;
mod draw;
mod editing;
mod facts;
mod part_input;
mod part_pointer;
mod parts;
pub(crate) mod region_collection;
mod region_template;
pub(crate) mod regions;
mod semantics;
mod style;
use super::common::Common;

use kivact_wire::{LayoutProps, NodeId, NodeType};

use super::descriptor::Mcx;

pub(crate) use style::{NodeStyle, bind_style};

/// Mount one host node with its common modifiers. Children are appended by the caller.
pub(crate) fn mount_host(
    cx: &Mcx<'_>,
    kind: NodeType,
    c: Common,
    at: &'static Location<'static>,
) -> NodeId {
    mount_host_with(cx, kind, c, at, None)
}

/// [`mount_host`] for a builder that owns part of its node's layout:
/// `fix_layout` runs on every layout resolved from the style layers (the
/// style is resolved even without layers) before it is set.
/// `virtual_list` keeps its vertical scroll this way.
///
/// A node is mounted in stages, each taking its own fields out of the
/// `Common` (the order matters: handlers are routed before the listener
/// flags are read, the node's first write comes before any binding, and
/// the references and bind hooks come last):
/// 1. `editing::compose_callbacks`: text edit callbacks.
/// 2. `attachments::bind`: parts and canvas regions.
/// 3. `create::write`: listener flags and the first write.
/// 4. `semantics::bind_props`: semantics and image properties.
/// 5. `style::bind_states`: disabled and selected.
/// 6. `style::bind_node_style`: the style effect.
/// 7. `draw::bind`: the draw effect.
/// 8. `semantics::bind_late` and `editing::bind_session`: the node's own text
///    and its IME session.
pub(crate) fn mount_host_with(
    cx: &Mcx<'_>,
    kind: NodeType,
    mut c: Common,
    at: &'static Location<'static>,
    fix_layout: Option<fn(&mut LayoutProps)>,
) -> NodeId {
    // `.at(..)` replaces the builder's call site.
    let at = c.at.take().unwrap_or(at);
    parts::reject_modal(&mut c, cx, at);
    diagnostics::report_unreachable_press(&c, cx, at);
    let id = cx.host(kind);
    editing::compose_callbacks(&mut c);
    let mut attachments = attachments::bind(&mut c, cx, id, at);
    let actions = std::mem::take(&mut attachments.actions);
    let facts = create::write(cx, id, kind, &mut c, actions, at);
    semantics::bind_props(cx, id, &mut c, at);
    let states = style::bind_states(cx, id, &mut c, at);
    style::bind_node_style(cx, id, &mut c, &facts, states, fix_layout, at);
    draw::bind(cx, id, &mut c, &facts, attachments.regions(), at);
    semantics::bind_late(cx, id, &mut c, at);
    if let Some(session) = c.ime.take() {
        editing::bind_session(cx, id, session, at);
    }
    attachments.finish(cx, id, &mut c, at);
    id
}
