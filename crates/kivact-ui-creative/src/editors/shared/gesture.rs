//! Role: One canvas press state and shared drag update, selection and finish order.
//!
//! Interaction DragApi owns pointer identity, capture and edit commit/cancel. This
//! state owns only the domain's press decision and opaque drag snapshot.
use super::selection::merge_marquee;
use kivact::geometry::{Point, Rect};
use kivact::interaction::{DragApi, DragPhase, DragUpdate};
use kivact::state::{Machine, MachineHandle, Step, Value, machine};
use kivact::view::{ButtonPress, MouseButton, PointerEvent, PointerPhase};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
struct Decision<T, D> {
    current: Point,
    target: Option<T>,
    payload: D,
    click: ClickIntent,
}

/// The press the canvas owns: none, a press deciding what it does, or a drag.
#[derive(Clone, PartialEq)]
enum Press<T, D> {
    Idle,
    Deciding(Decision<T, D>),
    Dragging {
        drag: D,
        current: Point,
        target: Option<T>,
    },
}

#[derive(Clone, PartialEq)]
enum ClickIntent {
    Single { selection: Option<u64> },
    Double { selection: Option<u64> },
}
impl ClickIntent {
    fn selection(&self) -> Option<u64> {
        match self {
            Self::Single { selection } | Self::Double { selection } => *selection,
        }
    }
    fn set_selection(&mut self, pending: Option<u64>) {
        match self {
            Self::Single { selection } | Self::Double { selection } => *selection = pending,
        }
    }
}

enum DecisionEvent<T, D> {
    Begin(Decision<T, D>),
    Payload(D),
    Pending(Option<u64>),
    Motion {
        current: Point,
        moved: bool,
    },
    Clear,
    InputsChanged {
        focused: bool,
        visible: bool,
        enabled: bool,
        data_valid: bool,
        rollback: Rc<dyn Fn()>,
    },
}
struct DecisionMachine<T, D>(std::marker::PhantomData<(T, D)>);
impl<T: Copy + PartialEq + 'static, D: Clone + PartialEq + 'static> Machine
    for DecisionMachine<T, D>
{
    type State = Press<T, D>;
    type Event = DecisionEvent<T, D>;
    type Effect = Rc<dyn Fn()>;
    fn transition(
        &self,
        state: &Self::State,
        event: Self::Event,
    ) -> Step<Self::State, Self::Event, Self::Effect> {
        match event {
            DecisionEvent::Begin(decision) => Step::to(Press::Deciding(decision)),
            DecisionEvent::Clear => Step::to(Press::Idle),
            DecisionEvent::InputsChanged {
                focused,
                visible,
                enabled,
                data_valid,
                rollback,
            } => {
                if !focused || !visible || !enabled || !data_valid {
                    Step::to(Press::Idle).with(rollback)
                } else {
                    Step::stay()
                }
            }
            DecisionEvent::Payload(payload) => match state {
                Press::Deciding(decision) => {
                    let mut next = decision.clone();
                    next.payload = payload;
                    Step::to(Press::Deciding(next))
                }
                Press::Dragging {
                    current, target, ..
                } => Step::to(Press::Dragging {
                    drag: payload,
                    current: *current,
                    target: *target,
                }),
                Press::Idle => Step::stay(),
            },
            DecisionEvent::Pending(pending) => {
                if let Press::Deciding(decision) = state {
                    let mut next = decision.clone();
                    next.click.set_selection(pending);
                    Step::to(Press::Deciding(next))
                } else {
                    Step::stay()
                }
            }
            DecisionEvent::Motion { current, moved } => match state {
                Press::Deciding(decision) if moved => Step::to(Press::Dragging {
                    drag: decision.payload.clone(),
                    current,
                    target: decision.target,
                }),
                Press::Deciding(decision) => {
                    let mut next = decision.clone();
                    next.current = current;
                    Step::to(Press::Deciding(next))
                }
                Press::Dragging { drag, target, .. } => Step::to(Press::Dragging {
                    drag: drag.clone(),
                    current,
                    target: *target,
                }),
                Press::Idle => Step::stay(),
            },
        }
    }
}

pub(crate) struct CanvasGesture<
    T: Copy + PartialEq + 'static,
    D: Clone + PartialEq + 'static,
    S: Clone + PartialEq + 'static,
