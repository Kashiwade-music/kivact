//! Role: Mounting builds one host node from its description and effects.

use std::panic::Location;

use kivact_engine::contract::NodeType;
use kivact_runtime::Prop as InputProp;
use kivact_types::{LayoutProps, NodeId};

use super::resolve::{self, Resolved};
use super::{attachments, create, draw, editing, semantics, style};
use crate::view::descriptor::Mcx;
use crate::view::element_data::Common;

/// Mount one host node with its common modifiers. Children are appended by the caller.
/// `site` is the call site recorded on the node (`None` for a node the App did not write).
pub(crate) fn mount_host(
    cx: &Mcx<'_>,
    kind: NodeType,
    c: Common,
    site: Option<&'static Location<'static>>,
) -> NodeId {
    mount_node(cx, kind, c, site, None)
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
/// 3. `style::States`: the disabled and selected readers (no effect).
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
    mount_node(cx, kind, c, Some(at), fix_layout)
}

/// Mount host node `c`. `site` is the call site recorded on the node (`None` for a node the App
/// did not write, such as a string child); its effects are located there, or here without one.
fn mount_node(
    cx: &Mcx<'_>,
    kind: NodeType,
    c: Common,
    site: Option<&'static Location<'static>>,
    fix_layout: Option<fn(&mut LayoutProps)>,
) -> NodeId {
    // `.located_at_caller()` replaces the builder's call site.
    let site = c.at.or(site);
    let at = site.unwrap_or_else(Location::caller);
    let id = cx.host(kind);
    let Resolved {
        first,
        semantics,
        disabled,
        selected,
        style,
        variants,
        facts,
        draw,
        late_semantics,
        ime,
        refs,
        attached,
    } = resolve::resolve(cx, id, kind, c, at);
    create::write(cx, id, first, site, at);
    semantics::bind_props(cx, id, semantics, at);
    let change = [&disabled, &selected]
        .into_iter()
        .any(|prop| prop.as_ref().is_some_and(|p| p.fixed().is_none()));
    let states = style::States {
        change,
        disabled: disabled.map(InputProp::into_fn),
        selected: selected.map(InputProp::into_fn),
    };
    let input = style::StyleInput {
        layers: style,
        variants,
        fix_layout,
    };
    style::bind_node_style(cx, id, input, &facts, states, at);
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
