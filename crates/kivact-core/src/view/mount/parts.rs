//! Role: Mounted parts combine caller constraints and preserve behavior defaults.

use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::Owner;
use kivact_reactive::Prop as InputProp;

use crate::view::common::SemanticSources;
use crate::view::common::{Common, Constraints};
use crate::view::descriptor::Mcx;
use crate::view::part_data::{PartBinding, PartMount};

impl Common {
    /// Merge part semantics and handlers now, but preserve constraints until mount.
    pub(crate) fn attach_part(&mut self, mut part: Common) {
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
        self.part_modal |= part.part_modal;
        self.part_constraints.disabled.extend(part.disabled.take());
        self.part_constraints
            .disabled
            .append(&mut part.part_constraints.disabled);
        self.part_constraints.hidden.extend(part.hidden.take());
        self.part_constraints
            .hidden
            .append(&mut part.part_constraints.hidden);
        self.part_constraints
            .focusable
            .extend(take_focusable(&mut part));
        self.part_constraints
            .focusable
            .append(&mut part.part_constraints.focusable);
        replace(&mut self.focus_on_press, part.focus_on_press.take());
        replace(&mut self.role, part.role.take());
        replace(&mut self.text_editing, part.text_editing.take());
        replace(&mut self.name, part.name.take());
        replace(&mut self.description, part.description.take());
        replace(&mut self.value, part.value.take());
        replace(&mut self.selected, part.selected.take());
        replace(&mut self.invalid, part.invalid.take());
        replace(&mut self.checked, part.checked.take());
        replace(&mut self.expanded, part.expanded.take());
        if let Some(semantics) = part.semantics {
            merge_semantics(
                self.semantics.get_or_insert_with(Default::default),
                *semantics,
            );
        }
        self.node_ref.append(&mut part.node_ref);
        self.part_bindings.append(&mut part.part_bindings);
        self.part_actions.compose(part.handlers.take_part_actions());
        self.part_actions
            .compose(std::mem::take(&mut part.part_actions));
    }
}

/// Only Overlay consumes a modal requirement; other attachments report it.
pub(super) fn reject_modal(common: &mut Common, cx: &Mcx<'_>, at: &'static Location<'static>) {
    if std::mem::take(&mut common.part_modal) {
        cx.rt.report(
            Some(cx.owner),
            kivact_reactive::UiError::new(
                kivact_reactive::ErrorKind::Unsupported,
                "Part.modal(true) requires an Overlay attachment; requirement dropped",
                Some(at),
            ),
        );
    }
}

fn replace<T>(target: &mut Option<T>, incoming: Option<T>) {
    if incoming.is_some() {
        *target = incoming;
    }
}

fn merge_semantics(target: &mut SemanticSources, mut part: SemanticSources) {
    replace(&mut target.indeterminate, part.indeterminate.take());
    replace(&mut target.value_min, part.value_min.take());
    replace(&mut target.value_max, part.value_max.take());
    replace(&mut target.value_now, part.value_now.take());
    replace(&mut target.live_region, part.live_region.take());
    replace(&mut target.sort, part.sort.take());
    replace(&mut target.orientation, part.orientation.take());
    replace(&mut target.active_descendant, part.active_descendant.take());
    replace(&mut target.level, part.level.take());
    replace(&mut target.set_size, part.set_size.take());
    replace(&mut target.pos_in_set, part.pos_in_set.take());
    replace(&mut target.multiselectable, part.multiselectable.take());
    target.described_by.append(&mut part.described_by);
    target.controls.append(&mut part.controls);
    target.labelled_by.append(&mut part.labelled_by);
}

fn take_focusable(common: &mut Common) -> Option<InputProp<bool>> {
    let constant = common.focusable.take();
    common
        .semantics
        .as_mut()
        .and_then(|s| s.focusable.take())
        .or_else(|| constant.map(kivact_reactive::Prop::constant))
}

/// Resolve only after all caller modifiers have run, before semantic mounting.
pub(super) fn constraints(
    common: &mut Common,
    cx: &Mcx<'_>,
    at: &'static Location<'static>,
    parent: Option<Constraints>,
) -> Constraints {
    let value = |source: InputProp<bool>| source.into_value(cx.owner, at);
    let caller_disabled = value(
        common
            .disabled
            .take()
            .unwrap_or_else(|| kivact_reactive::Prop::constant(false)),
    );
    let caller_hidden = value(
        common
            .hidden
            .take()
            .unwrap_or_else(|| kivact_reactive::Prop::constant(false)),
    );
    let caller_focus = take_focusable(common).map(value);
    let mut required_disabled = std::mem::take(&mut common.part_constraints.disabled);
    let mut required_hidden = std::mem::take(&mut common.part_constraints.hidden);
    if let Some(parent) = parent {
        required_disabled.push(kivact_reactive::Prop::value(parent.disabled));
        required_hidden.push(kivact_reactive::Prop::value(parent.hidden));
    }
    let required_focus = std::mem::take(&mut common.part_constraints.focusable);
    let focus_default = !required_focus.is_empty();
    let disabled = value(kivact_reactive::Prop::computed(Rc::new(move || {
        caller_disabled.get() || required_disabled.iter().any(kivact_reactive::Prop::get)
    })));
    let hidden = value(kivact_reactive::Prop::computed(Rc::new(move || {
        caller_hidden.get() || required_hidden.iter().any(kivact_reactive::Prop::get)
    })));
    let focusable = value(kivact_reactive::Prop::computed(Rc::new(move || {
        caller_focus.map_or(focus_default, |v| v.get())
            && required_focus.iter().all(kivact_reactive::Prop::get)
    })));
    let eligible = value(kivact_reactive::Prop::computed(Rc::new(move || {
        !caller_disabled.get()
            && !caller_hidden.get()
            && caller_focus.is_none_or(|v| v.get())
            && parent.is_none_or(|p| !p.caller_disabled.get() && !p.caller_hidden.get())
    })));
    common.disabled = Some(kivact_reactive::Prop::value(disabled));
    common.hidden = Some(kivact_reactive::Prop::value(hidden));
    common
        .semantics
        .get_or_insert_with(Default::default)
        .focusable = Some(kivact_reactive::Prop::value(focusable));
    Constraints {
        disabled,
        hidden,
        focusable,
        eligible,
        caller_disabled,
        caller_hidden,
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
