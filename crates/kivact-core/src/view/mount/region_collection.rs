//! Role: Region collections share keyed diff and item owner lifecycles.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use crate::state::Value;
use crate::view::element_data::{Common, Declaration, Entry};
use kivact_runtime::EffectKind;
use kivact_runtime::Prop as InputProp;
use kivact_runtime::keyed::{self, Diff, Plan, Rows};
use kivact_style::draw::RegionId;

/// A keyed row and the attachment entry its setup declared.
struct Row<T: 'static> {
    value: keyed::Row<RegionId, T>,
    entry: Option<Rc<Entry>>,
}

impl<T: 'static> AsRef<keyed::Row<RegionId, T>> for Row<T> {
    fn as_ref(&self) -> &keyed::Row<RegionId, T> {
        &self.value
    }
}

pub(crate) fn declaration<T: kivact_runtime::PropValue>(
    items: InputProp<Vec<T>>,
    key: impl Fn(&T) -> RegionId + 'static,
    make: impl Fn(RegionId, Value<T>) -> Common + 'static,
    at: &'static Location<'static>,
) -> Declaration {
    Declaration::Collection(Box::new(move |cx, registry, group| {
        let state = RefCell::new(Rows::<RegionId, Row<T>>::new());
        let runtime = cx.weak();
        let weak_registry = Rc::downgrade(&registry);
        let parent = cx.owner;
        cx.effect(EffectKind::Internal, at, move || {
            let (Some(_), Some(registry)) = (runtime.upgrade(), weak_registry.upgrade()) else {
                return;
            };
            let diff = kivact_runtime::pure(|| {
                items.with(|items| keyed::diff(&state.borrow(), items, &key))
            });
            match diff {
                Diff::Values(changed) => {
                    let updates = {
                        let state = state.borrow();
                        changed
                            .into_iter()
                            .map(|(index, item)| (state.rows()[index].value.item(), item))
                            .collect::<Vec<_>>()
                    };
                    for (signal, item) in updates {
                        signal.update(|slot| *slot = item);
                    }
                }
                Diff::Structure { keys, plans, dups } => {
                    for hash in dups {
                        registry.duplicate_key(hash);
                    }
                    let mut old = state
                        .borrow_mut()
                        .take()
                        .into_iter()
                        .map(Some)
                        .collect::<Vec<_>>();
                    let mut next = Vec::with_capacity(keys.len());
                    for (key, plan) in keys.into_iter().zip(plans) {
                        match plan {
                            Plan::Keep {
                                old: index,
                                changed,
                            } => {
                                if let Some(row) = old[index].take() {
                                    if let Some(item) = changed {
                                        row.value.item().update(|slot| *slot = item);
                                    }
                                    next.push(row);
                                }
                            }
                            Plan::New(item) => {
                                let Some(value) = keyed::Row::new(parent, key, item, at) else {
                                    continue;
                                };
                                let entry = Some(registry.entry(
                                    key,
                                    value.setup(at, |item| make(key, item)),
                                    value.owner(),
                                ));
                                next.push(Row { value, entry });
                            }
                        }
                    }
                    // Retire owners through the shared reconciler, after every
                    // collection settles and adopted refs and callbacks detach.
                    registry.update(
                        group,
                        next.iter().filter_map(|row| row.entry.clone()).collect(),
                        old.into_iter()
                            .flatten()
                            .map(|row| row.value.owner())
                            .collect(),
                    );
                    state.borrow_mut().replace(next);
                }
            }
        });
    }))
}
