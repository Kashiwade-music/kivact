//! Role: Mounting resolves a host node's declared modifiers into the input of each stage.
//!
//! `Common` is what the modifiers declare. `resolve` consumes it once and destructures it
//! exhaustively (a new field is a compile error until it is handled): everything that changes what
//! a later stage sees is decided here (the parts' constraints, the routing of the handlers, the
//! composition of the edit callbacks, the draw-size observer), and each stage then only binds its
//! own input, moved into it by value.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::contract::NodeType;
use kivact_runtime::Prop;
use kivact_runtime::Prop as InputProp;
use kivact_style::draw::Size;
use kivact_types::NodeId;

use super::attachments::Attached;
use super::create::FirstWrite;
use super::draw::DrawInput;
use super::facts::{LocalPaintSource, NodeFacts};
use super::parts::ConstraintSources;
use super::semantics::SemanticProps;
use super::{diagnostics, part_input, parts, regions};
use crate::view::common::{Common, SemanticSources, StyleLayer};
use crate::view::descriptor::Mcx;
use crate::view::editing_data::TextEditing;
use crate::view::ime_session::ImeSession;
use crate::view::node_ref::NodeRef;

/// One host node's stage inputs, each moved into exactly one stage.
pub(super) struct Resolved {
    pub(super) first: FirstWrite,
    pub(super) semantics: SemanticProps,
    pub(super) disabled: Option<InputProp<bool>>,
    pub(super) selected: Option<InputProp<bool>>,
    pub(super) style: Vec<StyleLayer>,
    pub(super) facts: NodeFacts,
    pub(super) draw: Option<DrawInput>,
    pub(super) late_semantics: Option<Box<SemanticSources>>,
    pub(super) ime: Option<ImeSession>,
    pub(super) refs: Vec<NodeRef>,
    pub(super) attached: Option<Attached>,
}

/// Resolve the declared `common` of host node `id` (of kind `kind`) into the stage inputs.
pub(super) fn resolve(
    cx: &Mcx<'_>,
    id: NodeId,
    kind: NodeType,
    common: Common,
    at: &'static Location<'static>,
) -> Resolved {
    let common = {
        let mut node = crate::view::node::Node::new(crate::view::node::Content::Box, None);
        node.common = common;
        while !node.common.deferred_parts.is_empty() {
            let deferred = std::mem::take(&mut node.common.deferred_parts);
            for apply in deferred {
                node = apply(node).resolve_modifiers();
            }
        }
        node.common
    };
    let Common {
        deferred_parts: _,
        exit_layout,
        exit_complete,
        test_id,
        node_ref,
        style,
        base_styles: _,
        role,
        name,
        description,
        value,
        selected,
        disabled,
        hidden,
        invalid,
        semantics,
        checked,
        indeterminate,
        expanded,
        focusable,
        focusable_when_disabled,
        roving_focus,
        hit_testable,
        focus_on_press,
        mut handlers,
        text_editing,
        ime,
        part_constraints,
        part_actions,
        part_region_events: _,
        part_bindings,
        part_modal,
        regions: declarations,
        draw: below,
        draw_over: over,
        intrinsic,
        image_fit,
        monochrome,
        at: _,
    } = common;
    if part_modal {
        parts::report_modal_unsupported(cx, at);
    }
    let focus_stop_declared =
        focusable.as_ref().is_some_and(|f| f.fixed() != Some(false)) || text_editing.is_some();
    if handlers.activate.is_some()
        && role.is_none()
        && !focus_stop_declared
        && part_constraints.focusable.is_empty()
    {
        diagnostics::report_unreachable_press(cx, at);
    }
    // The node's own edit and session slots; no modifier writes them before mount.
    let editing = text_editing.map(|editing| {
        let TextEditing {
            props,
            on_edit,
            on_selection,
            on_phase,
        } = *editing;
        handlers.edit_phase = on_phase;
        handlers.text = on_edit;
        handlers.selection = on_selection;
        props
    });
    let mut actions = handlers.actions();
    for action in part_actions.actions() {
        if !actions.contains(&action) {
            actions.push(action);
        }
    }
    let (attached, disabled, hidden, focusable) =
        if part_bindings.is_empty() && declarations.is_empty() {
            (None, disabled, hidden, focusable)
        } else {
            let sources = ConstraintSources {
                disabled,
                hidden,
                focusable,
                required: part_constraints,
            };
            let constraints = parts::constraints(sources, cx, at, None);
            let root = part_input::root_target(&handlers, part_actions, constraints, cx);
            let registry = regions::prepare(declarations, root, cx, id, at);
            handlers = part_input::route(id, handlers, registry.clone());
            (
                Some(Attached {
                    constraints,
                    bindings: part_bindings,
                    regions: registry,
                }),
                Some(Prop::value(constraints.disabled)),
                Some(Prop::value(constraints.hidden)),
                Some(Prop::value(constraints.focusable)),
            )
        };
    // A focus stop gets the theme's focus ring and disabled look (`kivact_style::internal::compile_style`).
    let focus_stop =
        focusable.as_ref().is_some_and(|f| f.fixed() != Some(false)) || editing.is_some();
    let local_paint =
        (!style.is_empty() && kind == NodeType::Box && editing.is_none()).then(|| {
            LocalPaintSource {
                value: Rc::new(RefCell::new(None)),
                revision: cx.owner.notifier(at),
            }
        });
    let observe_layout = below.is_some() || over.is_some();
    let draw = (observe_layout || local_paint.is_some()).then(|| {
        let size = cx.owner.signal(Size::new(0.0, 0.0), at);
        handlers.observe_layout(move |_, s| size.set(s));
        DrawInput {
            below,
            over,
            local_paint: local_paint.clone(),
            regions: attached.as_ref().map(|a| a.regions.clone()),
            size,
        }
    });
    let (early, late) = match semantics {
        Some(s) if s.text.is_some() => (None, Some(s)),
        s => (s, None),
    };
    Resolved {
        first: FirstWrite {
            exit_layout: exit_layout.unwrap_or_default(),
            exit_complete,
            test_id,
            role,
            focusable,
            focusable_when_disabled,
            roving_focus,
            hit_testable,
            actions,
            handlers,
            intrinsic,
            observe_layout,
            editing,
        },
        semantics: SemanticProps {
            name,
            description,
            value,
            checked,
            indeterminate,
            expanded,
            invalid,
            hidden,
            image_fit,
            monochrome,
            focus_on_press,
            sources: early,
        },
        disabled,
        selected,
        style,
        facts: NodeFacts {
            kind,
            focus_stop,
            local_paint,
        },
        draw,
        late_semantics: late,
        ime,
        refs: node_ref,
        attached,
    }
}
