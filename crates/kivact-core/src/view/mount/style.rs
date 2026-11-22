//! Role: Mounting binds the style, disabled and selected states of a host node.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;
use kivact_runtime::driver::EffectKind;
use kivact_runtime::{Signal, UiError, Value};
use kivact_style::{CompiledStyle, Interactive, Style, Theme};
use kivact_types::{LayoutProps, NodeId, StateSet};

use super::facts::{LocalPaint, LocalPaintSource, NodeFacts};
use crate::view::Variants;
use crate::view::descriptor::Mcx;
use crate::view::devtools_hooks::StyleHook;
use crate::view::element_data::{LayerFn, StyleLayer};
use crate::view::state::ViewState;

/// Merge `layers` in order (a recipe layer reads `variants`). In debug builds
/// it also counts the declarations that repeat a property within their own
/// layer: a repeat across layers is an overlay, not a duplicate.
fn merge_layers(layers: &[StyleLayer], variants: &Variants) -> (Style, u32) {
    let mut duplicates = 0;
    let mut out: Option<Style> = None;
    for layer in layers {
        let style = layer.f.style(variants);
        if cfg!(debug_assertions) {
            duplicates += kivact_style::internal::duplicates(&style);
        }
        out = Some(match out {
            Some(acc) => acc.merge(&style),
            None => style,
        });
    }
    (out.unwrap_or_default(), duplicates)
}

/// The memo of the merged layers of a node (see [`Layers::Memo`]), with the
/// DevTools hook's overrides applied.
fn merged_memo(hook: StyleHook, layers: Vec<StyleLayer>, variants: Variants) -> Value<Merged> {
    kivact_runtime::memo(move || {
        kivact_runtime::pure(|| {
            let (style, duplicates) = merge_layers(&layers, &variants);
            Merged(Rc::new((hook.apply(style), duplicates)))
        })
    })
}

/// The layers with each recipe layer fixed to `variants`, for DevTools: the
/// style inspector reads a plain closure per layer.
fn plain_layers(layers: &[StyleLayer], variants: &Variants) -> Vec<StyleLayer> {
    layers
        .iter()
        .map(|layer| match &layer.f {
            LayerFn::Variants(f) => {
                let (f, variants) = (f.clone(), variants.clone());
                StyleLayer {
                    f: LayerFn::Plain(Rc::new(move || f(&variants))),
                    ..layer.clone()
                }
            }
            LayerFn::Plain(_) => layer.clone(),
        })
        .collect()
}

/// The readers of the disabled and selected states, for the style effect.
pub(super) struct States {
    pub(super) disabled: Option<Rc<dyn Fn() -> bool>>,
    pub(super) selected: Option<Rc<dyn Fn() -> bool>>,
    /// Whether either reader can change (it is not a fixed value).
    pub(super) change: bool,
}

