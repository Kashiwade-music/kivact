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
use crate::view::descriptor::Mcx;
use crate::view::editing_data::TextEditing;
use crate::view::element_data::{Common, StyleLayer};
use crate::view::ime_session::ImeSession;
use crate::view::node_ref::NodeRef;

// xtask-allow(state-fields): Independent mounting stage inputs are consumed by their respective publishers.
/// One host node's stage inputs, each moved into exactly one stage.
pub(super) struct Resolved {
    pub(super) first: FirstWrite,
    pub(super) semantics: SemanticProps,
    pub(super) disabled: Option<InputProp<bool>>,
    pub(super) selected: Option<InputProp<bool>>,
    pub(super) style: Vec<StyleLayer>,
    pub(super) facts: NodeFacts,
    pub(super) draw: Option<DrawInput>,
    pub(super) late_semantics: Vec<crate::view::semantics_data::SemanticsPatch>,
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
    let mut common = apply_deferred(common);
    prepare_constraints(&mut common, cx, at);
    let editing = common.text_editing.take().map(|editing| {
        let TextEditing {
            props,
            on_edit,
            on_selection,
            on_phase,
        } = *editing;
        common.handlers.edit_phase = on_phase;
        common.handlers.text = on_edit;
        common.handlers.selection = on_selection;
        props
    });
    let attached = attach_parts(&mut common, cx, id, at);
    let focus_stop = common
        .focusable_source()
        .as_ref()
        .is_some_and(|f| f.fixed() != Some(false))
        || editing.is_some();
    let local_paint = (!common.style.is_empty() && kind == NodeType::Box && editing.is_none())
        .then(|| LocalPaintSource {
            value: Rc::new(RefCell::new(None)),
            revision: cx.owner.notifier(at),
        });
    let draw = prepare_draw(&mut common, cx, attached.as_ref(), local_paint.clone(), at);
    finish_resolve(
        common,
        kind,
        editing,
        attached,
        draw,
        NodeFacts {
            kind,
            focus_stop,
            local_paint,
        },
    )
}

fn apply_deferred(common: Common) -> Common {
    let mut node =
        crate::view::element_data::Node::new(crate::view::element_data::Content::Box, None);
    node.common = common;
    while !node.common.deferred_parts.is_empty() {
        let deferred = std::mem::take(&mut node.common.deferred_parts);
        for apply in deferred {
            node = node.apply_part(apply);
        }
    }
    node.common
}

fn prepare_constraints(common: &mut Common, cx: &Mcx<'_>, at: &'static Location<'static>) {
    for patch in common.semantics.iter().filter(|p| p.part) {
        use crate::view::semantics_data::SemanticInput;
        match &patch.input {
            SemanticInput::Disabled(v) => common.part_constraints.disabled.push(v.clone()),
            SemanticInput::Hidden(v) => common.part_constraints.hidden.push(v.clone()),
            SemanticInput::Focusable(v) => common.part_constraints.focusable.push(v.clone()),
            _ => {}
        }
    }
    if common.part_modal {
        parts::report_modal_unsupported(cx, at);
    }
    let focus_stop = common
        .focusable_source()
        .as_ref()
        .is_some_and(|f| f.fixed() != Some(false))
        || common.text_editing.is_some();
    if common.handlers.activate.is_some()
        && common.role_source().is_none()
        && !focus_stop
        && common.part_constraints.focusable.is_empty()
    {
        diagnostics::report_unreachable_press(cx, at);
    }
}

