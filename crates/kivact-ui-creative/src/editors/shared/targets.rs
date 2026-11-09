//! Role: Stable private region allocation independent of application identities.

use kivact::UiError;
use kivact::custom::Generation;
use kivact::draw::RegionId;
use kivact::state::callback;
use kivact::state::{Machine, MachineHandle, Step, Value, effect, machine, memo};
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
struct TargetState<T: kivact::state::ItemKey, A> {
    table: TargetTable<T>,
    active: Option<A>,
    result: Result<(), UiError>,
}
enum TargetEvent<T, A> {
    Synchronize(Result<Vec<T>, UiError>),
    Activate(Option<A>),
}
struct TargetController<T: kivact::state::ItemKey, A: kivact::state::PropValue>(
    std::marker::PhantomData<(T, A)>,
);
impl<T: kivact::state::ItemKey, A: kivact::state::PropValue> Machine for TargetController<T, A> {
    type State = TargetState<T, A>;
    type Event = TargetEvent<T, A>;
    type Effect = UiError;
    fn transition(
        &self,
        state: &Self::State,
        event: Self::Event,
    ) -> Step<Self::State, Self::Event, Self::Effect> {
        match event {
            TargetEvent::Activate(active) => {
                if state.active == active {
                    Step::stay()
                } else {
                    Step::to(TargetState {
                        active,
                        ..state.clone()
                    })
                }
            }
            TargetEvent::Synchronize(source) => {
                let mut table = state.table.clone();
                match source.and_then(|targets| table.synchronize(targets)) {
                    Ok(_) => Step::to(TargetState {
                        table,
                        active: state.active.clone(),
                        result: Ok(()),
                    }),
                    Err(error) => Step::to(TargetState {
                        table: state.table.clone(),
                        active: state.active.clone(),
                        result: Err(error.clone()),
                    })
                    .with(error),
                }
            }
        }
    }
}

/// The targets of one editor: the live scene and the target the keyboard acts on.
pub(crate) struct Targets<T: kivact::state::ItemKey, A: kivact::state::PropValue> {
    controller: MachineHandle<TargetController<T, A>>,
}

impl<T: kivact::state::ItemKey, A: kivact::state::PropValue> Clone for Targets<T, A> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: kivact::state::ItemKey, A: kivact::state::PropValue> Copy for Targets<T, A> {}

impl<T: kivact::state::ItemKey, A: kivact::state::PropValue> Targets<T, A> {
    /// The target scene, or the error of the last source.
    pub(crate) fn scene(self) -> Value<Result<Rc<Scene<T>>, UiError>> {
        let state = self.controller.state();
        memo(move || state.with(|state| state.result.clone().map(|()| state.table.scene.clone())))
    }

    /// The target the keyboard acts on; any live target or none is valid.
    pub(crate) fn active(self) -> kivact::state::Binding<Option<A>> {
        let state = self.controller.state();
        let controller = self.controller;
        kivact::state::binding(
            move || state.with(|state| state.active.clone()),
            move |active| controller.send(TargetEvent::Activate(active)),
        )
    }
}

pub(crate) fn targets<T: kivact::state::ItemKey, A: kivact::state::PropValue>(
    source: impl Fn() -> Result<Vec<T>, UiError> + 'static,
) -> Targets<T, A> {
    let report = callback(|error| -> kivact::Result<()> { Err(error) });
    let controller = machine(
        TargetState {
            table: TargetTable::default(),
            active: None,
            result: Ok(()),
        },
        || TargetController(std::marker::PhantomData),
        move |_, error| report.call(error),
    );
    effect(move || controller.send(TargetEvent::Synchronize(source())));
    Targets { controller }
}
