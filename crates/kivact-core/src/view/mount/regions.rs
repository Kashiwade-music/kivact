//! Role: Region registries retain keyed attachments across geometry updates.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::panic::Location;
use std::rc::Rc;

use super::{parts, region_template};
use crate::view::common::{
    Adopted, Common, Declaration, Entry, Gestures, RegionBinding, RegionEntry, RegionKey, Registry,
    RegistryPhase, State, Target, TargetId,
};
use crate::view::descriptor::Mcx;
use kivact_runtime::{EffectKind, ErrorKind, Owner, Runtime, UiError};
use kivact_style::draw::{RegionId, RegionSemantics};
use kivact_types::NodeId;

impl Registry {
    pub(super) fn entry(&self, id: RegionId, template: Common, item_owner: Owner) -> Rc<Entry> {
        let mut node = crate::view::node::Node::new(crate::view::node::Content::Box, None);
        node.common = template;
        while !node.common.deferred_parts.is_empty() {
            let deferred = std::mem::take(&mut node.common.deferred_parts);
            for apply in deferred {
                node = apply(node).resolve_modifiers();
            }
        }
        let mut template = node.common;
        if template.text_editing.take().is_some()
            && let Some(rt) = self.root.runtime.upgrade()
        {
            rt.report(
                Some(item_owner),
                UiError::new(
                    ErrorKind::Unsupported,
                    "TextEditing requires a node attachment; declaration ignored",
                    Some(self.at),
                ),
            );
        }
        let mut state = self.state.borrow_mut();
        let generation = state.next.next();
        Rc::new(Entry {
            id,
            generation,
            item_owner,
            template,
            state: RefCell::new(RegionEntry::Pending),
        })
    }

    pub(super) fn lookup(&self, id: RegionId) -> Option<Rc<Target>> {
        self.state.borrow().by_id.get(&id).cloned()
    }

    /// End the pointer gestures that target `id` (it was disabled, hidden or retired).
    pub(super) fn cancel(&self, id: TargetId) {
        super::part_pointer::cancel(self, id);
    }

    pub(super) fn duplicate_key(&self, hash: u64) {
        if self.state.borrow().duplicate_keys.note(hash)
            && let Some(rt) = self.root.runtime.upgrade()
        {
            rt.report(
                Some(self.root.owner),
                UiError::new(
                    ErrorKind::DuplicateKey { key_hash: hash },
                    format!("duplicate canvas region key (hash {hash:#x}); later item dropped"),
                    Some(self.at),
                ),
            );
        }
    }

    pub(super) fn update(&self, index: usize, entries: Vec<Rc<Entry>>, retired: Vec<Owner>) {
        {
            let mut state = self.state.borrow_mut();
            state.groups[index] = entries;
            state.retired.extend(retired);
        }
        self.changed.notify();
    }

