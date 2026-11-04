//! Role: Opt-in direct manipulation for a standard switch.
use crate::values::{ControlSize, SwitchDragState};
use kivact_core::draw::Point;
use kivact_core::primitive::{Part, clock_now};
use kivact_core::style::{Theme, ThemeExtension, current_theme};
use kivact_core::view::{
    Handled, MouseButton, PointerEvent, PointerId, PointerKind, PointerPhase, ReadingDirection,
    current_direction,
};
use kivact_core::{Binding, Signal, Value, Var, signal, var};
use kivact_interaction::ToggleState;
use std::time::Duration;

/// Whether switches in this theme use direct manipulation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SwitchDrag(pub bool);

impl ThemeExtension for SwitchDrag {
    fn fallback(_: &Theme) -> Self {
        Self(false)
    }
}

#[derive(Clone, Copy)]
struct Contact {
    id: PointerId,
    origin: Point,
    started_on: bool,
    visually_on: bool,
    armed: bool,
    flipped: bool,
    horizontal: bool,
    vertical: bool,
    touch: bool,
    started: u64,
}

#[derive(Clone, Copy)]
struct Geometry {
    width: f64,
    height: f64,
    off_side: f64,
    on_side: f64,
    off_inset: f64,
    sign: f64,
}

impl Geometry {
    fn new(size: ControlSize) -> Self {
        let sign = if current_direction() == ReadingDirection::Rtl {
            -1.0
        } else {
            1.0
        };
        if matches!(size, ControlSize::ExtraSmall | ControlSize::Small) {
            Self {
                width: 32.0,
                height: 20.0,
                off_side: 14.0,
                on_side: 16.0,
                off_inset: 3.0,
                sign,
            }
        } else {
            Self {
                width: 40.0,
                height: 24.0,
                off_side: 16.0,
                on_side: 20.0,
                off_inset: 4.0,
                sign,
            }
        }
    }

    fn travel(self) -> f64 {
        self.width - 2.0 - self.on_side / 2.0 - self.off_inset - self.off_side / 2.0
    }

    fn band(self, raw: f64, on: bool) -> f64 {
        let end = self.travel();
        let inset = if on { 2.0 } else { self.off_inset };
        let dimension = (12.0 * self.height / 28.0).min(inset / 2.0);
        let clamped = raw.clamp(0.0, end);
        let excess = raw - clamped;
        clamped
            + excess.signum() * (1.0 - 1.0 / (excess.abs() * 0.55 / dimension + 1.0)) * dimension
    }
}

#[derive(Clone, Copy)]
struct Gesture {
    value: Binding<bool>,
    geometry: Geometry,
    contact: Var<Option<Contact>>,
    visual: Signal<Option<SwitchDragState>>,
    generation: Signal<u64>,
}

pub(super) type Visual = Option<Value<Option<SwitchDragState>>>;

pub(super) fn switch(
    control: Part<ToggleState>,
    value: Binding<bool>,
    size: ControlSize,
) -> (Part<ToggleState>, Visual) {
    if !current_theme().get::<SwitchDrag>().0 {
        return (control, None);
    }
    let gesture = Gesture {
        value,
        geometry: Geometry::new(size),
        contact: var(None),
        visual: signal(None),
        generation: signal(0),
    };
    let state = control.state();
    kivact_core::effect(move || {
        if state.get().input.disabled {
            gesture.contact.set(None);
            gesture.visual.set(None);
            gesture.generation.update(|id| *id = id.wrapping_add(1));
        }
    });
    let control = control.on_pointer(move |event| gesture.pointer(event));
    (control, Some(gesture.visual.into()))
}

