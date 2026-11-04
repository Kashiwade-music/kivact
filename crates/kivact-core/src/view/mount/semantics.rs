//! Role: Semantic sources publish reactive accessibility properties.

use std::panic::Location;

use crate::view::NodeRef;
use crate::view::common::SemanticSources;
use crate::view::descriptor::Mcx;
use crate::view::state::ViewState;
use kivact_engine::api::PropValue;
use kivact_runtime::Prop as InputProp;
use kivact_runtime::{EffectKind, ErrorKind, UiError};
use kivact_style::asset::ImageFit;
use kivact_types::{CheckedState, NodeId};

use kivact_types::{Finite, FocusTarget, Semantics};
/// The nodes `refs` are bound to now, in order, leaving out a reference
/// that is unbound or whose node is gone (tracked: the relation follows
/// them being bound and unmounted).
fn bound(
    refs: &[crate::view::NodeRef],
    at: &'static std::panic::Location<'static>,
) -> Vec<kivact_types::FocusTarget> {
    refs.iter().filter_map(|r| r.target_value(at)).collect()
}
pub(super) fn bind(
    cx: &Mcx<'_>,
    id: kivact_types::NodeId,
    sources: SemanticSources,
    at: &'static std::panic::Location<'static>,
) {
    if sources.value_min.is_none()
        && sources.value_max.is_none()
        && sources.value_now.is_none()
        && sources.live_region.is_none()
        && sources.sort.is_none()
        && sources.orientation.is_none()
        && sources.active_descendant.is_none()
        && sources.level.is_none()
        && sources.set_size.is_none()
        && sources.pos_in_set.is_none()
        && sources.multiselectable.is_none()
        && sources.busy.is_none()
        && sources.required.is_none()
        && sources.read_only.is_none()
        && sources.has_popup.is_none()
        && sources.current_item.is_none()
        && sources.described_by.is_empty()
        && sources.controls.is_empty()
        && sources.labelled_by.is_empty()
        && sources.text.is_none()
    {
        return;
    }
    let weak = cx.weak();
    let owner = cx.owner;
    cx.effect(EffectKind::Render, at, move || {
        let mut props = resolve(&sources, at);
        if let Some(rt) = weak.upgrade() {
            // A text the tree cannot publish is reported, not published.
            if let Some(Err(why)) = props.text.as_ref().map(kivact_types::check_text) {
                props.text = None;
                rt.report(
                    Some(owner),
                    UiError::new(
                        ErrorKind::Unsupported,
                        format!("semantics_text is not published: {why}"),
                        Some(at),
                    ),
                );
            }
            let mut current = ViewState::tree(&rt)
                .reader()
                .semantics(id)
                .unwrap_or_default();
            current.value_min = props.value_min;
            current.value_max = props.value_max;
            current.value_now = props.value_now;
            current.live_region = props.live_region;
            current.sort = props.sort;
            current.orientation = props.orientation;
            current.active_descendant = props.active_descendant;
            current.level = props.level;
            current.set_size = props.set_size;
            current.pos_in_set = props.pos_in_set;
            current.multiselectable = props.multiselectable;
            current.busy = props.busy;
            current.required = props.required;
            current.read_only = props.read_only;
            current.has_popup = props.has_popup;
            current.current_item = props.current_item;
            current.described_by = props.described_by;
            current.controls = props.controls;
            current.labelled_by = props.labelled_by;
            current.text = props.text;
            ViewState::tree_mut(&rt).update(id, PropValue::Semantics(current), rt.running_effect());
        }
    });
}

/// Stage 6: bind the semantic sources that include the node's own text, in
/// the existing effect order after draw layers for retained paint consumers.
/// Text geometry may instead use `custom::shape_paragraph` directly. A value that
/// reads what the drawing just made sees it in the same turn.
pub(super) fn bind_late(
    cx: &Mcx<'_>,
    id: kivact_types::NodeId,
    sources: Option<Box<SemanticSources>>,
    at: &'static std::panic::Location<'static>,
) {
    if let Some(sources) = sources {
        bind(cx, id, *sources, at);
    }
}

/// The published checked state of a node or region: mixed wins, else the
/// binary value, else not a toggle.
pub(super) fn checked_state(
    checked: Option<bool>,
    indeterminate: Option<bool>,
) -> Option<CheckedState> {
    match (checked, indeterminate) {
        (_, Some(true)) => Some(CheckedState::Indeterminate),
        (Some(checked), _) => Some(CheckedState::from_bool(checked)),
        (None, _) => None,
    }
}

