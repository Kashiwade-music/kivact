//! Role: Mounted parts combine caller constraints and preserve behavior defaults.

use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::Prop as InputProp;
use kivact_runtime::{Owner, Value};

use crate::view::descriptor::Mcx;
use crate::view::element_data::{Common, Constraints};
use crate::view::part_data::{PartBinding, PartConstraints, PartMount};

impl Common {
    /// Apply a later caller layer without changing handler or constraint ownership.
    pub(crate) fn apply_caller(&mut self, mut later: Common) {
        macro_rules! replace_fields { ($($field:ident),* $(,)?) => { $(replace(&mut self.$field, later.$field.take());)* }; }
        replace_fields!(
            exit_layout,
            focusable_when_disabled,
            exit_complete,
            roving_focus,
            hit_testable,
            focus_on_press,
            text_editing,
            ime,
            draw,
            draw_over,
            draw_with_layout,
            intrinsic,
            image_fit,
            monochrome,
            image_playing,
            layout_id,
            at
        );
        self.part_modal |= later.part_modal;
        self.part_region_events |= later.part_region_events;
        self.append_semantics(&mut later.semantics);
        self.node_ref.append(&mut later.node_ref);
        let at = self.base_count();
        let n = later.base_count();
        self.style.splice(at..at, later.style.drain(..n));
        self.style.append(&mut later.style);
        self.regions.append(&mut later.regions);
        self.part_bindings.append(&mut later.part_bindings);
        self.deferred_parts.append(&mut later.deferred_parts);
        self.part_constraints
            .disabled
            .append(&mut later.part_constraints.disabled);
        self.part_constraints
            .hidden
            .append(&mut later.part_constraints.hidden);
        self.part_constraints
            .focusable
            .append(&mut later.part_constraints.focusable);
        self.handlers.merge_caller(later.handlers);
        self.part_actions.compose(later.part_actions);
    }

    /// Merge part semantics and handlers now, but preserve constraints until mount.
    ///
    /// Precedence: a property a part sets (role, name, description, value,
    /// selected, invalid, checked, expanded, the extra semantic sources, text
    /// editing, the press focus destination) replaces what an earlier part or
    /// the node set before it; a part attached later wins. Lists (relations,
    /// references, bindings, actions, handlers) and the disabled, hidden and
    /// focusable requirements accumulate instead. A later part therefore
    /// restates a semantic of the same target (for example `expanded`) without
    /// removing the earlier part's other behavior.
    pub(crate) fn attach_part(&mut self, mut part: Common) {
        self.deferred_parts.append(&mut part.deferred_parts);
        let region_events = part.part_region_events;
        self.handlers
            .key
            .absorb_part(std::mem::take(&mut part.handlers.key), region_events);
        self.handlers
            .key_up
            .absorb_part(std::mem::take(&mut part.handlers.key_up), region_events);
        self.handlers
            .pointer
            .absorb_part(std::mem::take(&mut part.handlers.pointer), region_events);
        self.handlers
            .wheel
            .absorb_part(std::mem::take(&mut part.handlers.wheel), region_events);
        replace(
            &mut self.handlers.secondary_context,
            part.handlers.secondary_context.take(),
        );
        self.part_modal |= part.part_modal;
        if let Some(required) = part.focusable_when_disabled.take() {
            let caller = self.focusable_when_disabled.take();
            self.focusable_when_disabled =
                Some(kivact_runtime::Prop::computed(Rc::new(move || {
                    caller.as_ref().is_some_and(|v| v.get()) || required.get()
                })));
        }
        self.part_constraints
            .disabled
            .extend(part.disabled_source());
        self.part_constraints
            .disabled
            .append(&mut part.part_constraints.disabled);
        self.part_constraints.hidden.extend(part.hidden_source());
        self.part_constraints
            .hidden
            .append(&mut part.part_constraints.hidden);
        self.part_constraints
            .focusable
            .extend(part.focusable_source());
        self.part_constraints
            .focusable
            .append(&mut part.part_constraints.focusable);
        replace(&mut self.focus_on_press, part.focus_on_press.take());
        replace(&mut self.text_editing, part.text_editing.take());
        part.semantics.retain(|p| {
            !matches!(
                p.input,
                crate::view::semantics_data::SemanticInput::Disabled(_)
                    | crate::view::semantics_data::SemanticInput::Hidden(_)
                    | crate::view::semantics_data::SemanticInput::Focusable(_)
            )
        });
        self.append_semantics(&mut part.semantics);
        self.node_ref.append(&mut part.node_ref);
        self.part_bindings.append(&mut part.part_bindings);
        self.part_actions.compose(part.handlers.take_part_actions());
        self.part_actions
            .compose(std::mem::take(&mut part.part_actions));
    }
}

/// Report a `Part::modal` requirement on a node or region that is not an Overlay (only an Overlay
/// consumes it; the requirement is dropped).
pub(super) fn report_modal_unsupported(cx: &Mcx<'_>, at: &'static Location<'static>) {
    cx.rt.report(
        Some(cx.owner),
        kivact_runtime::UiError::new(
            kivact_runtime::ErrorKind::Unsupported,
            "Node.modal() requires an Overlay attachment; requirement dropped",
            Some(at),
        ),
    );
}