> {
    controller: MachineHandle<DecisionMachine<T, D>>,
    snapshot: Value<Option<S>>,
}
impl<T: Copy + PartialEq + 'static, D: Clone + PartialEq + 'static, S: Clone + PartialEq + 'static>
    Copy for CanvasGesture<T, D, S>
{
}
impl<T: Copy + PartialEq + 'static, D: Clone + PartialEq + 'static, S: Clone + PartialEq + 'static>
    Clone for CanvasGesture<T, D, S>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Copy + PartialEq + 'static, D: Clone + PartialEq + 'static, S: Clone + PartialEq + 'static>
    CanvasGesture<T, D, S>
{
    pub(crate) fn editor(
        edit: kivact::interaction::EditSession<S>,
        empty: impl Fn() -> S + 'static,
    ) -> (Self, DragApi<S>) {
        let drag = kivact::interaction::use_drag::<S>(kivact::interaction::DragCoordinates::Local)
            .slop(0.0)
            .buttons([MouseButton::Middle]);
        let press = Self::new(drag);
        let drag = drag.edit_on_change(edit, move || press.initial().unwrap_or_else(&empty));
        (press, drag)
    }
    pub(crate) fn new(drag: DragApi<S>) -> Self {
        Self {
            controller: machine(
                Press::Idle,
                || DecisionMachine(std::marker::PhantomData),
                |_, rollback| rollback(),
            ),
            snapshot: drag.active_payload(),
        }
    }
    pub(crate) fn begin(
        self,
        event: &PointerEvent,
        target: Option<T>,
        payload: D,
        pending: Option<u64>,
    ) {
        let double = matches!(
            event.phase(),
            PointerPhase::Down(ButtonPress {
                button: MouseButton::Primary,
                clicks: 2
            })
        );
        self.controller.send(DecisionEvent::Begin(Decision {
            current: event.pos(),
            target,
            payload,
            click: if double {
                ClickIntent::Double { selection: pending }
            } else {
                ClickIntent::Single { selection: pending }
            },
        }));
    }

    pub(crate) fn inputs_changed(
        self,
        focused: bool,
        visible: bool,
        enabled: bool,
        data_valid: bool,
        rollback: impl Fn() + 'static,
    ) {
        self.controller.send(DecisionEvent::InputsChanged {
            focused,
            visible,
            enabled,
            data_valid,
            rollback: Rc::new(rollback),
        });
    }
    pub(crate) fn with_payload<R>(self, read: impl FnOnce(&D) -> R) -> Option<R> {
        self.controller.state().with(|state| match state {
            Press::Deciding(decision) => Some(read(&decision.payload)),
            Press::Dragging { drag, .. } => Some(read(drag)),
            Press::Idle => None,
        })
    }
    pub(crate) fn update_payload(self, change: impl FnOnce(&mut D)) {
        if let Some(mut payload) = self.with_payload(Clone::clone) {
            change(&mut payload);
            self.controller.send(DecisionEvent::Payload(payload));
        }
    }

    pub(crate) fn pending(self) -> Option<u64> {
        self.controller.state().with(|state| match state {
            Press::Deciding(decision) => decision.click.selection(),
            Press::Idle | Press::Dragging { .. } => None,
        })
    }
    pub(crate) fn set_pending(self, pending: Option<u64>) {
        self.controller.send(DecisionEvent::Pending(pending));
    }

    pub(crate) fn initial(self) -> Option<S> {
        self.snapshot.get()
    }
    pub(crate) fn current(self) -> Point {
        self.controller.state().with(|state| match state {
            Press::Deciding(decision) => decision.current,
            Press::Dragging { current, .. } => *current,
            Press::Idle => Point::new(0., 0.),
        })
    }
    pub(crate) fn pressed(self) -> Option<T> {
        self.controller.state().with(|state| match state {
            Press::Deciding(decision) => decision.target,
            Press::Dragging { target, .. } => *target,
            Press::Idle => None,
        })
    }
    pub(crate) fn moved(self) -> bool {
        self.controller
            .state()
            .with(|state| matches!(state, Press::Dragging { .. }))
    }
    pub(crate) fn update(
        self,
        pos: Point,
        origin: Point,
        snapshot: &S,
        motion: impl FnOnce(Point, Point, &S),
    ) {
        self.controller.send(DecisionEvent::Motion {
            current: pos,
            moved: pos != origin,
        });
        motion(pos, origin, snapshot);
    }
    fn clear(self) {
        self.controller.send(DecisionEvent::Clear);
    }
    pub(crate) fn cancel(self, reset: impl FnOnce()) {
        reset();
        self.clear();
    }
    pub(crate) fn finish(
        self,
        update: &DragUpdate<S>,
        marquee: impl Fn() -> bool,
        selection: &(impl Fn() -> Vec<u64>, impl Fn(Vec<u64>), impl Fn(u64)),
        callbacks: &(
            impl Fn(Point, Point, &S),
            impl Fn(Rect) -> Vec<u64>,
            impl Fn(T),
            impl Fn(bool),
        ),
    ) {
        self.update(update.pos, update.start_pos, &update.payload, &callbacks.0);
        let activation = self.controller.state().with(|state| match state {
            Press::Deciding(decision) if matches!(decision.click, ClickIntent::Double { .. }) => {
                decision.target
            }
            _ => None,
        });
        if marquee() {
            let hit = callbacks.1(Rect::from_points(update.start_pos, self.current()));
            selection.1(merge_marquee(&selection.0(), hit, update.modifiers));
        }
        if !self.moved()
            && let Some(id) = self.pending()
        {
            selection.2(id);
        }
        callbacks.3(true);
        self.clear();
        if let Some(target) = activation {
            callbacks.2(target);
        }
    }
    /// Every domain keeps its own on_start, coordinates and payload operations.
    /// `reset(true)` finalizes domain-only work; `reset(false)` cancels it.
    pub(crate) fn install(
        self,
        drag: DragApi<S>,
        marquee: impl Fn() -> bool + 'static,
        selection: (
            impl Fn() -> Vec<u64> + 'static,
            impl Fn(Vec<u64>) + 'static,
            impl Fn(u64) + 'static,
        ),
        callbacks: (
            impl Fn(Point, Point, &S) + 'static,
            impl Fn(Rect) -> Vec<u64> + 'static,
            impl Fn(T) + 'static,
            impl Fn(bool) + 'static,
        ),
    ) -> DragApi<S> {
        let hooks = Rc::new((marquee, selection, callbacks));
        let updates = hooks.clone();
        drag.on_update(move |update: DragUpdate<S>| {
            self.update(update.pos, update.start_pos, &update.payload, &updates.2.0)
        })
        .on_end(move |update: DragUpdate<S>| {
            if update.phase == DragPhase::End {
                self.finish(&update, &hooks.0, &hooks.1, &hooks.2);
            } else {
                self.cancel(|| hooks.2.3(false));
            }
        })
    }
}