fn attach_parts(
    common: &mut Common,
    cx: &Mcx<'_>,
    id: NodeId,
    at: &'static Location<'static>,
) -> Option<Attached> {
    let mut actions = common.handlers.actions();
    for action in common.part_actions.actions() {
        if !actions.contains(&action) {
            actions.push(action);
        }
    }
    let attached = if common.part_bindings.is_empty()
        && common.regions.is_empty()
        && common.part_constraints.disabled.is_empty()
        && common.part_constraints.hidden.is_empty()
        && common.part_constraints.focusable.is_empty()
    {
        None
    } else {
        let sources = ConstraintSources {
            disabled: common.disabled_source(),
            hidden: common.hidden_source(),
            focusable: common.focusable_source(),
            required: std::mem::take(&mut common.part_constraints),
        };
        let constraints = parts::constraints(sources, cx, at, None);
        let root = part_input::root_target(
            &common.handlers,
            std::mem::take(&mut common.part_actions),
            constraints,
            cx,
        );
        let registry = regions::prepare(std::mem::take(&mut common.regions), root, cx, id, at);
        common.handlers =
            part_input::route(id, std::mem::take(&mut common.handlers), registry.clone());
        Some(Attached {
            constraints,
            bindings: std::mem::take(&mut common.part_bindings),
            regions: registry,
        })
    };
    let mut final_semantics = Common::default();
    if let Some(attached) = &attached {
        final_semantics.set_disabled(Prop::value(attached.constraints.disabled));
        final_semantics.set_hidden(Prop::value(attached.constraints.hidden));
        final_semantics.set_focusable(Prop::value(attached.constraints.focusable));
    }
    if !actions.is_empty() {
        final_semantics.semantic(Prop::constant(actions), |s, value| s.actions = Some(value));
    }
    let next = common.semantics.last().map_or(0, |p| p.layer + 1);
    for patch in &mut final_semantics.semantics {
        patch.layer += next;
    }
    common.semantics.append(&mut final_semantics.semantics);
    attached
}

fn prepare_draw(
    common: &mut Common,
    cx: &Mcx<'_>,
    attached: Option<&Attached>,
    local_paint: Option<LocalPaintSource>,
    at: &'static Location<'static>,
) -> Option<DrawInput> {
    if common.draw_with_layout.is_some() && common.handlers.selection.is_none() {
        common.handlers.selection = Some(kivact_runtime::Handler::from_fn(
            |_: kivact_editing::TextSession| {},
        ));
    }
    let observe_layout =
        common.draw.is_some() || common.draw_over.is_some() || common.draw_with_layout.is_some();
    (observe_layout || local_paint.is_some()).then(|| {
        let size = cx.owner.signal(Size::new(0.0, 0.0), at);
        common.handlers.observe_layout(move |_, s| size.set(s));
        DrawInput {
            below: common.draw.take(),
            over: common.draw_over.take(),
            with_layout: common.draw_with_layout.take(),
            local_paint,
            regions: attached.map(|a| a.regions.clone()),
            size,
        }
    })
}

fn finish_resolve(
    common: Common,
    _kind: NodeType,
    editing: Option<crate::view::editing_data::EditorProps>,
    attached: Option<Attached>,
    draw: Option<DrawInput>,
    facts: NodeFacts,
) -> Resolved {
    let disabled = common.disabled_source();
    let selected = common.selected_source();
    let Common {
        deferred_parts: _,
        exit_layout,
        exit_complete,
        node_ref,
        style,
        base_styles: _,
        semantics,
        focusable_when_disabled,
        roving_focus,
        hit_testable,
        focus_on_press,
        handlers,
        text_editing: _,
        ime,
        part_constraints: _,
        part_actions: _,
        part_region_events: _,
        part_bindings: _,
        part_modal: _,
        regions: _,
        draw: _,
        draw_over: _,
        draw_with_layout: _,
        intrinsic,
        image_fit,
        monochrome,
        at: _,
    } = common;
    let (early, late) = if semantics.iter().any(|p| p.text) {
        (Vec::new(), semantics)
    } else {
        (semantics, Vec::new())
    };
    let observe_layout = draw.as_ref().is_some_and(|draw| {
        draw.below.is_some() || draw.over.is_some() || draw.with_layout.is_some()
    });
    Resolved {
        first: FirstWrite {
            exit_layout: exit_layout.unwrap_or_default(),
            exit_complete,
            focusable_when_disabled,
            roving_focus,
            hit_testable,
            handlers,
            intrinsic,
            observe_layout,
            editing,
        },
        semantics: SemanticProps {
            patches: early,
            image_fit,
            monochrome,
            focus_on_press,
        },
        disabled,
        selected,
        style,
        facts,
        draw,
        late_semantics: late,
        ime,
        refs: node_ref,
        attached,
    }
}
