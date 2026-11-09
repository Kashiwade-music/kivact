//! Role: Attach zoom and pan machines to the caller's viewport.
use super::data::{ViewTransform, ZoomPanOptions, finite, valid_scale};
use super::machine;
use crate::EditSession;
use crate::options_cell::OptionsCell;
use kivact_core::ErrorKind;
use kivact_core::custom::{Part, PartState, part, raise};
use kivact_core::geometry::{Point, Rect, Size};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, MachineHandle, Signal, Value, machine, memo, signal};
use kivact_core::view::EditPhase;
use kivact_core::view::HandlerResult;
use kivact_core::view::{Handled, Key, KeyEvent, PointerEvent, PointerPhase, WheelEvent};
use machine::Gesture;
use std::rc::Rc;

/// A headless binding-backed zoom and pan builder. Attach [`Self::viewport`]
/// to the caller's node; drawing and content remain the caller's responsibility.
#[derive(Clone, Copy)]
pub struct ZoomPanApi {
    // xtask-allow(state-fields): independent: the observation of the attached viewport part, absent until it attaches
    observed: Signal<Option<Value<PartState>>>,
    transform: Binding<ViewTransform>,
    settings: OptionsCell<ZoomPanOptions>,
    edit: EditSession<ViewTransform>,
    /// The one gesture; Space is held in its idle state, so
    /// `is_gesturing` and `is_space_armed` follow it.
    gesture: MachineHandle<machine::Machine>,
    gesturing: Value<bool>,
    armed: Value<bool>,
}

/// Create shared zoom and pan behavior during setup.
#[track_caller]
pub fn use_zoom_pan(transform: impl kivact_core::state::IntoBinding<ViewTransform>) -> ZoomPanApi {
    let transform = transform.into_binding();
    let settings = OptionsCell::new(ZoomPanOptions::default());
    let edit = EditSession::new(move |initial| transform.set(initial));
    let session = Rc::new(edit);
    let gesture = machine(
        Gesture::Idle {
            space: false,
            touches: Vec::new(),
        },
        move || machine::Machine {
            options: settings.get(),
            transform: transform.get(),
            session: session.clone(),
        },
        move |_, effect| match effect {
            machine::Effect::Begin(session, initial) => session.begin(initial),
            machine::Effect::Write(value) => {
                transform.set(value);
            }
            machine::Effect::Finish(session, commit) => {
                if commit {
                    session.commit();
                } else {
                    session.cancel();
                }
            }
        },
    );
    let gesturing = memo(move || gesture.get().active());
    let armed = memo(move || gesture.get().space());
    ZoomPanApi {
        observed: signal(None),
        transform,
        settings,
        edit,
        gesture,
        gesturing,
        armed,
    }
}

impl kivact_core::view::ViewportTransform for ZoomPanApi {
    fn view_transform(&self) -> Value<ViewTransform> {
        self.transform.into_value()
    }
    fn attach(
        self,
        node: kivact_core::view::Node,
        content: Value<Rect>,
    ) -> kivact_core::view::Node {
        node.part(self.content(content).viewport())
    }
}
impl ZoomPanApi {
    /// Map a view-local point through the current shared pose into content.
    pub fn to_content(self, point: Point) -> Point {
        self.transform.get().to_content(point)
    }
    /// Map a content point through the current shared pose into the view.
    pub fn to_view(self, point: Point) -> Point {
        self.transform.get().to_view(point)
    }

    /// Drawing state of the actual attached input target.
    pub fn viewport_state(self) -> Value<PartState> {
        kivact_core::state::memo(move || {
            self.observed
                .get()
                .map(|state| state.get())
                .unwrap_or_default()
        })
    }