fn replace<T>(target: &mut Option<T>, incoming: Option<T>) {
    if incoming.is_some() {
        *target = incoming;
    }
}

/// What a target's final constraints combine: the caller's own modifiers and the parts' requirements.
pub(super) struct ConstraintSources {
    // xtask-allow(state-fields): option: a builder input carried to mount; absent means enabled
    pub(super) disabled: Option<InputProp<bool>>,
    // xtask-allow(state-fields): option: a builder input carried to mount; absent means shown
    pub(super) hidden: Option<InputProp<bool>>,
    // xtask-allow(state-fields): option: the caller's own last `focusable` modifier; absent lets the parts decide: focusable when a part requires focus, otherwise not
    pub(super) focusable: Option<InputProp<bool>>,
    pub(super) required: PartConstraints,
}

/// Resolve only after all caller modifiers have run, before semantic mounting.
pub(super) fn constraints(
    sources: ConstraintSources,
    cx: &Mcx<'_>,
    at: &'static Location<'static>,
    parent: Option<Constraints>,
) -> Constraints {
    let ConstraintSources {
        disabled,
        hidden,
        focusable,
        required,
    } = sources;
    let flags = ConstFlags {
        owner: cx.owner,
        at,
        values: Cell::new([None, None]),
    };
    let value = |source: InputProp<bool>| match source.fixed() {
        Some(b) => flags.get(b),
        None => source.into_value(cx.owner, at),
    };
    let caller_disabled = value(disabled.unwrap_or_else(|| InputProp::constant(false)));
    let caller_hidden = value(hidden.unwrap_or_else(|| InputProp::constant(false)));
    let caller_focus = focusable.map(value);
    let PartConstraints {
        disabled: mut required_disabled,
        hidden: mut required_hidden,
        focusable: mut required_focus,
    } = required;
    if let Some(parent) = parent {
        required_disabled.push(InputProp::value(parent.disabled));
        required_hidden.push(InputProp::value(parent.hidden));
    }
    required_disabled.push(InputProp::value(caller_disabled));
    required_hidden.push(InputProp::value(caller_hidden));
    let focus_default = !required_focus.is_empty();
    required_focus.push(
        caller_focus
            .map(InputProp::value)
            .unwrap_or_else(|| InputProp::constant(focus_default)),
    );
    let disabled = value(combine(required_disabled, false));
    let hidden = value(combine(required_hidden, false));
    let focusable = value(combine(required_focus, true));
    let mut eligible = vec![
        InputProp::value(caller_disabled),
        InputProp::value(caller_hidden),
    ];
    if let Some(parent) = parent {
        eligible.push(InputProp::value(parent.caller_disabled));
        eligible.push(InputProp::value(parent.caller_hidden));
    }
    let available = combine(eligible, false);
    let eligible = value(combine(
        vec![
            invert(available),
            caller_focus
                .map(InputProp::value)
                .unwrap_or_else(|| InputProp::constant(true)),
        ],
        true,
    ));
    Constraints {
        disabled,
        hidden,
        focusable,
        eligible,
        caller_disabled,
        caller_hidden,
    }
}

/// The two constant flags one mount needs at most, created on first use.
struct ConstFlags {
    owner: Owner,
    at: &'static Location<'static>,
    values: Cell<[Option<Value<bool>>; 2]>,
}
impl ConstFlags {
    fn get(&self, value: bool) -> Value<bool> {
        let mut values = self.values.get();
        let slot = &mut values[usize::from(value)];
        let v = *slot.get_or_insert_with(|| self.owner.constant(value, self.at));
        self.values.set(values);
        v
    }
}

/// Preserve constants when combining constraint inputs, avoiding a render effect
/// for a node whose authored and behavior requirements are all constant.
fn combine(sources: Vec<InputProp<bool>>, all: bool) -> InputProp<bool> {
    let fixed: Option<Vec<_>> = sources.iter().map(InputProp::fixed).collect();
    let reduce = move |values: Vec<bool>| {
        if all {
            values.into_iter().all(|v| v)
        } else {
            values.into_iter().any(|v| v)
        }
    };
    if let Some(values) = fixed {
        InputProp::constant(reduce(values))
    } else {
        InputProp::computed(Rc::new(move || {
            reduce(sources.iter().map(InputProp::get).collect())
        }))
    }
}
fn invert(source: InputProp<bool>) -> InputProp<bool> {
    match source.fixed() {
        Some(value) => InputProp::constant(!value),
        None => InputProp::computed(Rc::new(move || !source.get())),
    }
}

/// Call hooks with refs already bound, under the actual target's setup owner.
pub(super) fn bind_hooks(owner: Owner, bindings: &[PartBinding], constraints: Constraints) {
    for binding in bindings {
        let mount = PartMount {
            node: binding.node,
            disabled: constraints.disabled,
            hidden: constraints.hidden,
            focusable: constraints.focusable,
            eligible: constraints.eligible,
        };
        for hook in &binding.hooks {
            if let Err(err) = hook(mount) {
                owner.report(err);
            }
        }
    }
}
