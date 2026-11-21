//! Role: Dismiss a surface by swiping it toward its edge (a toast card, a drawer).
mod machine;
use crate::DrawerSwipe;
use kivact_core::custom::{Part, PartState, clock_now, part};
use kivact_core::geometry::{Edge, Rect};
use kivact_core::state::{Callback, IntoHandler, MachineHandle, Value, machine, memo};
use kivact_core::view::{Handled, PointerEvent, PointerKind, PointerPhase};
use machine::{Event, Swipe};

/// Which contacts a swipe follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeInputs {
    All,
    TouchAndPen,
}
impl SwipeInputs {
    fn accepts(self, kind: PointerKind) -> bool {
        self == Self::All || matches!(kind, PointerKind::Touch | PointerKind::Pen)
    }
}
/// How a swipe toward `edge` is recognised and when its release dismisses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwipeDismissPolicy {
    /// The edge the surface leaves through.
    pub edge: Edge,
    pub inputs: SwipeInputs,
    /// Logical pixels a contact moves before it is a swipe or not.
    pub slop: f64,
    /// The share of the surface's size along the edge's axis that dismisses.
    pub close_fraction: f64,
    /// The least travel that dismisses by distance.
    pub min_distance: f64,
    /// The least travel of a flick.
    pub flick_distance: f64,
    /// Logical pixels per millisecond toward the edge that make a flick.
    pub flick_speed: f64,
}
impl SwipeDismissPolicy {
    /// A toast card's swipe: touch and pen only, 40 % of its size or 48 px,
    /// or a flick of 48 px faster than 1.2 px/ms.
    pub fn toast(edge: Edge, slop: f64) -> Self {
        Self {
            edge,
            inputs: SwipeInputs::TouchAndPen,
            slop,
            close_fraction: 0.4,
            min_distance: 48.0,
            flick_distance: 48.0,
            flick_speed: 1.2,
        }
    }
    /// A drawer's handle: every pointer, the theme's [`DrawerSwipe`].
    pub fn drawer(edge: Edge, swipe: DrawerSwipe) -> Self {
        Self {
            edge,
            inputs: SwipeInputs::All,
            slop: 0.0,
            close_fraction: swipe.close_fraction,
            min_distance: 0.0,
            flick_distance: swipe.flick_distance,
            flick_speed: swipe.flick_speed,
        }
    }
    pub(crate) fn dismisses(&self, travel: f64, size: f64, speed: f64) -> bool {
        travel >= (size * self.close_fraction).max(self.min_distance)
            || (travel >= self.flick_distance && speed > self.flick_speed)
    }
}
/// Where a swipe is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwipePhase {
    Rest,
    /// A contact is down and has not become a swipe yet.
    Pressed,
    /// The surface follows the contact `offset` px toward its edge; `size` is its extent on that axis.
    Dragging {
        offset: f64,
        size: f64,
    },
    /// A release dismissed the surface `travel` px from its place.
    Dismissed {
        travel: f64,
        size: f64,
    },
}
#[derive(Clone, Copy)]
pub struct SwipeDismissApi {
    session: MachineHandle<machine::Machine>,
    inputs: SwipeInputs,
    phase: Value<SwipePhase>,
    held: Value<bool>,
}
/// Create a swipe that calls `on_dismiss` once a release qualifies.
/// `rect` is the surface's rectangle. Setup only.
#[track_caller]
pub fn use_swipe_dismiss<M>(
    policy: SwipeDismissPolicy,
    rect: Value<Option<Rect>>,
    on_dismiss: impl IntoHandler<(), M>,
) -> SwipeDismissApi {
    let on_dismiss = Callback::from_handler(on_dismiss.into_handler());
    let session = machine(
        Swipe::Rest,
        move || machine::Machine {
            policy,
            rect: rect.get(),
            now: clock_now(),
        },
        move |_, machine::Effect::Dismiss| on_dismiss.call(()),
    );
    let phase = memo(move || session.get().phase());
    let held = memo(move || !matches!(session.get().phase(), SwipePhase::Rest));
    SwipeDismissApi {
        session,
        inputs: policy.inputs,
        phase,
        held,
    }
}
impl SwipeDismissApi {
    pub fn phase(self) -> Value<SwipePhase> {
        self.phase
    }
    /// True from a press through the drag and a dismissing release (a toast holds its timer).
    pub fn is_held(self) -> Value<bool> {
        self.held
    }
    /// The part the contact presses and moves: the toast card or the drawer's handle. Setup only.
    #[track_caller]
    pub fn control(self) -> Part<PartState> {
        let input = part(|input| input);
        Part::new(input.state(), move |host| {
            host.part(input)
                .on_pointer(move |event: PointerEvent| self.pointer(&event))
        })
    }
    /// A part for a button inside the surface: its press never starts a swipe. Setup only.
    #[track_caller]
    pub fn protect(self) -> Part<PartState> {
        let input = part(|input| input);
        Part::new(input.state(), move |host| {
            host.part(input).on_pointer(move |event: PointerEvent| {
                if matches!(event.phase(), PointerPhase::Down(_)) {
                    self.session.send(Event::Protect(event.id()));
                }
                Handled::No
            })
        })
    }
    fn pointer(self, event: &PointerEvent) -> Handled {
        let before = self.session.state().get_untracked();
        self.session.send(Event::Pointer(*event));
        let after = self.session.state().get_untracked();
        let pos = event.window_pos();
        if !self.inputs.accepts(event.kind())
            || !pos.x.is_finite()
            || !pos.y.is_finite()
            || matches!(before, Swipe::Released { .. })
        {
            return Handled::No;
        }
        let mine = |swipe: &Swipe| matches!(swipe, Swipe::Dragging { contact, .. } if contact.pointer == event.id());
        match event.phase() {
            PointerPhase::Down(_) => {
                if matches!(&after, Swipe::Pressed(contact) if contact.pointer == event.id()) {
                    event.capture();
                }
                Handled::No
            }
            PointerPhase::Move if mine(&after) => Handled::Yes,
            PointerPhase::Up(_) if mine(&before) => Handled::Yes,
            _ => Handled::No,
        }
    }
}