    /// Positive finite scale bounds with `min <= max` (default 0.01 through
    /// 100); anything else is a usage error. Any order. Setup only.
    #[track_caller]
    pub fn scale_range(self, min: impl Into<f64>, max: impl Into<f64>) -> Self {
        let min = min.into();
        let max = max.into();
        let valid = |bound: f64| bound.is_finite() && bound > 0.0;
        if !valid(min) || !valid(max) || min > max {
            raise(
                ErrorKind::Message,
                format!(
                    "ZoomPanApi::scale_range: the bounds must be positive and finite with min <= max, got {min} and {max}"
                ),
            );
        }
        self.settings.update(|settings| {
            settings.min = min;
            settings.max = max;
        });
        self
    }

    /// The content's bounds in content coordinates, which the `0` key fits
    /// into the surface. Any order. Setup only.
    #[track_caller]
    pub fn content(self, bounds: impl IntoValue<Rect>) -> Self {
        let bounds = bounds.into_value();
        self.settings
            .update(|settings| settings.content = Some(bounds));
        self
    }

    /// Report one edit per wheel/key operation and one per complete pan.
    pub fn on_edit<R: HandlerResult>(self, observer: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.edit.on_edit(observer);
        self
    }

    /// Space is held on the surface and no pan or pinch runs, so a primary
    /// press pans (tracked).
    pub fn is_space_armed(self) -> Value<bool> {
        self.armed
    }

    /// A pan or a pinch is active (tracked).
    pub fn is_gesturing(self) -> Value<bool> {
        self.gesturing
    }

    fn zoom(self, current: ViewTransform, anchor: Point, factor: f64) -> ViewTransform {
        let ZoomPanOptions { min, max, .. } = self.settings.get();
        let scale = (valid_scale(current.scale) * factor).clamp(min, max);
        let content = current.to_content(anchor);
        ViewTransform {
            scale,
            offset: Point::new(anchor.x - content.x * scale, anchor.y - content.y * scale),
        }
    }

    /// End the gesture (if any) and its edit: committed, or cancelled with
    /// the starting transform written back. The only place a gesture ends.
    fn stop(self, commit: bool) {
        self.gesture.send(machine::Event::Stop(commit));
    }

    fn viewport_key(self, event: &KeyEvent, rect: Option<Rect>) -> Handled {
        let binding = self.transform;
        let value = binding.into_value();
        let edit = self.edit;
        let gesturing = self.gesturing.get_untracked();
        if event.key() == Key::Space {
            self.gesture.send(machine::Event::Space(true));
            return if gesturing {
                Handled::Yes
            } else {
                Handled::PreventDefault
            };
        }
        if event.key() == Key::Escape && gesturing {
            self.stop(false);
            return Handled::Yes;
        }
        if gesturing && matches!(event.key(), Key::Char('+' | '-' | '0')) {
            return Handled::Yes;
        }
        let current = value.get();
        let Some(rect) = rect else {
            return Handled::No;
        };
        let anchor = Point::new(rect.width / 2.0, rect.height / 2.0);
        let next = match event.key() {
            Key::Char('+') => self.zoom(current, anchor, 1.2),
            Key::Char('-') => self.zoom(current, anchor, 1.0 / 1.2),
            Key::Char('0') => {
                let Some(content) = self.settings.get().content else {
                    return Handled::No;
                };
                let bounds = content.get();
                let fit = ViewTransform::fit(
                    Size::new(bounds.width, bounds.height),
                    Size::new(rect.width, rect.height),
                );
                // The bounds may not start at the origin: its corner
                // goes to the surface's.
                let fit = ViewTransform {
                    scale: fit.scale,
                    offset: Point::new(
                        fit.offset.x - bounds.x * fit.scale,
                        fit.offset.y - bounds.y * fit.scale,
                    ),
                };
                self.zoom(fit, anchor, 1.0)
            }
            _ => return Handled::No,
        };
        edit.once(current, || {
            binding.set(next);
        });
        Handled::Yes
    }

