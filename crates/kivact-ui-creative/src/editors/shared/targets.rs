//! Role: Stable private region allocation independent of application identities.

use kivact::UiError;
use kivact::custom::Generation;
use kivact::draw::RegionId;
use kivact::state::callback;
use kivact::state::{Machine, Step, Value, effect, machine, memo};
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

#[derive(Clone, PartialEq)]
struct RegionAllocator(Generation);
impl RegionAllocator {
    fn allocate(&mut self) -> RegionId {
        RegionId(self.0.next().as_u64())
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct TargetTable<T: Eq + Hash + Clone> {
    allocator: RegionAllocator,
    scene: Rc<Scene<T>>,
}

impl<T: Eq + Hash + Clone> Default for TargetTable<T> {
    fn default() -> Self {
        Self {
            allocator: RegionAllocator(Generation::new()),
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
                None => self.allocator.allocate(),
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

#[derive(Clone, PartialEq)]
struct TargetState<T: kivact::state::ItemKey> {
    table: TargetTable<T>,
    result: Result<(), UiError>,
}
struct TargetController<T>(std::marker::PhantomData<T>);
impl<T: kivact::state::ItemKey> Machine for TargetController<T> {
    type State = TargetState<T>;
    type Event = Result<Vec<T>, UiError>;
    type Effect = UiError;
    fn transition(
        &self,
        state: &Self::State,
        source: Self::Event,
    ) -> Step<Self::State, Self::Event, Self::Effect> {
        let mut table = state.table.clone();
        match source.and_then(|targets| table.synchronize(targets)) {
            Ok(_) => Step::to(TargetState {
                table,
                result: Ok(()),
            }),
            Err(error) => Step::to(TargetState {
                table: state.table.clone(),
                result: Err(error.clone()),
            })
            .with(error),
        }
    }
}

pub(crate) fn targets<T: kivact::state::ItemKey>(
    source: impl Fn() -> Result<Vec<T>, UiError> + 'static,
) -> Value<Result<Rc<Scene<T>>, UiError>> {
    let report = callback(|error| -> kivact::Result<()> { Err(error) });
    let controller = machine(
        TargetState {
            table: TargetTable::default(),
            result: Ok(()),
        },
        || TargetController(std::marker::PhantomData),
        move |_, error| report.call(error),
    );
    effect(move || controller.send(source()));
    let state = controller.state();
    memo(move || state.with(|state| state.result.clone().map(|()| state.table.scene.clone())))
}