/// The merged layers of one node, with the overrides applied, and the
/// duplicates of its layers. Each recomputation is a new value, so equality
/// is identity: the memo notifies only when it ran again.
#[derive(Clone)]
struct Merged(Rc<(Style, u32)>);
impl PartialEq for Merged {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// The merged layers of a node, as its style effect reads them.
enum Layers {
    /// A node whose states can change (disabled, selected, forced, or host
    /// states it reports): the layers' closures run again only when what
    /// they read changes, not when a state does. Its compiled styles are
    /// cached, since a state change compiles again from the same merge.
    Memo(Value<Merged>, CompileCache),
    /// A node without such states: its style effect runs again only when
    /// the layers' reads or the theme change, so the effect merges the
    /// layers itself and no merged copy is kept per node.
    Direct(Vec<StyleLayer>, Variants),
}

/// The last two compiled styles of a node, by merged style, theme and
/// interactive state. Pressing and releasing alternate between two states,
/// and compiling is a pure function of these three. Only a [`Layers::Memo`]
/// node has one, and its entries are allocated when first compiled: an
/// entry holds a whole compiled style, too large to keep inline per node.
#[derive(Default)]
struct CompileCache(RefCell<Vec<CompileEntry>>);

/// One entry of [`CompileCache`]. It holds the merged style, so its address
/// cannot be reused while the entry lives.
struct CompileEntry {
    merged: Rc<(Style, u32)>,
    theme: Theme,
    state: Interactive,
    compiled: CompiledStyle,
}

impl CompileCache {
    /// The compiled style of `merged` in `theme` and `state`, compiled now
    /// only when no entry has them.
    fn compile(
        &self,
        theme: &Theme,
        merged: &Rc<(Style, u32)>,
        state: Interactive,
    ) -> CompiledStyle {
        let mut slots = self.0.borrow_mut();
        let hit = slots
            .iter()
            .find(|e| Rc::ptr_eq(&e.merged, merged) && e.state == state && e.theme == *theme);
        if let Some(hit) = hit {
            return hit.compiled.clone();
        }
        let compiled = kivact_style::internal::compile_style(merged.0.clone(), theme, state);
        if slots.len() == 2 {
            slots.pop();
        }
        slots.insert(
            0,
            CompileEntry {
                merged: merged.clone(),
                theme: theme.clone(),
                state,
                compiled: compiled.clone(),
            },
        );
        compiled
    }
}

/// From now on the merged layers of a node that reports host states are
/// kept in a memo: its style effect runs again on every state change, and
/// the layers need to run again only when what they read changes.
fn keep_merged(source: &RefCell<Layers>, owner: kivact_runtime::Owner, hook: &StyleHook) {
    let next = match &*source.borrow() {
        Layers::Direct(layers, variants) => {
            let (layers, variants) = (layers.clone(), variants.clone());
            Some(Layers::Memo(
                owner.with_setup(|| merged_memo(hook.clone(), layers, variants)),
                CompileCache::default(),
            ))
        }
        Layers::Memo(..) => None,
    };
    if let Some(next) = next {
        *source.borrow_mut() = next;
    }
}

/// The style inputs of a host node, as [`bind_node_style`] takes them.
pub(super) struct StyleInput {
    /// The style layers, in merge order.
    pub(super) layers: Vec<StyleLayer>,
    /// The variant values of the node (`.variant(..)`), read by its recipe layers.
    pub(super) variants: Variants,
    /// Applied to every resolved layout (see [`NodeStyle::fix_layout`]).
    pub(super) fix_layout: Option<fn(&mut LayoutProps)>,
}

/// Stage 4: bind the style effect when the node has style layers, owns part
/// of its layout, or is a focus stop (which gets the theme's focus ring).
pub(super) fn bind_node_style(
    cx: &Mcx<'_>,
    id: NodeId,
    input: StyleInput,
    facts: &NodeFacts,
    states: States,
    at: &'static Location<'static>,
) {
    let StyleInput {
        layers,
        variants,
        fix_layout,
    } = input;
    if !layers.is_empty() || fix_layout.is_some() || facts.focus_stop {
        let style = NodeStyle {
            layers,
            variants,
            focusable: facts.focus_stop,
            disabled: states.disabled,
            selected: states.selected,
            states_change: states.change,
            fix_layout,
            local_paint: facts.local_paint.clone(),
        };
        bind_style(cx, id, facts.kind, style, at);
    }
}

/// What the style of a host node is resolved from (see [`bind_style`]).
#[derive(Default)]
pub(crate) struct NodeStyle {
    /// Style layers, merged in order.
    pub layers: Vec<StyleLayer>,
    /// The node's variant values (`.variant(..)`), read by its recipe layers.
    pub variants: Variants,
    /// The node is a focus stop: the theme's focus ring and disabled look
    /// apply under its style.
    pub focusable: bool,
    /// Reader of the disabled state (applies the `disabled` state style).
    pub disabled: Option<Rc<dyn Fn() -> bool>>,
    /// Reader of the selected state (applies the `selected` state style).
    pub selected: Option<Rc<dyn Fn() -> bool>>,
    /// Whether the disabled or selected reader can change: only then are
    /// the merged layers kept in a memo (see [`Layers`]).
    pub states_change: bool,
    /// Applied to every resolved layout before it is set (a builder that
    /// owns part of its node's layout).
    pub fix_layout: Option<fn(&mut LayoutProps)>,
    /// How the style's paint callback is recorded.
    pub local_paint: LocalPaint,
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
        variants,
        focusable,
        disabled,
        selected,
        states_change,
        fix_layout,
        local_paint,
    } = style;
    let owner = cx.owner;
    let window = cx.window;
    let local_paint = RefCell::new(local_paint);
    let hooked = crate::view::devtools_hooks::hooks(cx.rt).is_some();
    if hooked {
        ViewState::tree_mut(cx.rt).set_style_layers(id, plain_layers(&layers, &variants));
    }
    let hook = crate::view::devtools_hooks::StyleHook::new(cx.rt, cx.owner, id, at);
    let reports = StyleReports::default();
    let theme = cx.theme();
    // The layers' closures run again only when what they read changes, not
    // when the disabled or selected state or the forced states change.
    let source = RefCell::new(if states_change || hooked {
        Layers::Memo(
            cx.owner
                .with_setup(|| merged_memo(hook.clone(), layers, variants)),
            CompileCache::default(),
        )
    } else {
        Layers::Direct(layers, variants)
    });
    let weak = cx.weak();
    // The host states the engine reports for this node, once its style asks
    // for them (see `track_states`).
    let reported: std::cell::Cell<Option<Signal<StateSet>>> = std::cell::Cell::new(None);
    cx.effect(EffectKind::Render, at, move || {
        let Some(theme_sig) = theme else { return };
        let Some(rt) = weak.upgrade() else { return };
        // The style closures only compute values: pure even when this is
        // an Internal effect feeding a paint transition.
        let (resolved, duplicates) = kivact_runtime::pure(|| {
            theme_sig.with(|theme| {
                let state = Interactive {
                    focusable,
                    disabled: disabled.as_ref().is_some_and(|f| f()),
                    selected: selected.as_ref().is_some_and(|f| f()),
                    root: kind == NodeType::Root,
                    forced: hook.forced(),
                    active: reported.get().map(|sig| sig.get()).unwrap_or_default(),
                };
                match &*source.borrow() {
                    Layers::Memo(merged, cache) => {
                        let merged = merged.with(|m| m.0.clone());
                        let duplicates = merged.1;
                        (cache.compile(theme, &merged, state), duplicates)
                    }
                    Layers::Direct(layers, variants) => {
                        let (style, duplicates) = merge_layers(layers, variants);
                        let style = hook.apply(style);
                        let resolved = kivact_style::internal::compile_style(style, theme, state);
                        (resolved, duplicates)
                    }
                }
            })
        });
        let mcx = Mcx {
            rt: &rt,
            owner,
            window,
        };
        track_states(&rt, id, owner, at, &reported, &resolved);
        if reported.get().is_some() {
            keep_merged(&source, owner, &hook);
        }
        feed_local_paint(&mcx, id, &local_paint, &resolved, &reports, at);
        reports.report(&rt, owner, id, &resolved, duplicates, at);
        write_props(&rt, id, resolved, fix_layout);
    });
}

