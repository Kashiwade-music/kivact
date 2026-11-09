//! Role: Ordered semantic patches publish one canonical value per node.

use crate::view::NodeRef;
use crate::view::descriptor::Mcx;
use crate::view::semantics_data::{SemanticInput, SemanticsPatch};
use crate::view::state::ViewState;
use kivact_engine::api::PropValue;
use kivact_runtime::{EffectKind, ErrorKind, Prop as InputProp, UiError};
use kivact_style::asset::ImageFit;
use kivact_types::{CheckedState, FocusTarget, NodeId, Semantics};
use std::cell::RefCell;
use std::panic::Location;

/// Fold declaration layers in order, retaining relation declaration order.
pub(super) fn resolve(patches: &[SemanticsPatch]) -> Semantics<FocusTarget> {
    let mut semantics = Semantics::default();
    let mut checked = None;
    let mut indeterminate = None;
    for patch in patches {
        (patch.apply)(&mut semantics);
        match &patch.input {
            SemanticInput::Checked(source) => checked = Some(source.get()),
            SemanticInput::Indeterminate(source) => indeterminate = Some(source.get()),
            _ => {}
        }
    }
    if checked.is_some() || indeterminate.is_some() {
        semantics.checked = checked_state(checked, indeterminate);
    }
    semantics
}

pub(super) fn checked_state(
    checked: Option<bool>,
    indeterminate: Option<bool>,
) -> Option<CheckedState> {
    match (checked, indeterminate) {
        (_, Some(true)) => Some(CheckedState::Indeterminate),
        (Some(checked), _) => Some(CheckedState::from_bool(checked)),
        _ => None,
    }
}

fn bind(
    cx: &Mcx<'_>,
    id: NodeId,
    mut patches: Vec<SemanticsPatch>,
    at: &'static Location<'static>,
) {
    if patches.is_empty() {
        return;
    }
    patches.sort_by_key(|p| p.layer);
    let reactive = patches.iter().any(|patch| patch.reactive);
    if let Some(node) = ViewState::tree_mut(cx.rt).get_mut(id) {
        node.semantic_origins = patches.iter().map(|p| p.at).collect();
    }
    let weak = cx.weak();
    let owner = cx.owner;
    let previous: RefCell<Option<Semantics<FocusTarget>>> = RefCell::new(None);
    let publish = move || {
        let mut props = resolve(&patches);
        if let Some(rt) = weak.upgrade() {
            if let Some(Err(why)) = props.text.as_ref().map(kivact_types::check_text) {
                props.text = None;
                let at = patches.iter().rev().find(|p| p.text).map_or(at, |p| p.at);
                rt.report(
                    Some(owner),
                    UiError::new(
                        ErrorKind::Unsupported,
                        format!("semantics_text is not published: {why}"),
                        Some(at),
                    ),
                );
            }
            if previous.borrow().as_ref() != Some(&props) {
                let old = previous.replace(Some(props.clone()));
                let writer = rt.running_effect();
                let mut tree = ViewState::tree_mut(&rt);
                // These facts also drive native input and layout, independently
                // of the canonical semantic payload.
                if old
                    .as_ref()
                    .is_none_or(|old| old.disabled != props.disabled)
                {
                    tree.update(id, PropValue::Disabled(props.disabled), writer);
                }
                if old.as_ref().is_none_or(|old| old.hidden != props.hidden) {
                    tree.update(id, PropValue::Hidden(props.hidden), writer);
                }
                if old
                    .as_ref()
                    .is_none_or(|old| old.focusable != props.focusable)
                {
                    tree.update(id, PropValue::Focusable(props.focusable), writer);
                }
                tree.update(id, PropValue::Semantics(props), writer);
            }
        }
    };
    if reactive {
        cx.effect(EffectKind::Render, at, publish);
    } else {
        publish();
    }
}

/// Text geometry is evaluated after the node's draw layers, in the same publisher.
pub(super) fn bind_late(
    cx: &Mcx<'_>,
    id: NodeId,
    patches: Vec<SemanticsPatch>,
    at: &'static Location<'static>,
) {
    bind(cx, id, patches, at);
}

/// Non-semantic host properties share the semantic mounting stage.
pub(super) struct SemanticProps {
    pub(super) patches: Vec<SemanticsPatch>,
    // xtask-allow(state-fields): mount config - image fit
    pub(super) image_fit: Option<InputProp<ImageFit>>,
    // xtask-allow(state-fields): mount config - monochrome
    pub(super) monochrome: Option<InputProp<bool>>,
    // xtask-allow(state-fields): mount config - image playing
    pub(super) image_playing: Option<InputProp<bool>>,
    // xtask-allow(state-fields): mount config - layout id
    pub(super) layout_id: Option<std::sync::Arc<str>>,
    // xtask-allow(state-fields): mount config - focus on press
    pub(super) focus_on_press: Option<NodeRef>,
}

pub(super) fn bind_props(
    cx: &Mcx<'_>,
    id: NodeId,
    props: SemanticProps,
    at: &'static Location<'static>,
) {
    bind(cx, id, props.patches, at);
    if let Some(source) = props.image_fit {
        cx.bind(id, source, PropValue::ImageFit, at);
    }
    if let Some(source) = props.monochrome {
        cx.bind(id, source, PropValue::Monochrome, at);
    }
    if let Some(source) = props.image_playing {
        cx.bind(id, source, PropValue::ImagePlaying, at);
    }
    if let Some(layout_id) = props.layout_id {
        // Written once at mount: the id is an attribute of the node.
        let writer = cx.rt.running_effect();
        ViewState::tree_mut(cx.rt).update(id, PropValue::LayoutId(Some(layout_id)), writer);
    }
    if let Some(destination) = props.focus_on_press {
        let weak = cx.weak();
        cx.effect(EffectKind::Render, at, move || {
            let target = destination.target_value(at);
            if let Some(rt) = weak.upgrade() {
                ViewState::tree_mut(&rt).update(
                    id,
                    PropValue::FocusOnPress(target),
                    rt.running_effect(),
                );
            }
        });
    }
}