    /// A focusable surface using the existing native capture and focus paths.
    ///
    /// Wheel zooms around the pointer by `exp(-delta.y * 0.001)`; `+` and `-`
    /// zoom around the surface center by 1.2 and its reciprocal. `0` fits the
    /// [`content`](Self::content) bounds. Zoom scales are clamped to
    /// [`Self::scale_range`].
    /// Middle-button drag pans. Plain primary presses are unhandled, leaving
    /// selection and gizmos to the caller; Space plus primary is the exception
    /// and pans while Space is held at the press ([`is_space_armed`](Self::is_space_armed)).
    /// Releasing Space does not end an active pan. Space is cleared on key
    /// release, focus loss and Cancel.
    /// Space presses and releases reach ancestor handlers except during a pan
    /// ([`Handled::PreventDefault`]), but never activate the focused control
    /// or region inside the surface. An application-wide Space action
    /// therefore also runs when Space is pressed to start a
    /// Space-plus-primary pan.
    /// Pan snapshots the binding at press, commits on matching release, and
    /// restores it on Escape, PointerCancel, focus departure, hiding, disabling
    /// or owner disposal. Release applies its final position before Commit.
    /// Wheel and zoom/fit keys are consumed without changing the transform
    /// during an active pan or pinch. Caller handlers run first and can
    /// consume events.
    ///
    /// Two fingers on the surface pinch: the second touch contact down while
    /// another is down on it (and no pan is active) starts a pinch that
    /// zooms by the change of the fingers' distance and pans by their
    /// midpoint, keeping the content under the fingers there. The first
    /// finger alone is unhandled, like a plain primary press. A pinch is one
    /// edit like a pan: either finger lifted commits it, a canceled finger,
    /// Escape, focus departure, hiding, disabling or owner disposal restore
    /// the transform from its start. A third finger is unhandled. Dedicated
    /// touchpad pinch events are not exposed by the public input path;
    /// trackpad wheel events zoom.
    #[track_caller]
    pub fn viewport(self) -> Part<PartState> {
        let binding = self.transform;
        let value = binding.into_value();
        let edit = self.edit;
        let input = part(|input| input);
        let part_state = input.state();
        self.observed.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            let surface = host
                .part(input)
                .node_ref(target)
                .focusable(true)
                .region_events();
            let state = part_state;
            let rect = target.rect();
            let focus = target.focus_within();
            self.gesture.feed(
                memo(move || {
                    let state = state.get();
                    focus.get() && !state.disabled && !state.hidden
                }),
                machine::Event::Available,
            );
            surface
                .on_pointer(move |event: PointerEvent| {
                    if event.phase() == PointerPhase::Cancel {
                        self.gesture.send(machine::Event::Space(false));
                    }
                    let snapshot = machine::Machine {
                        options: self.settings.get(),
                        transform: value.get_untracked(),
                        session: Rc::new(edit),
                    };
                    let handled = snapshot.pointer(&self.gesture.get(), event).1;
                    // The machine input reads the same surface snapshot used to accept input.
                    if state.get_untracked().disabled
                        && matches!(event.phase(), PointerPhase::Down(_))
                    {
                        return Handled::No;
                    }
                    self.gesture.send(machine::Event::Pointer(event));
                    handled
                })
                .on_wheel(move |event: WheelEvent| {
                    if self.gesturing.get_untracked() {
                        return Handled::Yes;
                    }
                    if !finite(event.pos())
                        || !event.delta().y.is_finite()
                        || event.delta().y == 0.0
                        || state.get().disabled
                    {
                        return Handled::No;
                    }
                    let current = value.get();
                    edit.once(current, || {
                        binding.set(self.zoom(
                            current,
                            event.pos(),
                            (-event.delta().y * 0.001).exp(),
                        ));
                    });
                    Handled::Yes
                })
                .on_key(move |event: KeyEvent| self.viewport_key(&event, rect.get()))
                .on_key_up(move |event: KeyEvent| {
                    if event.key() == Key::Space {
                        self.gesture.send(machine::Event::Space(false));
                        if self.gesturing.get_untracked() {
                            Handled::Yes
                        } else {
                            Handled::PreventDefault
                        }
                    } else {
                        Handled::No
                    }
                })
        })
    }
}