/// Ask the engine for the host states of node `id` once its compiled state
/// styles set more than paint (ST-05). The style effect then reads them as
/// `Interactive::active`, and runs again when they change.
fn track_states(
    rt: &kivact_runtime::driver::Runtime,
    id: NodeId,
    owner: kivact_runtime::Owner,
    at: &'static Location<'static>,
    reported: &std::cell::Cell<Option<Signal<StateSet>>>,
    resolved: &kivact_style::CompiledStyle,
) {
    let wants = resolved
        .states
        .as_ref()
        .is_some_and(|s| !s.report.is_empty());
    if wants && reported.get().is_none() {
        let sig = crate::view::node_states::source(rt, id, owner, at);
        // Read it from now on: the effect runs again when the engine reports
        // a change.
        let _ = sig.get();
        reported.set(Some(sig));
    }
}

/// Set the props of a resolved style on node `id`; `fix_layout` applies to
/// the layout first.
fn write_props(
    rt: &kivact_runtime::driver::Runtime,
    id: NodeId,
    resolved: kivact_style::CompiledStyle,
    fix_layout: Option<fn(&mut LayoutProps)>,
) {
    let kivact_style::CompiledStyle {
        mut layout,
        paint,
        text,
        states,
        ..
    } = resolved;
    if let Some(fix) = fix_layout {
        fix(&mut layout);
    }
    let writer = rt.running_effect();
    let mut tree = ViewState::tree_mut(rt);
    tree.update(id, PropValue::Layout(layout), writer);
    tree.update(id, PropValue::Paint(paint), writer);
    tree.update(id, PropValue::TextStyle(text), writer);
    tree.update(id, PropValue::States(states.unwrap_or_default()), writer);
}

