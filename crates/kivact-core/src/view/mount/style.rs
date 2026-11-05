//! Role: Mounting binds the style, disabled and selected states of a host node.

use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;
use kivact_runtime::Prop as InputProp;
use kivact_runtime::{EffectKind, UiError};
use kivact_style::{Interactive, Style};
use kivact_types::{LayoutProps, NodeId};

use super::facts::{LocalPaintSource, NodeFacts};
use crate::view::descriptor::Mcx;
use crate::view::element_data::StyleLayer;
use crate::view::state::ViewState;

/// Merge `layers` in order over the empty style.
pub(crate) fn merge_layers(layers: &[StyleLayer]) -> Style {
    layers
        .iter()
        .fold(Style::default(), |s, layer| s.merge(&layer()))
}

// xtask-allow(state-fields): Independent disabled and selected readers feed style compilation without owning their state.
/// The readers of the disabled and selected states, for the style effect.
pub(super) struct States {
    disabled: Option<Rc<dyn Fn() -> bool>>,
    selected: Option<Rc<dyn Fn() -> bool>>,
}

/// Stage 3: retain the semantic inputs read by the state style effect.
/// The semantic publisher owns the corresponding engine writes.
pub(super) fn bind_states(
    cx: &Mcx<'_>,
    id: NodeId,
    disabled: Option<InputProp<bool>>,
    selected: Option<InputProp<bool>>,
    at: &'static Location<'static>,
) -> States {
    let _ = (cx, id, at);
    let disabled = disabled.map(InputProp::into_fn);
    let selected = selected.map(InputProp::into_fn);
    States { disabled, selected }
}

/// Stage 4: bind the style effect when the node has style layers, owns part
/// of its layout, or is a focus stop (which gets the theme's focus ring).
pub(super) fn bind_node_style(
    cx: &Mcx<'_>,
    id: NodeId,
    layers: Vec<StyleLayer>,
    facts: &NodeFacts,
    states: States,
    fix_layout: Option<fn(&mut LayoutProps)>,
    at: &'static Location<'static>,
) {
    if !layers.is_empty() || fix_layout.is_some() || facts.focus_stop {
        let style = NodeStyle {
            layers,
            focusable: facts.focus_stop,
            disabled: states.disabled,
            selected: states.selected,
            fix_layout,
            local_paint: facts.local_paint.clone(),
        };
        bind_style(cx, id, facts.kind, style, at);
    }
}

// xtask-allow(state-fields): Independent style source readers and optional layout/paint hooks are mount configuration.
/// What the style of a host node is resolved from (see [`bind_style`]).
#[derive(Default)]
pub(crate) struct NodeStyle {
    /// Style layers, merged in order.
    pub layers: Vec<StyleLayer>,
    /// The node is a focus stop: the theme's focus ring and disabled look
    /// apply under its style.
    pub focusable: bool,
    /// Reader of the disabled state (applies the `disabled` state style).
    pub disabled: Option<Rc<dyn Fn() -> bool>>,
    /// Reader of the selected state (applies the `selected` state style).
    pub selected: Option<Rc<dyn Fn() -> bool>>,
    /// Applied to every resolved layout before it is set (a builder that
    /// owns part of its node's layout).
    pub fix_layout: Option<fn(&mut LayoutProps)>,
    /// The merged local paint read by the existing drawing effect.
    pub local_paint: Option<LocalPaintSource>,
}