/// Resolve shared semantic sources in the current tracked rendering context.
pub(super) fn resolve(
    sources: &SemanticSources,
    at: &'static std::panic::Location<'static>,
) -> Semantics<FocusTarget> {
    Semantics {
        value_min: sources
            .value_min
            .as_ref()
            .map(kivact_runtime::Prop::get)
            .and_then(Finite::new),
        value_max: sources
            .value_max
            .as_ref()
            .map(kivact_runtime::Prop::get)
            .and_then(Finite::new),
        value_now: sources
            .value_now
            .as_ref()
            .map(kivact_runtime::Prop::get)
            .and_then(Finite::new),
        live_region: sources.live_region.as_ref().map(|s| s.get()),
        sort: sources.sort.as_ref().map(|s| s.get()).unwrap_or_default(),
        orientation: sources
            .orientation
            .as_ref()
            .map(|s| s.get())
            .unwrap_or_default(),
        active_descendant: sources
            .active_descendant
            .as_ref()
            .and_then(kivact_runtime::Prop::get)
            .and_then(|r| r.target_value(at)),
        level: sources.level.as_ref().map(kivact_runtime::Prop::get),
        set_size: sources.set_size.as_ref().map(kivact_runtime::Prop::get),
        pos_in_set: sources.pos_in_set.as_ref().map(kivact_runtime::Prop::get),
        multiselectable: sources
            .multiselectable
            .as_ref()
            .map(kivact_runtime::Prop::get),
        busy: sources.busy.as_ref().map(kivact_runtime::Prop::get),
        required: sources.required.as_ref().map(kivact_runtime::Prop::get),
        read_only: sources.read_only.as_ref().map(kivact_runtime::Prop::get),
        has_popup: sources
            .has_popup
            .as_ref()
            .and_then(kivact_runtime::Prop::get),
        current_item: sources
            .current_item
            .as_ref()
            .and_then(kivact_runtime::Prop::get),
        described_by: bound(&sources.described_by, at),
        controls: bound(&sources.controls, at),
        labelled_by: bound(&sources.labelled_by, at),
        text: sources.text.as_ref().map(kivact_runtime::Prop::get),
        ..Semantics::default()
    }
}

/// Bind the optional source `source` to `prop`: a constant is written once,
/// a reactive value by one render effect, and no source writes nothing.
fn bind_prop<T: Clone + 'static>(
    cx: &Mcx<'_>,
    id: NodeId,
    source: Option<InputProp<T>>,
    value: fn(T) -> PropValue,
    at: &'static Location<'static>,
) {
    if let Some(source) = source {
        cx.bind(id, source, value, at);
    }
}

/// Stage 2's input: the semantic and image properties.
pub(super) struct SemanticProps {
    pub(super) name: Option<InputProp<String>>,
    pub(super) description: Option<InputProp<String>>,
    pub(super) value: Option<InputProp<String>>,
    pub(super) checked: Option<InputProp<bool>>,
    /// Mixed checked state; wins over `checked`.
    pub(super) indeterminate: Option<InputProp<bool>>,
    pub(super) expanded: Option<InputProp<bool>>,
    pub(super) invalid: Option<InputProp<bool>>,
    /// Final: the caller's, or combined with the parts' requirements.
    pub(super) hidden: Option<InputProp<bool>>,
    pub(super) image_fit: Option<InputProp<ImageFit>>,
    pub(super) monochrome: Option<InputProp<bool>>,
    pub(super) focus_on_press: Option<NodeRef>,
    /// The extra semantic sources without the node's own text (those wait for `bind_late`).
    pub(super) sources: Option<Box<SemanticSources>>,
}

/// Stage 2: bind the semantic properties (name, description, value, checked,
/// expanded, the extra semantic sources, the press focus destination,
/// invalid, hidden) and the image properties (fit, monochrome).
pub(super) fn bind_props(
    cx: &Mcx<'_>,
    id: NodeId,
    props: SemanticProps,
    at: &'static Location<'static>,
) {
    let SemanticProps {
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
        sources,
    } = props;
    bind_prop(cx, id, name, PropValue::Name, at);
    bind_prop(cx, id, description, PropValue::Description, at);
    bind_prop(cx, id, value, PropValue::Value, at);
    if checked.is_some() || indeterminate.is_some() {
        let weak = cx.weak();
        cx.effect(EffectKind::Render, at, move || {
            let state = checked_state(
                checked.as_ref().map(kivact_runtime::Prop::get),
                indeterminate.as_ref().map(kivact_runtime::Prop::get),
            );
            if let Some(rt) = weak.upgrade() {
                ViewState::tree_mut(&rt).update(id, PropValue::Checked(state), rt.running_effect());
            }
        });
    }
    bind_prop(cx, id, expanded, PropValue::Expanded, at);
    if let Some(sources) = sources {
        bind(cx, id, *sources, at);
    }
    if let Some(destination) = focus_on_press {
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
    bind_prop(cx, id, invalid, PropValue::Invalid, at);
    bind_prop(cx, id, hidden, PropValue::Hidden, at);
    bind_prop(cx, id, image_fit, PropValue::ImageFit, at);
    bind_prop(cx, id, monochrome, PropValue::Monochrome, at);
}
