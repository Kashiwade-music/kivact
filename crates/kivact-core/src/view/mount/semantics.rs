//! Role: Semantic sources publish reactive accessibility properties.

use std::panic::Location;

use crate::view::common::{Common, SemanticSources};
use crate::view::descriptor::Mcx;
use crate::view::state::ViewState;
use kivact_engine::bridge::PropValue;
use kivact_reactive::Prop as InputProp;
use kivact_reactive::{EffectKind, ErrorKind, UiError};
use kivact_wire::NodeId;

use kivact_wire::SemanticsProperties;
/// The nodes `refs` are bound to now, in order, leaving out a reference
/// that is unbound or whose node is gone (tracked: the relation follows
/// them being bound and unmounted).
fn bound(
    refs: &[crate::view::NodeRef],
    at: &'static std::panic::Location<'static>,
) -> Vec<kivact_wire::FocusTarget> {
    refs.iter().filter_map(|r| r.target_value(at)).collect()
}
pub(super) fn bind(
    cx: &Mcx<'_>,
    id: kivact_wire::NodeId,
    mut sources: SemanticSources,
    at: &'static std::panic::Location<'static>,
) {
    if let Some(source) = sources.focusable.take() {
        cx.bind(id, source, PropValue::Focusable, at);
    }
    if sources.indeterminate.is_none()
        && sources.value_min.is_none()
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
            if let Some(Err(why)) = props.text.as_ref().map(kivact_wire::check_text) {
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
            ViewState::tree_mut(&rt).update(
                id,
                PropValue::Accessibility(props),
                rt.running_effect(),
            );
        }
    });
}

/// Stage 8: bind the semantic sources that include the node's own text, in
/// the existing effect order after draw layers for retained paint consumers.
/// Text geometry may instead use `primitive::shape_paragraph` directly. A value that
/// reads what the drawing just made sees it in the same turn.
pub(super) fn bind_late(
    cx: &Mcx<'_>,
    id: kivact_wire::NodeId,
    c: &mut Common,
    at: &'static std::panic::Location<'static>,
) {
    if let Some(s) = c.semantics.take() {
        bind(cx, id, *s, at);
    }
}

/// Resolve shared semantic sources in the current tracked rendering context.
pub(super) fn resolve(
    sources: &SemanticSources,
    at: &'static std::panic::Location<'static>,
) -> SemanticsProperties {
    SemanticsProperties {
        indeterminate: sources
            .indeterminate
            .as_ref()
            .map(kivact_reactive::Prop::get)
            .unwrap_or(false),
        value_min: sources.value_min.as_ref().map(kivact_reactive::Prop::get),
        value_max: sources.value_max.as_ref().map(kivact_reactive::Prop::get),
        value_now: sources.value_now.as_ref().map(kivact_reactive::Prop::get),
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
            .and_then(kivact_reactive::Prop::get)
            .and_then(|r| r.target_value(at)),
        level: sources.level.as_ref().map(kivact_reactive::Prop::get),
        set_size: sources.set_size.as_ref().map(kivact_reactive::Prop::get),
        pos_in_set: sources.pos_in_set.as_ref().map(kivact_reactive::Prop::get),
        multiselectable: sources
            .multiselectable
            .as_ref()
            .map(kivact_reactive::Prop::get),
        described_by: bound(&sources.described_by, at),
        controls: bound(&sources.controls, at),
        labelled_by: bound(&sources.labelled_by, at),
        text: sources.text.as_ref().map(kivact_reactive::Prop::get),
        ..Default::default()
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

/// Stage 4: bind the semantic properties (name, description, value, checked,
/// expanded, the extra semantic sources, the press focus destination,
/// invalid, hidden) and the image properties (fit, monochrome).
pub(super) fn bind_props(cx: &Mcx<'_>, id: NodeId, c: &mut Common, at: &'static Location<'static>) {
    bind_prop(cx, id, c.name.take(), PropValue::Name, at);
    bind_prop(cx, id, c.description.take(), PropValue::Description, at);
    bind_prop(cx, id, c.value.take(), PropValue::Value, at);
    bind_prop(
        cx,
        id,
        c.checked.take(),
        |b| PropValue::Checked(Some(b)),
        at,
    );
    bind_prop(
        cx,
        id,
        c.expanded.take(),
        |b| PropValue::Expanded(Some(b)),
        at,
    );
    // Sources with the node's own text wait for the draw stage
    // (`bind_late`) preserves ordering for retained paint consumers. Text
    // geometry can shape directly and does not require a preceding draw.
    if c.semantics.as_ref().is_some_and(|s| s.text.is_none())
        && let Some(s) = c.semantics.take()
    {
        bind(cx, id, *s, at);
    }
    if let Some(destination) = c.focus_on_press.take() {
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
    bind_prop(cx, id, c.invalid.take(), PropValue::Invalid, at);
    bind_prop(cx, id, c.hidden.take(), PropValue::Hidden, at);
    bind_prop(cx, id, c.image_fit.take(), PropValue::ImageFit, at);
    bind_prop(cx, id, c.monochrome.take(), PropValue::Monochrome, at);
}