    /// Adoption changes dispose old attachment owners before new refs bind.
    pub(super) fn reconcile(self: &Rc<Self>) {
        let Some(rt) = self.root.runtime.upgrade() else {
            return;
        };
        kivact_runtime::untrack(|| {
            let (wanted, old, duplicates) = {
                let state = self.state.borrow();
                let mut seen = std::collections::HashMap::new();
                let mut duplicates = Vec::new();
                let wanted = state
                    .groups
                    .iter()
                    .flatten()
                    .filter_map(|entry| {
                        if let Some(by) = seen.get(&entry.id).copied() {
                            if !matches!(*entry.state.borrow(), RegionEntry::Adopted(_)) {
                                *entry.state.borrow_mut() = RegionEntry::Shadowed { by };
                            }
                            duplicates.push(entry.id);
                            None
                        } else {
                            seen.insert(entry.id, RegionKey(entry.generation));
                            Some(entry.clone())
                        }
                    })
                    .collect::<Vec<_>>();
                (wanted, state.active.clone(), duplicates)
            };
            for id in duplicates {
                if self.state.borrow().duplicates.note(id) {
                    rt.report(Some(self.root.owner), UiError::new(ErrorKind::Message,
                    format!("canvas region {} has duplicate setup declarations; later attachment dropped", id.0), Some(self.at)));
                }
            }
            let retained: HashSet<_> = wanted.iter().map(|entry| entry.generation).collect();
            for entry in old {
                if !retained.contains(&entry.generation) {
                    let next = self
                        .state
                        .borrow()
                        .groups
                        .iter()
                        .flatten()
                        .find(|candidate| {
                            candidate.id == entry.id && candidate.generation != entry.generation
                        })
                        .map(|candidate| RegionEntry::Shadowed {
                            by: RegionKey(candidate.generation),
                        })
                        .unwrap_or(RegionEntry::Pending);
                    let previous = std::mem::replace(&mut *entry.state.borrow_mut(), next);
                    if let RegionEntry::Adopted(adopted) = previous {
                        self.cancel(adopted.target.id);
                        adopted.owner.dispose();
                    }
                }
            }
            // Removed item state must outlive cancellation and adopted ref cleanup.
            let retired = std::mem::take(&mut self.state.borrow_mut().retired);
            for owner in retired {
                owner.dispose();
            }
            for entry in &wanted {
                if !matches!(*entry.state.borrow(), RegionEntry::Adopted(_)) {
                    if let RegionEntry::Shadowed { by } = &*entry.state.borrow() {
                        debug_assert!(by.0 != entry.generation, "an entry cannot shadow itself");
                    }
                    let adopted = self.adopt(&rt, entry);
                    *entry.state.borrow_mut() = RegionEntry::Adopted(adopted);
                }
            }
            let by_id = wanted
                .iter()
                .filter_map(|entry| match &*entry.state.borrow() {
                    RegionEntry::Adopted(adopted) => Some((entry.id, adopted.target.clone())),
                    RegionEntry::Pending | RegionEntry::Shadowed { .. } => None,
                })
                .collect();
            {
                let mut state = self.state.borrow_mut();
                state.active = wanted;
                state.by_id = by_id;
            }
            self.revision.notify();
        })
    }

    fn adopt(self: &Rc<Self>, rt: &Rc<Runtime>, entry: &Entry) -> Rc<Adopted> {
        let owner = entry.item_owner.child(self.at);
        owner.with_setup(|| {
            let cx = Mcx { rt, owner };
            let mut common = region_template::instantiate(&entry.template);
            if std::mem::take(&mut common.part_modal) {
                parts::report_modal_unsupported(&cx, self.at);
            }
            let sources = parts::ConstraintSources {
                disabled: common.disabled.take(),
                hidden: common.hidden.take(),
                focusable: common.focusable.take(),
                required: std::mem::take(&mut common.part_constraints),
            };
            let constraints =
                parts::constraints(sources, &cx, self.at, Some(self.root.constraints));
            let mut handlers = std::mem::take(&mut common.handlers);
            let mut actions = handlers.take_part_actions();
            actions.compose(std::mem::take(&mut common.part_actions));
            let bindings = std::mem::take(&mut common.part_bindings);
            let generation = {
                let mut state = self.state.borrow_mut();
                state.next.next()
            };
            let id = TargetId::Region {
                region: entry.id,
                generation,
            };
            let target = Rc::new(Target {
                id,
                owner,
                runtime: Rc::downgrade(rt),
                handlers,
                actions,
                bindings,
                constraints,
            });
            let weak = Rc::downgrade(self);
            owner.effect(
                EffectKind::Internal,
                move || {
                    if (constraints.disabled.get() || constraints.hidden.get())
                        && let Some(registry) = weak.upgrade()
                    {
                        registry.cancel(id);
                    }
                },
                self.at,
            );
            let adopted = Rc::new(Adopted {
                owner,
                common,
                target,
                generation,
                binding: Cell::new(RegionBinding::Unbound),
            });
            if self.phase.get() == RegistryPhase::Mounted {
                self.bind_adopted(rt, &adopted);
            }
            adopted
        })
    }

    fn bind_adopted(&self, rt: &Rc<Runtime>, adopted: &Adopted) {
        debug_assert!(
            matches!(adopted.target.id, TargetId::Region { generation, .. } if generation == adopted.generation)
        );
        let Some(region) = adopted.target.region() else {
            return;
        };
        if adopted.binding.replace(RegionBinding::Bound) == RegionBinding::Bound {
            return;
        }
        adopted.owner.with_setup(|| {
            for (index, reference) in adopted.common.node_ref.iter().enumerate() {
                if !adopted.common.node_ref[..index]
                    .iter()
                    .any(|previous| previous == reference)
                {
                    reference.bind_region(rt, self.node, region, adopted.owner, self.at);
                }
            }
            parts::bind_hooks(
                adopted.owner,
                &adopted.target.bindings,
                adopted.target.constraints,
            );
        })
    }