impl Gesture {
    fn pointer(self, event: PointerEvent) -> Handled {
        let touch = event.kind() == PointerKind::Touch;
        if touch && !event.is_primary() {
            return Handled::Yes;
        }
        if matches!(event.phase(), PointerPhase::Up(press) if press.button != MouseButton::Primary)
        {
            return Handled::No;
        }
        if matches!(event.phase(), PointerPhase::Down(press) if press.button == MouseButton::Primary)
            && event.is_primary()
        {
            let started_on = self.value.get_untracked();
            self.generation.update(|id| *id = id.wrapping_add(1));
            self.contact.set(Some(Contact {
                id: event.id(),
                origin: event.window_pos(),
                started_on,
                visually_on: started_on,
                armed: false,
                flipped: false,
                horizontal: false,
                vertical: false,
                touch,
                started: clock_now(),
            }));
            self.visual.set(touch.then_some(SwitchDragState {
                offset: None,
                visually_on: started_on,
                touch,
                tracking: false,
                lifted: true,
                released: false,
            }));
            if touch {
                event.suppress_activation();
                return Handled::Yes;
            }
            return Handled::No;
        }
        let Some(mut contact) = self
            .contact
            .get()
            .filter(|contact| contact.id == event.id())
        else {
            if !touch
                && matches!(event.phase(), PointerPhase::Enter | PointerPhase::Move)
                && self
                    .visual
                    .get_untracked()
                    .is_some_and(|visual| visual.touch)
            {
                self.generation.update(|id| *id = id.wrapping_add(1));
                self.visual.set(None);
            }
            return Handled::No;
        };
        match event.phase() {
            PointerPhase::Cancel => {
                self.contact.set(None);
                self.visual.set(None);
                Handled::No
            }
            PointerPhase::Move | PointerPhase::Up(_) => {
                let dx = (event.window_pos().x - contact.origin.x) * self.geometry.sign;
                let dy = event.window_pos().y - contact.origin.y;
                if !contact.horizontal && !contact.vertical {
                    if dx.abs() >= 10.0 && dx.abs() > dy.abs() {
                        contact.horizontal = true;
                        event.suppress_activation();
                    } else if dy.abs() >= 10.0 && dy.abs() >= dx.abs() {
                        contact.vertical = true;
                        self.visual.set(None);
                    }
                }
                if contact.horizontal {
                    let toward = if contact.started_on { -dx } else { dx };
                    let side = if contact.started_on {
                        self.geometry.on_side
                    } else {
                        self.geometry.off_side
                    };
                    let past = toward >= self.geometry.width - side;
                    if contact.visually_on != contact.started_on {
                        if toward < 0.0 {
                            contact.visually_on = contact.started_on;
                            contact.armed = false;
                        }
                    } else if contact.armed && past {
                        contact.visually_on = !contact.started_on;
                        contact.flipped = true;
                        contact.armed = false;
                    } else {
                        contact.armed = past;
                    }
                }
                if matches!(event.phase(), PointerPhase::Up(_)) {
                    self.contact.set(None);
                    if contact.vertical {
                        self.visual.set(None);
                        return if contact.touch {
                            Handled::Yes
                        } else {
                            Handled::No
                        };
                    }
                    if contact.touch || contact.horizontal {
                        let next = if contact.flipped {
                            contact.visually_on
                        } else {
                            !contact.started_on
                        };
                        if self.value.get_untracked() != next {
                            self.value.set(next);
                        }
                        self.settle(contact, next);
                        return Handled::Yes;
                    }
                    return Handled::No;
                }
                if contact.horizontal {
                    let start = if contact.started_on {
                        self.geometry.travel()
                    } else {
                        0.0
                    };
                    self.visual.set(Some(SwitchDragState {
                        offset: contact.touch.then(|| {
                            self.geometry.band(start + dx, contact.visually_on) * self.geometry.sign
                        }),
                        visually_on: contact.visually_on,
                        touch: contact.touch,
                        tracking: true,
                        lifted: contact.touch,
                        released: false,
                    }));
                }
                self.contact.set(Some(contact));
                if contact.horizontal {
                    Handled::Yes
                } else {
                    Handled::No
                }
            }
            _ => Handled::No,
        }
    }

    fn settle(self, contact: Contact, on: bool) {
        if !contact.touch {
            self.visual.set(None);
            return;
        }
        self.visual.set(Some(SwitchDragState {
            offset: None,
            visually_on: on,
            touch: true,
            tracking: false,
            lifted: true,
            released: true,
        }));
        let generation = self.generation.get_untracked();
        let elapsed = Duration::from_nanos(clock_now().saturating_sub(contact.started));
        kivact_core::task::spawn(async move {
            kivact_core::task::sleep(Duration::from_millis(235).saturating_sub(elapsed)).await;
            if self.generation.get_untracked() == generation {
                self.visual.set(Some(SwitchDragState {
                    offset: None,
                    visually_on: on,
                    touch: true,
                    tracking: false,
                    lifted: false,
                    released: true,
                }));
            }
        });
    }
}