/// Hand the style's paint callback to the node's draw effect (see
/// [`LocalPaint`]). The callback's own reads are tracked by that effect,
/// so a signal only the callback reads never runs the style again.
fn feed_local_paint(
    cx: &Mcx<'_>,
    id: NodeId,
    local_paint: &RefCell<LocalPaint>,
    resolved: &kivact_style::CompiledStyle,
    reports: &StyleReports,
    at: &'static Location<'static>,
) {
    let has_paint = resolved.has_retained_paint();
    if has_paint && matches!(*local_paint.borrow(), LocalPaint::OnFirstPaint) {
        let source = LocalPaintSource::new(cx.owner, at);
        *source.value.borrow_mut() = resolved.retained_paint();
        super::draw::bind_style_paint(cx, id, source.clone(), at);
        *local_paint.borrow_mut() = LocalPaint::WithDraw(source);
    }
    match &*local_paint.borrow() {
        LocalPaint::None => {
            if has_paint && reports.leaf_paint.note() {
                cx.rt.report(
                    Some(cx.owner),
                    UiError::new(
                        kivact_runtime::ErrorKind::Unsupported,
                        "local paint requires a box node; apply it to the containing node",
                        Some(at),
                    ),
                );
            }
        }
        LocalPaint::OnFirstPaint => {}
        LocalPaint::WithDraw(source) => {
            // A style can acquire paint later; observe only when it first does.
            // Never disable observation needed by a NodeRef or another consumer.
            if has_paint {
                ViewState::tree_mut(cx.rt).set_observe(id, true);
            }
            // Callbacks compare by identity: a node redraws only for a new one.
            let retained = resolved.retained_paint();
            let changed = *source.value.borrow() != retained;
            if changed {
                *source.value.borrow_mut() = retained;
                source.revision.notify_from_render();
            }
        }
    }
}

#[derive(Default)]
struct StyleReports {
    content: kivact_types::cell::ReportOnce,
    text: kivact_types::cell::ReportOnce,
    states: kivact_types::cell::ReportOnce,
    leaf_paint: kivact_types::cell::ReportOnce,
    /// The compile reports already shown for this node, by message.
    compiled: kivact_types::cell::ReportOnceSet<String>,
}
impl StyleReports {
    fn report(
        &self,
        rt: &kivact_runtime::driver::Runtime,
        owner: kivact_runtime::Owner,
        id: NodeId,
        resolved: &kivact_style::CompiledStyle,
        duplicates: u32,
        at: &'static Location<'static>,
    ) {
        if !resolved.nested_in_states.is_empty() && self.states.note() {
            rt.report(
                Some(owner),
                UiError::new(
                    kivact_runtime::ErrorKind::Unsupported,
                    format!(
                        "state styles set a state or group inside a state, ignored: {}; set each state and group() on the base style",
                        resolved.nested_in_states.join(", ")
                    ),
                    Some(at),
                ),
            );
        }
        if duplicates > 0 && cfg!(debug_assertions) {
            rt.note(format!(
                "style of node {} sets {} properties twice (last wins)",
                id.0, duplicates
            ));
        }
        if resolved.misplaced_lengths > 0 && self.content.note() {
            rt.report(
                Some(owner),
                UiError::new(
                    kivact_runtime::ErrorKind::Unsupported,
                    format!(
                        "{} length(s) the property cannot take: a content size outside width, height and basis, or auto, a content size or a variable left unresolved in a paint position or radius; they are 0 or auto there",
                        resolved.misplaced_lengths
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
        // Each compile report goes to the owner once, by its message: a
        // recompilation that repeats one stays silent.
        for error in &resolved.reports {
            if self.compiled.note(error.message().to_string()) {
                let kind = error.kind().clone();
                rt.report(Some(owner), UiError::new(kind, error.message(), Some(at)));
            }
        }
    }
}
