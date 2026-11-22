//! Role: The switch drag handle: the gesture session and the control part.

use crate::{EditSession, InteractionPolicy, ToggleState};
use kivact_core::custom::{Part, clock_now};
use kivact_core::state::{IntoValue, MachineHandle, Signal, Value, machine, memo};
use kivact_core::style::use_theme;
use kivact_core::view::{
    EditPhase, Handled, HandlerResult, MouseButton, PointerKind, PointerPhase, ReadingDirection,
    use_direction,
};

use super::machine::{self, Event, State, SwitchGesture};
use super::types::{SwitchTrack, ThumbDrag};

/// A switch's drag: the thumb follows a touch or pointer along the track
/// and the release commits one value. Inert when the theme's
/// [`InteractionPolicy`] leaves `switch.drag` off.
#[derive(Clone, Copy)]
pub struct SwitchDragApi {
    session: Option<MachineHandle<machine::Machine>>,
    edit: EditSession<bool>,
    thumb: Value<Option<ThumbDrag>>,
}

/// Create the drag of a switch that edits `value` on `track`. Setup only.
#[track_caller]
pub fn use_switch_drag(value: Signal<bool>, track: impl IntoValue<SwitchTrack>) -> SwitchDragApi {
    let track = track.into_value();
    let edit = EditSession::new(move |initial| value.set(initial));
    let enabled = use_theme().get().ext::<InteractionPolicy>().switch.drag;
    if !enabled {
        return SwitchDragApi {
            session: None,
            edit,
            thumb: memo(|| None),
        };
    }
    let sign = if use_direction() == ReadingDirection::Rtl {
        -1.0
    } else {
        1.0
    };
    let session = machine(
        State::Gesture(SwitchGesture::Idle),
        move || machine::Machine {
            value: value.get_untracked(),
            track: track.get(),
            sign,
            now: clock_now(),
        },
        move |_, effect| match effect {
            machine::Effect::Set(next) => edit.once(value.get_untracked(), || value.set(next)),
            machine::Effect::Suppress(event) => event.suppress_activation(),
        },
    );
    let thumb = memo(move || session.get().visual());
    SwitchDragApi {
        session: Some(session),
        edit,
        thumb,
    }
}

impl SwitchDragApi {
    /// Whether the theme turned the drag on.
    pub fn is_enabled(self) -> bool {
        self.session.is_some()
    }
    /// Where the thumb is during a drag; `None` while no drag runs.
    pub fn thumb(self) -> Value<Option<ThumbDrag>> {
        self.thumb
    }
    /// Report each drag's `Begin` and `Commit`. Setup only.
    pub fn on_edit<R: HandlerResult>(self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.edit.on_edit(f);
        self
    }
    /// Add the drag to a switch's control part (`SwitchApi::control()`);
    /// returns `control` unchanged when the drag is off. Setup only.
    #[track_caller]
    pub fn control(self, control: Part<ToggleState>) -> Part<ToggleState> {
        let Some(session) = self.session else {
            return control;
        };
        let state = control.state();
        session.feed(move || state.get().input.is_disabled(), Event::Disabled);
        Part::new(state, move |host| {
            host.part(control).on_pointer(move |event| {
                let touch = event.kind() == PointerKind::Touch;
                if touch && !event.is_primary() {
                    return Handled::Yes;
                }
                let before = session.get();
                session.send(Event::Pointer(event));
                let after = session.get();
                let handled = match event.phase() {
                    PointerPhase::Down(press) => {
                        touch && event.is_primary() && press.button == MouseButton::Primary
                    }
                    PointerPhase::Move => after.handles(event),
                    PointerPhase::Up(press) if press.button == MouseButton::Primary => {
                        before.handles(event)
                            || (touch && before.tracks(event))
                            || (before.tracks(event) && matches!(after, State::Released { .. }))
                    }
                    _ => false,
                };
                if handled { Handled::Yes } else { Handled::No }
            })
        })
    }
}
