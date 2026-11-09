//! Role: Attach the Space modifier machine to focused surfaces.
use super::machine;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::{MachineHandle, Signal, Value, machine, memo, signal};
use kivact_core::view::{Handled, Key, KeyEvent, NodeRef, PointerEvent, PointerPhase};

/// Whether Space is held on a focused surface.
///
/// ```no_run
/// use kivact_core::view::{Element, node};
/// use kivact_interaction::use_space_pan;
/// let space = use_space_pan();
/// let canvas = node().part(space.control());
/// let panning = space.armed();
/// ```
#[derive(Clone, Copy)]
pub struct SpacePanApi {
    observed: Signal<Option<Value<PartState>>>,
    held: MachineHandle<machine::Machine>,
    armed: Value<bool>,
}

/// Create a Space tracker. Setup only.
#[track_caller]
pub fn use_space_pan() -> SpacePanApi {
    let held = machine(
        machine::SpaceKey::Up,
        || machine::Machine,
        |_, effect| match effect {},
    );
    SpacePanApi {
        observed: signal(None),
        held,
        armed: memo(move || held.get() == machine::SpaceKey::Held),
    }
}

impl SpacePanApi {
    /// Drawing state of the actual attached input target.
    pub fn control_state(self) -> Value<PartState> {
        kivact_core::state::memo(move || {
            self.observed
                .get()
                .map(|state| state.get())
                .unwrap_or_default()
        })
    }

    /// Whether Space is held while the surface has the focus (tracked). It
    /// turns false on the key's release, when the focus leaves the surface,
    /// when the surface is disabled or hidden and on a pointer `Cancel`.
    pub fn armed(self) -> Value<bool> {
        self.armed
    }

    /// The part on the focusable surface (its canvas regions' unhandled keys
    /// included): Space presses and releases update [`armed`](Self::armed) and
    /// return `Handled::PreventDefault`, so the focused control is not
    /// activated and ancestors still see them. Attach one control. Setup only.
    #[track_caller]
    pub fn control(self) -> Part<PartState> {
        let input = part(|input| input);
        let part_state = input.state();
        self.observed.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            let surface = host.part(input).node_ref(target).region_events();
            self.watch(target, part_state);
            surface
                .on_key(move |event: KeyEvent| self.space(&event, true))
                .on_key_up(move |event: KeyEvent| self.space(&event, false))
                .on_pointer(move |event: PointerEvent| {
                    if event.phase() == PointerPhase::Cancel {
                        self.disarm();
                    }
                    Handled::No
                })
        })
    }

    fn space(self, event: &KeyEvent, down: bool) -> Handled {
        if self.key(event, down) {
            Handled::PreventDefault
        } else {
            Handled::No
        }
    }

    /// Update `held` for a key event; whether it was Space.
    pub(crate) fn key(self, event: &KeyEvent, down: bool) -> bool {
        let is_space = event.key() == Key::Space;
        if is_space {
            self.held.send(machine::Event::Key(down));
        }
        is_space
    }

    pub(crate) fn disarm(self) {
        self.held.send(machine::Event::Disarm);
    }

    /// Disarm when `surface` loses the focus within and when `state` is
    /// disabled or hidden. Setup only.
    #[track_caller]
    pub(crate) fn watch(self, surface: NodeRef, state: Value<PartState>) {
        let focus = surface.focus_within();
        self.held.feed(
            memo(move || {
                let state = state.get();
                focus.get() && !state.disabled && !state.hidden
            }),
            machine::Event::Available,
        );
    }
}