/// Resolve the style of node `id` (the one style path of every node):
/// one effect merges the layers over the theme visible from the mount
/// owner, applies the disabled/selected state styles, resolves and sets
/// Layout, Paint, Text and the host-evaluated state styles. A paint
/// transition is resolved with the target, then evaluated by the host.
pub(crate) fn bind_style(
    cx: &Mcx<'_>,
    id: NodeId,
    kind: NodeType,
    style: NodeStyle,
    at: &'static Location<'static>,
) {
    let NodeStyle {
        layers,
        focusable,
        disabled,
        selected,
        fix_layout,
        local_paint,
    } = style;
    let owner = cx.owner;
    let reports = StyleReports::default();
    let theme = cx.theme();
    let weak = cx.weak();
    cx.effect(EffectKind::Render, at, move || {
        let Some(theme_sig) = theme else { return };
        let Some(rt) = weak.upgrade() else { return };
        // The style closures only compute values: pure even when this is
        // an Internal effect feeding a paint transition.
        let resolved = kivact_runtime::pure(|| {
            theme_sig.with(|theme| {
                let s = merge_layers(&layers);
                let state = Interactive {
                    focusable,
                    disabled: disabled.as_ref().is_some_and(|f| f()),
                    selected: selected.as_ref().is_some_and(|f| f()),
                    root: kind == NodeType::Root,
                };
                kivact_style::internal::compile_style(&s, theme, state)
            })
        });
        let has_paint = resolved.has_retained_paint();
        if has_paint && local_paint.is_none() && reports.leaf_paint.note() {
            rt.report(
                Some(owner),
                UiError::new(
                    kivact_runtime::ErrorKind::Unsupported,
                    "local paint requires a box node; apply it to the containing node",
                    Some(at),
                ),
            );
        }
        if let Some(source) = &local_paint {
            // A style can acquire paint later; observe only when it first does.
            // Never disable observation needed by a NodeRef or another consumer.
            if has_paint {
                ViewState::tree_mut(&rt).set_observe(id, true);
            }
            let changed = source
                .value
                .borrow()
                .as_ref()
                .map_or(has_paint, |old| !old.same_retained_paint(&resolved));
            if changed {
                *source.value.borrow_mut() = has_paint.then(|| resolved.clone());
                source.revision.notify_from_render();
            }
        }
        let mut layout = resolved.layout.clone();
        if let Some(fix) = fix_layout {
            fix(&mut layout);
        }
        let writer = rt.running_effect();
        let mut tree = ViewState::tree_mut(&rt);
        tree.update(id, PropValue::Layout(layout), writer);
        tree.update(id, PropValue::Paint(resolved.paint.clone()), writer);
        tree.update(id, PropValue::TextStyle(resolved.text.clone()), writer);
        tree.update(
            id,
            PropValue::States(resolved.states.clone().unwrap_or_default()),
            writer,
        );
        drop(tree);
        reports.report(&rt, owner, id, &resolved, at);
    });
}

#[derive(Default)]
struct StyleReports {
    content: kivact_types::cell::ReportOnce,
    text: kivact_types::cell::ReportOnce,
    states: kivact_types::cell::ReportOnce,
    leaf_paint: kivact_types::cell::ReportOnce,
}
impl StyleReports {
    fn report(
        &self,
        rt: &kivact_runtime::Runtime,
        owner: kivact_runtime::Owner,
        id: NodeId,
        resolved: &kivact_style::CompiledStyle,
        at: &'static Location<'static>,
    ) {
        if !resolved.unsupported_in_states.is_empty() && self.states.note() {
            rt.report(
                Some(owner),
                UiError::new(
                    kivact_runtime::ErrorKind::Unsupported,
                    format!(
                        "state styles set properties their state cannot change, ignored: {}; a hover, pressed, focus,                          focus_visible, group_hover, group_focus, enter or exit style changes paint only (background,                          border, ring, radius, opacity, shadow, transform, z_index, scrollbar, cursor, filters, mask,                          clip_path, color), and a paint callback belongs on the base style",
                        resolved.unsupported_in_states.join(", ")
                    ),
                    Some(at),
                ),
            );
        }
        if resolved.duplicates > 0 && cfg!(debug_assertions) {
            rt.note(format!(
                "style of node {} sets {} properties twice (last wins)",
                id.0, resolved.duplicates
            ));
        }
        if resolved.misplaced_content > 0 && self.content.note() {
            rt.report(
                Some(owner),
                UiError::new(
                    kivact_runtime::ErrorKind::Unsupported,
                    format!(
                        "{} content size(s) (Length::MinContent, Length::MaxContent, Length::FitContent) outside width, height and basis; they are auto there",
                        resolved.misplaced_content
                    ),
                    Some(at),
                ),
            );
        }
        if resolved.unsupported_text > 0 && self.text.note() {
            rt.report(
                Some(owner),
                UiError::new(
                    kivact_runtime::ErrorKind::Unsupported,
                    format!(
                        "{} text value(s) the host cannot draw (a font feature tag that is not four ASCII characters, a text shadow's spread or inset); dropped",
                        resolved.unsupported_text
                    ),
                    Some(at),
                ),
            );
        }
    }
}
