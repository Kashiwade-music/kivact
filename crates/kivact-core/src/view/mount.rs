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
mod resolve;
mod semantics;
mod style;
use super::common::Common;
use resolve::Resolved;

use kivact_engine::contract::NodeType;
use kivact_types::{LayoutProps, NodeId};

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
/// `resolve::resolve` consumes the declared `Common` once and hands each stage its own input
/// (`Resolved`), so no stage reads a field another stage took or rewrote. What remains is the
/// order the effects are created in:
/// 1. `create::write`: the first write (handlers, listener flags, focusability, text editing).
/// 2. `semantics::bind_props`: semantics and image properties.
/// 3. `style::bind_states`: disabled and selected.
/// 4. `style::bind_node_style`: the style effect.
/// 5. `draw::bind`: the draw effect.
/// 6. `semantics::bind_late` and `editing::bind_session`: effects that read what the drawing made.
/// 7. `attachments::finish`: references, part hooks and regions, after every handler is installed.
pub(crate) fn mount_host_with(
    cx: &Mcx<'_>,
    kind: NodeType,
    c: Common,
    at: &'static Location<'static>,
    fix_layout: Option<fn(&mut LayoutProps)>,
) -> NodeId {
    // `.at(..)` replaces the builder's call site.
    let at = c.at.unwrap_or(at);
    let id = cx.host(kind);
    let Resolved {
        first,
        semantics,
        disabled,
        selected,
        style,
        facts,
        draw,
        late_semantics,
        ime,
        refs,
        attached,
    } = resolve::resolve(cx, id, kind, c, at);
    create::write(cx, id, first, at);
    semantics::bind_props(cx, id, semantics, at);
    let states = style::bind_states(cx, id, disabled, selected, at);
    style::bind_node_style(cx, id, style, &facts, states, fix_layout, at);
    if let Some(draw) = draw {
        draw::bind(cx, id, draw, at);
    }
    semantics::bind_late(cx, id, late_semantics, at);
    if let Some(session) = ime {
        editing::bind_session(cx, id, session, at);
    }
    attachments::finish(cx, id, refs, attached, at);
    id
}
