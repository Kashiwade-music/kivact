//! Role: Region registries retain keyed attachments across geometry updates.

use kivact_reactive::IntoValue;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::panic::Location;
use std::rc::Rc;

use super::{parts, region_template};
use crate::view::common::{
    Adopted, Common, Declaration, Entry, Gestures, Registry, RegistryPhase, State, Target,
    TargetId,
};
use crate::view::descriptor::Mcx;
use kivact_reactive::{EffectKind, ErrorKind, Owner, Runtime, UiError};
use kivact_style::draw::{RegionId, RegionSemantics};
use kivact_wire::NodeId;

impl Registry {
    pub(super) fn entry(&self, id: RegionId, mut template: Common, item_owner: Owner) -> Rc<Entry> {
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
        state.next += 1;
        Rc::new(Entry {
            id,
            generation: state.next,
            item_owner,
            template,
            adopted: RefCell::new(None),
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
        if self.state.borrow_mut().duplicate_keys.insert(hash)
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
        self.changed.update(|value| *value = value.wrapping_add(1));
    }

    /// Adoption changes dispose old attachment owners before new refs bind.
    pub(super) fn reconcile(self: &Rc<Self>) {
        let Some(rt) = self.root.runtime.upgrade() else {
            return;
        };
        kivact_reactive::untrack(|| {
            let (wanted, old, duplicates) = {
                let state = self.state.borrow();
                let mut seen = HashSet::new();
                let mut duplicates = Vec::new();
                let wanted = state
                    .groups
                    .iter()
                    .flatten()
                    .filter_map(|entry| {
                        if seen.insert(entry.id) {
                            Some(entry.clone())
                        } else {
                            duplicates.push(entry.id);
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                (wanted, state.active.clone(), duplicates)
            };
            for id in duplicates {
                if self.state.borrow_mut().duplicates.insert(id) {
                    rt.report(Some(self.root.owner), UiError::new(ErrorKind::Message,
                    format!("canvas region {} has duplicate setup declarations; later attachment dropped", id.0), Some(self.at)));
                }
            }
            let retained: HashSet<u64> = wanted.iter().map(|entry| entry.generation).collect();
            for entry in old {
                if !retained.contains(&entry.generation) {
                    let adopted = entry.adopted.borrow_mut().take();
                    if let Some(adopted) = adopted {
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
                if entry.adopted.borrow().is_none() {
                    let adopted = self.adopt(&rt, entry);
                    *entry.adopted.borrow_mut() = Some(adopted);
                }
            }
            let by_id = wanted
                .iter()
                .filter_map(|entry| {
                    entry
                        .adopted
                        .borrow()
                        .as_ref()
                        .map(|adopted| (entry.id, adopted.target.clone()))
                })
                .collect();
            {
                let mut state = self.state.borrow_mut();
                state.active = wanted;
                state.by_id = by_id;
            }
            self.revision.update(|value| *value = value.wrapping_add(1));
        })
    }

    fn adopt(self: &Rc<Self>, rt: &Rc<Runtime>, entry: &Entry) -> Rc<Adopted> {
        let owner = entry.item_owner.child(self.at);
        owner.with_setup(|| {
            let cx = Mcx { rt, owner };
            let mut common = region_template::instantiate(&entry.template);
            parts::reject_modal(&mut common, &cx, self.at);
            let constraints = parts::constraints(&mut common, &cx, self.at, Some(self.root.constraints));
            let mut handlers = std::mem::take(&mut common.handlers);
            let mut actions = handlers.take_part_actions();
            actions.compose(std::mem::take(&mut common.part_actions));
            let bindings = std::mem::take(&mut common.part_bindings);
            let generation = {
                let mut state = self.state.borrow_mut();
                state.next += 1;
                state.next
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
                bound: Cell::new(false),
            });
            if self.phase.get() == RegistryPhase::Mounted {
                self.bind_adopted(rt, &adopted);
            }
            adopted
        })
    }

    fn bind_adopted(&self, rt: &Rc<Runtime>, adopted: &Adopted) {
        let Some(region) = adopted.target.region() else {
            return;
        };
        if adopted.bound.replace(true) {
            return;
        }
        adopted.owner.with_setup(|| {
            let hovered = adopted
                .target
                .bindings
                .first()
                .map(|binding| binding.hovered)
                .unwrap_or_else(|| false.into_value());
            for (index, reference) in adopted.common.node_ref.iter().enumerate() {
                if !adopted.common.node_ref[..index]
                    .iter()
                    .any(|previous| previous == reference)
                {
                    reference.bind_region(rt, self.node, region, adopted.owner, hovered, self.at);
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
            if let Some(adopted) = entry.adopted.borrow().as_ref() {
                self.bind_adopted(&rt, adopted);
            }
        }
    }

    pub(super) fn resolve(&self) -> Vec<(RegionId, RegionSemantics)> {
        self.revision.get();
        let entries = self.state.borrow().active.clone();
        entries
            .iter()
            .enumerate()
            .filter_map(|(order, entry)| {
                let adopted = entry.adopted.borrow().clone()?;
                let common = &adopted.common;
                let mut properties = common
                    .semantics
                    .as_ref()
                    .map(|sources| super::semantics::resolve(sources, self.at))
                    .unwrap_or_default();
                properties.checked = common.checked.as_ref().map(kivact_reactive::Prop::get);
                properties.selected = common
                    .selected
                    .as_ref()
                    .is_some_and(kivact_reactive::Prop::get);
                properties.expanded = common.expanded.as_ref().map(kivact_reactive::Prop::get);
                properties.invalid = common
                    .invalid
                    .as_ref()
                    .is_some_and(kivact_reactive::Prop::get);
                Some((
                    entry.id,
                    RegionSemantics {
                        role: common.role.unwrap_or(crate::view::Role::Generic),
                        name: common
                            .name
                            .as_ref()
                            .map(kivact_reactive::Prop::get)
                            .unwrap_or_default(),
                        description: common.description.as_ref().map(kivact_reactive::Prop::get),
                        value: common.value.as_ref().map(kivact_reactive::Prop::get),
                        actions: adopted.target.actions.actions(),
                        properties: Some(Box::new(properties)),
                        disabled: adopted.target.constraints.disabled.get(),
                        hidden: adopted.target.constraints.hidden.get(),
                        focusable: adopted.target.constraints.focusable.get(),
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
        revision: cx.owner.signal(0u64, at),
        changed: cx.owner.signal(0u64, at),
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
            registry.changed.get();
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
