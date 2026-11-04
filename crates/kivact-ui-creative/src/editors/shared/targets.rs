//! Role: Stable private region allocation independent of application identities.

use kivact::UiError;
use kivact::draw::RegionId;
use kivact::state::Value;
use kivact::state::callback;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub(crate) struct Scene<T: Eq + Hash> {
    ids: HashMap<T, RegionId>,
    targets: HashMap<RegionId, T>,
}

impl<T: Eq + Hash> Scene<T> {
    pub(crate) fn live(&self, target: &T) -> bool {
        self.ids.contains_key(target)
    }

    pub(crate) fn id(&self, target: &T) -> Option<RegionId> {
        self.ids.get(target).copied()
    }

    pub(crate) fn target(&self, id: RegionId) -> Option<&T> {
        self.targets.get(&id)
    }
}

pub(crate) struct TargetTable<T: Eq + Hash + Clone> {
    next: u64,
    scene: Rc<Scene<T>>,
}

impl<T: Eq + Hash + Clone> Default for TargetTable<T> {
    fn default() -> Self {
        Self {
            next: 1,
            scene: Rc::new(Scene {
                ids: HashMap::new(),
                targets: HashMap::new(),
            }),
        }
    }
}

impl<T: Eq + Hash + Clone> TargetTable<T> {
    fn synchronize(&mut self, targets: Vec<T>) -> Result<Rc<Scene<T>>, UiError> {
        let mut unique = HashSet::new();
        let mut ids = HashMap::new();
        let mut reverse = HashMap::new();
        for target in targets {
            if !unique.insert(target.clone()) {
                return Err(UiError::msg("duplicate editor target"));
            }
            let id = match self.scene.ids.get(&target) {
                Some(id) => *id,
                None => {
                    let id = RegionId(self.next);
                    self.next = self
                        .next
                        .checked_add(1)
                        .ok_or_else(|| UiError::msg("editor region IDs exhausted"))?;
                    id
                }
            };
            ids.insert(target.clone(), id);
            reverse.insert(id, target);
        }
        self.scene = Rc::new(Scene {
            ids,
            targets: reverse,
        });
        Ok(self.scene.clone())
    }
}

pub(crate) fn targets<T: kivact::state::ItemKey>(
    source: impl Fn() -> Result<Vec<T>, UiError> + 'static,
) -> Value<Result<Rc<Scene<T>>, UiError>> {
    let table = kivact::state::var(TargetTable::default());
    let scene = kivact::state::memo(move || {
        table.update(|table| source().and_then(|targets| table.synchronize(targets)))
    });
    let report = callback(|error| -> kivact::Result<()> { Err(error) });
    kivact::state::effect(move || {
        if let Err(error) = scene.get() {
            report.call(error);
        }
    });
    scene
}
