//! Role: Ordered semantic patches publish one canonical value per node.

use crate::view::NodeRef;
use crate::view::descriptor::Mcx;
use crate::view::semantics_data::{SemanticInput, SemanticsPatch};
use crate::view::state::ViewState;
use kivact_engine::api::PropValue;
use kivact_runtime::{EffectKind, ErrorKind, Prop as InputProp, UiError};
use kivact_style::asset::ImageFit;
use kivact_types::{CheckedState, FocusTarget, NodeId, Semantics};
use std::panic::Location;
use std::rc::Rc;

/// What a node's semantic publisher keeps after mounting (M-R5): the fold of
/// the patches before the first reactive one, and the patches from it on.
/// A node with only constant patches keeps nothing: it publishes once.
struct Plan {
    base: Semantics<FocusTarget>,
    checked: Option<bool>,
    indeterminate: Option<bool>,
    rest: Box<[Rest]>,
}

/// One patch from the first reactive one on, in layer order.
struct Rest {
    apply: Rc<dyn Fn(&mut Semantics<FocusTarget>)>,
    input: RestInput,
}

/// The preserved input a later patch publishes through the checked state.
enum RestInput {
    Other,
    Checked(InputProp<bool>),
    Indeterminate(InputProp<bool>),
}

impl Plan {
    /// `patches` sorted by layer.
    fn new(patches: Vec<SemanticsPatch>) -> Self {
        let split = patches
            .iter()
            .position(|p| p.reactive)
            .unwrap_or(patches.len());
        let mut base = Semantics::default();
        let (mut checked, mut indeterminate) = (None, None);
        let mut rest = Vec::with_capacity(patches.len() - split);
        for (index, patch) in patches.into_iter().enumerate() {
            if index < split {
                (patch.apply)(&mut base);
                match &patch.input {
                    SemanticInput::Checked(source) => checked = Some(source.get()),
                    SemanticInput::Indeterminate(source) => indeterminate = Some(source.get()),
                    _ => {}
                }
            } else {
                let input = match patch.input {
                    SemanticInput::Checked(source) => RestInput::Checked(source),
                    SemanticInput::Indeterminate(source) => RestInput::Indeterminate(source),
                    _ => RestInput::Other,
                };
                rest.push(Rest {
                    apply: patch.apply,
                    input,
                });
            }
        }
        Plan {
            base,
            checked,
            indeterminate,
            rest: rest.into_boxed_slice(),
        }
    }

    /// The canonical value: the base, then each later patch in order.
    fn resolve(&self) -> Semantics<FocusTarget> {
        let mut semantics = self.base.clone();
        let (mut checked, mut indeterminate) = (self.checked, self.indeterminate);
        for patch in self.rest.iter() {
            (patch.apply)(&mut semantics);
            match &patch.input {
                RestInput::Checked(source) => checked = Some(source.get()),
                RestInput::Indeterminate(source) => indeterminate = Some(source.get()),
                RestInput::Other => {}
            }
        }
        if checked.is_some() || indeterminate.is_some() {
            semantics.checked = checked_state(checked, indeterminate);
        }
        semantics
    }
}

/// Fold declaration layers in order, retaining relation declaration order.
pub(super) fn resolve(patches: &[SemanticsPatch]) -> Semantics<FocusTarget> {
    Plan::new(patches.to_vec()).resolve()
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
    if let Some(node) = ViewState::tree_mut(cx.rt).get_mut(id) {
        node.semantic_origins = patches.iter().map(|p| p.at).collect();
    }
    let text_at = patches.iter().rev().find(|p| p.text).map_or(at, |p| p.at);
    let reactive = patches.iter().any(|patch| patch.reactive);
    let plan = Plan::new(patches);
    let weak = cx.weak();
    let owner = cx.owner;
    let publish = move || {
        let mut props = plan.resolve();
        let Some(rt) = weak.upgrade() else { return };
        if let Some(Err(why)) = props.text.as_ref().map(kivact_types::check_text) {
            props.text = None;
            rt.report(
                Some(owner),
                UiError::new(
                    ErrorKind::Unsupported,
                    format!("semantics_text is not published: {why}"),
                    Some(text_at),
                ),
            );
        }
        let writer = rt.running_effect();
        let mut tree = ViewState::tree_mut(&rt);
        // These facts also drive native input and layout; the engine ignores equal values.
        tree.update(id, PropValue::Disabled(props.disabled), writer);
        tree.update(id, PropValue::Hidden(props.hidden), writer);
        tree.update(id, PropValue::Focusable(props.focusable), writer);
        tree.update(id, PropValue::Semantics(props), writer);
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
    // xtask-allow(state-fields): option: absent means the image's own fit
    pub(super) image_fit: Option<InputProp<ImageFit>>,
    // xtask-allow(state-fields): option: absent means the image's own colors
    pub(super) monochrome: Option<InputProp<bool>>,
    // xtask-allow(state-fields): option: absent means the image plays
    pub(super) image_playing: Option<InputProp<bool>>,
    // xtask-allow(state-fields): option: absent means no shared layout
    pub(super) layout_id: Option<std::sync::Arc<str>>,
    // xtask-allow(state-fields): option: absent means the press does not focus
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