    /// Initial refs bind only after the canvas has installed every handler.
    pub(super) fn bind(&self) {
        self.phase.set(RegistryPhase::Mounted);
        let Some(rt) = self.root.runtime.upgrade() else {
            return;
        };
        let entries = self.state.borrow().active.clone();
        for entry in entries {
            if let RegionEntry::Adopted(adopted) = &*entry.state.borrow() {
                self.bind_adopted(&rt, adopted);
            }
        }
    }

    pub(super) fn resolve(&self) -> Vec<(RegionId, RegionSemantics)> {
        self.revision.track();
        let entries = self.state.borrow().active.clone();
        entries
            .iter()
            .enumerate()
            .filter_map(|(order, entry)| {
                let adopted = match &*entry.state.borrow() {
                    RegionEntry::Adopted(adopted) => adopted.clone(),
                    RegionEntry::Pending | RegionEntry::Shadowed { .. } => return None,
                };
                let common = &adopted.common;
                let properties = common
                    .semantics
                    .as_ref()
                    .map(|sources| super::semantics::resolve(sources, self.at))
                    .unwrap_or_default();
                Some((
                    entry.id,
                    RegionSemantics {
                        semantics: kivact_types::Semantics {
                            role: Some(common.role.unwrap_or(crate::view::Role::Generic)),
                            name: common
                                .name
                                .as_ref()
                                .map(kivact_runtime::Prop::get)
                                .map(Some)
                                .unwrap_or_default(),
                            description: common.description.as_ref().map(kivact_runtime::Prop::get),
                            value_text: common.value.as_ref().map(kivact_runtime::Prop::get),
                            actions: Some(adopted.target.actions.actions()),
                            checked: super::semantics::checked_state(
                                common.checked.as_ref().map(kivact_runtime::Prop::get),
                                common.indeterminate.as_ref().map(kivact_runtime::Prop::get),
                            ),
                            selected: common.selected.as_ref().map(kivact_runtime::Prop::get),
                            expanded: common.expanded.as_ref().map(kivact_runtime::Prop::get),
                            invalid: common.invalid.as_ref().map(kivact_runtime::Prop::get),
                            disabled: adopted.target.constraints.disabled.get(),
                            hidden: adopted.target.constraints.hidden.get(),
                            focusable: adopted.target.constraints.focusable.get(),
                            ..properties
                        },
                        focus_order: order.min(u32::MAX as usize) as u32,
                        focus_on_press: common.focus_on_press.and_then(|r| r.target_value(self.at)),
                    },
                ))
            })
            .collect()
    }
}

pub(super) fn prepare(
    declarations: Vec<Declaration>,
    root: Rc<Target>,
    cx: &Mcx<'_>,
    node: NodeId,
    at: &'static Location<'static>,
) -> Rc<Registry> {
    let registry = Rc::new(Registry {
        node,
        root,
        at,
        state: RefCell::new(State::default()),
        revision: cx.owner.notifier(at),
        changed: cx.owner.notifier(at),
        phase: Cell::new(RegistryPhase::Mounting),
        gestures: RefCell::new(Gestures::default()),
    });
    registry
        .state
        .borrow_mut()
        .groups
        .resize_with(declarations.len(), Vec::new);
    for (index, declaration) in declarations.into_iter().enumerate() {
        match declaration {
            Declaration::Static(id, common) => {
                let entry = registry.entry(id, *common, cx.owner);
                registry.state.borrow_mut().groups[index].push(entry);
            }
            Declaration::Collection(register) => register(cx, registry.clone(), index),
        }
    }
    // Drivers have lower creation order than this shared reconciler, so every
    // collection settles before duplicate checks, adoption, or drawing runs.
    let weak = Rc::downgrade(&registry);
    cx.effect(EffectKind::Internal, at, move || {
        if let Some(registry) = weak.upgrade() {
            registry.changed.track();
            registry.reconcile();
        }
    });
    // The node itself ends its gestures when it is disabled or hidden.
    let weak = Rc::downgrade(&registry);
    let constraints = registry.root.constraints;
    cx.effect(EffectKind::Internal, at, move || {
        if (constraints.disabled.get() || constraints.hidden.get())
            && let Some(registry) = weak.upgrade()
        {
            registry.cancel(TargetId::Root);
        }
    });
    registry
}
