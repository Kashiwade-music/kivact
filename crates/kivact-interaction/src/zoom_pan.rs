//! Role: Shared view transforms and owner-scoped zoom and pan input.
//!
//! [`use_zoom_pan`] edits a caller's [`ViewTransform`] binding and shares
//! one input map: wheel zoom around the pointer, zoom and fit keys, a
//! middle-button or Space-plus-primary pan and a two-finger pinch (the
//! fingers' distance ratio scales, and the content point under their
//! starting midpoint follows the current midpoint). A plain primary press is
//! left to the application, for selection and gizmos. Each wheel notch or
//! key is one edit and each pan or pinch is one edit of an [`EditSession`];
//! the session is used directly rather than through a `DragApi`, since these
//! gestures do not fit one primary-button drag (see `crate::drag`). The
//! gesture is one state ([`ZoomPanApi::is_gesturing`]) ended in one place. Space
//! is tracked by a [`SpacePanApi`](crate::SpacePanApi) ([`ZoomPanApi::is_space_armed`]):
//! while not panning, Space presses and releases update the state and reach
//! ancestors, returning `Handled::PreventDefault` so the focused part is not
//! activated; an application that binds Space to play or stop therefore
//! also runs it when a Space-plus-drag pan starts. Releasing Space alone
//! does not end a pan. Touchpad pinch events are not on the public input
//! path yet.
//!
//! The surface part opts into `Part::region_events`, so the unhandled
//! pointer, key and wheel input of a canvas's regions reaches it; a menu's
//! context trigger (`crate::menu`) uses the same opt-in, and a kit does not
//! deliver these events again itself.

use crate::EditSession;
use crate::space_pan::{SpacePanApi, use_space_pan};
use kivact_core::ErrorKind;
use kivact_core::custom::{Part, PartState, part, raise};
use kivact_core::geometry::{Point, Rect, Size};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Signal, Value, Var, effect, memo, signal, untrack, var};
use kivact_core::view::EditPhase;
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerId, PointerKind, PointerPhase,
    WheelEvent,
};

/// Content-to-view coordinates: `view = content * scale + offset`.
/// Invalid (non-positive or non-finite) scales are treated as 1 by operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub offset: Point,
    pub scale: f64,
}
kivact_core::state::constant_value!(ViewTransform);
impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            offset: Point::new(0.0, 0.0),
            scale: 1.0,
        }
    }
}
impl ViewTransform {
    /// Map a surface-local point into content coordinates.
    pub fn to_content(&self, view: Point) -> Point {
        let scale = valid_scale(self.scale);
        Point::new(
            (view.x - self.offset.x) / scale,
            (view.y - self.offset.y) / scale,
        )
    }
    /// Map a content point into surface-local coordinates.
    pub fn to_view(&self, content: Point) -> Point {
        let scale = valid_scale(self.scale);
        Point::new(
            content.x * scale + self.offset.x,
            content.y * scale + self.offset.y,
        )
    }
    /// Uniformly fit the whole content into the view, centered.
    /// Non-positive or non-finite dimensions return the identity transform.
    pub fn fit(content: Size, view: Size) -> Self {
        if ![content.width, content.height, view.width, view.height]
            .into_iter()
            .all(|v| v.is_finite() && v > 0.0)
        {
            return Self::default();
        }
        let scale = (view.width / content.width).min(view.height / content.height);
        Self {
            scale,
            offset: Point::new(
                (view.width - content.width * scale) / 2.0,
                (view.height - content.height * scale) / 2.0,
            ),
        }
    }
}
fn valid_scale(scale: f64) -> f64 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}
fn finite(point: Point) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

/// What the setters of [`ZoomPanApi`] store; read at the event, so they work in
/// any order.
#[derive(Clone, Copy)]
struct ZoomSettings {
    min: f64,
    max: f64,
    content: Option<Value<Rect>>,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            min: 0.01,
            max: 100.0,
            content: None,
        }
    }
}

/// The one gesture of the surface: nothing, a pan or a pinch.
#[derive(Clone, Copy)]
enum Gesture {
    Idle,
    Pan(Pan),
    Pinch(Pinch),
}

/// A pan: the pointer that runs it and the transform it started from.
#[derive(Clone, Copy)]
struct Pan {
    pointer: PointerId,
    position: Point,
    transform: ViewTransform,
    button: MouseButton,
}

/// A headless binding-backed zoom and pan builder. Attach [`Self::viewport`]
/// to the caller's node; drawing and content remain the caller's responsibility.
#[derive(Clone, Copy)]
pub struct ZoomPanApi {
    observed: Signal<Option<Value<PartState>>>,
    transform: Binding<ViewTransform>,
    settings: Var<ZoomSettings>,
    edit: EditSession<ViewTransform>,
    space: SpacePanApi,
    /// The one gesture; a Signal so that `is_gesturing` and `is_space_armed`
    /// follow it.
    gesture: Signal<Gesture>,
    /// The touch contacts down on the surface, at their last positions.
    touches: Var<Vec<(PointerId, Point)>>,
    gesturing: Value<bool>,
    armed: Value<bool>,
}

/// Create shared zoom and pan behavior during setup.
#[track_caller]
pub fn use_zoom_pan(transform: impl kivact_core::state::IntoBinding<ViewTransform>) -> ZoomPanApi {
    let transform = transform.into_binding();
    let space = use_space_pan();
    let gesture = signal(Gesture::Idle);
    let gesturing = memo(move || !matches!(gesture.with(|gesture| *gesture), Gesture::Idle));
    let armed = memo(move || space.armed().get() && !gesturing.get());
    ZoomPanApi {
        observed: signal(None),
        transform,
        settings: var(ZoomSettings::default()),
        edit: EditSession::new(move |initial| transform.set(initial)),
        space,
        gesture,
        touches: var(Vec::new()),
        gesturing,
        armed,
    }
}

/// One finger of a pinch: where it went down and where it is.
#[derive(Clone, Copy)]
struct Finger {
    pointer: PointerId,
    start: Point,
    now: Point,
}

/// A two-finger pinch: its fingers and the transform at its start.
#[derive(Clone, Copy)]
struct Pinch {
    fingers: [Finger; 2],
    transform: ViewTransform,
}

fn midpoint(a: Point, b: Point) -> Point {
    Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

impl ZoomPanApi {
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

    /// The transform `pinch` makes: the content point under the fingers'
    /// starting midpoint follows their midpoint, scaled by how far apart
    /// they are now against at the start (clamped to the scale range).
    fn pinched(self, pinch: Pinch) -> ViewTransform {
        let ZoomSettings { min, max, .. } = self.settings.get();
        let [a, b] = pinch.fingers;
        let (start, now) = (distance(a.start, b.start), distance(a.now, b.now));
        let factor = if start > 0.0 && now.is_finite() {
            now / start
        } else {
            1.0
        };
        let scale = (valid_scale(pinch.transform.scale) * factor).clamp(min, max);
        let content = pinch.transform.to_content(midpoint(a.start, b.start));
        let anchor = midpoint(a.now, b.now);
        ViewTransform {
            scale,
            offset: Point::new(anchor.x - content.x * scale, anchor.y - content.y * scale),
        }
    }

    fn zoom(self, current: ViewTransform, anchor: Point, factor: f64) -> ViewTransform {
        let ZoomSettings { min, max, .. } = self.settings.get();
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
        if self.gesturing.get_untracked() {
            self.gesture.update(|gesture| *gesture = Gesture::Idle);
        }
        if commit {
            self.edit.commit();
        } else {
            self.edit.cancel();
        }
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
            self.space.watch(target, state);
            effect(move || {
                let state = state.get();
                if !focus.get() || state.disabled || state.hidden {
                    untrack(|| self.stop(false));
                }
            });
            surface
                .on_pointer(move |event: PointerEvent| {
                    if event.kind() == PointerKind::Touch
                        && let Some(handled) = self.touch(event, state)
                    {
                        return handled;
                    }
                    if let PointerPhase::Down(press) = event.phase() {
                        let button = press.button;
                        if self.gesturing.get_untracked()
                            || state.get().disabled
                            || !finite(event.pos())
                            || !(button == MouseButton::Middle
                                || (button == MouseButton::Primary && self.armed.get_untracked()))
                        {
                            return Handled::No;
                        }
                        let transform = value.get();
                        self.gesture.update(|gesture| {
                            *gesture = Gesture::Pan(Pan {
                                pointer: event.id(),
                                position: event.pos(),
                                transform,
                                button,
                            });
                        });
                        edit.begin(transform);
                        return Handled::Yes;
                    }
                    if event.phase() == PointerPhase::Cancel {
                        self.space.disarm();
                    }
                    let Gesture::Pan(active) = self.gesture.get_untracked() else {
                        return Handled::No;
                    };
                    if event.id() != active.pointer {
                        return Handled::No;
                    }
                    match event.phase() {
                        PointerPhase::Cancel => self.stop(false),
                        PointerPhase::Up(press) if press.button == active.button => {
                            if finite(event.pos()) {
                                binding.set(panned(active, event.pos()));
                            }
                            self.stop(true);
                        }
                        PointerPhase::Move if finite(event.pos()) => {
                            binding.set(panned(active, event.pos()));
                        }
                        _ => return Handled::No,
                    }
                    Handled::Yes
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
                .on_key(move |event: KeyEvent| {
                    let gesturing = self.gesturing.get_untracked();
                    if self.space.key(&event, true) {
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
                    let Some(rect) = rect.get() else {
                        return Handled::No;
                    };
                    let anchor = Point::new(rect.width / 2.0, rect.height / 2.0);
                    let next = match event.key() {
                        Key::Char('+') => self.zoom(current, anchor, 1.2),
                        Key::Char('-') => self.zoom(current, anchor, 1.0 / 1.2),
                        Key::Char('0') => {
                            let Some(content) = self.settings.with(|settings| settings.content)
                            else {
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
                })
                .on_key_up(move |event: KeyEvent| {
                    if self.space.key(&event, false) {
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

    /// A touch contact's event on the surface: the pinch it starts, moves or
    /// ends (`Some`), or `None` to go on as any pointer's event (a pan with
    /// Space, else unhandled).
    fn touch(self, event: PointerEvent, state: Value<PartState>) -> Option<Handled> {
        let finite = finite(event.pos());
        let touches = self.touches;
        match event.phase() {
            PointerPhase::Down(_) => {
                if finite {
                    touches.update(|t| t.push((event.id(), event.pos())));
                }
                let others: Vec<(PointerId, Point)> = touches.with(|t| {
                    t.iter()
                        .copied()
                        .filter(|(id, _)| *id != event.id())
                        .collect()
                });
                if !finite
                    || others.len() != 1
                    || self.gesturing.get_untracked()
                    || state.get().disabled
                {
                    return None;
                }
                let (first, at) = others[0];
                let transform = self.transform.into_value().get();
                self.gesture.update(|gesture| {
                    *gesture = Gesture::Pinch(Pinch {
                        fingers: [
                            Finger {
                                pointer: first,
                                start: at,
                                now: at,
                            },
                            Finger {
                                pointer: event.id(),
                                start: event.pos(),
                                now: event.pos(),
                            },
                        ],
                        transform,
                    });
                });
                self.edit.begin(transform);
                Some(Handled::Yes)
            }
            PointerPhase::Move => {
                if finite {
                    touches.update(|t| {
                        if let Some(touch) = t.iter_mut().find(|(id, _)| *id == event.id()) {
                            touch.1 = event.pos();
                        }
                    });
                }
                let Gesture::Pinch(mut active) = self.gesture.get_untracked() else {
                    return None;
                };
                let finger = active
                    .fingers
                    .iter_mut()
                    .find(|f| f.pointer == event.id())?;
                if finite {
                    finger.now = event.pos();
                    self.gesture
                        .update(|gesture| *gesture = Gesture::Pinch(active));
                    self.transform.set(self.pinched(active));
                }
                Some(Handled::Yes)
            }
            PointerPhase::Up(_) | PointerPhase::Cancel => {
                touches.update(|t| t.retain(|(id, _)| *id != event.id()));
                let Gesture::Pinch(mut active) = self.gesture.get_untracked() else {
                    return None;
                };
                let finger = active
                    .fingers
                    .iter_mut()
                    .find(|f| f.pointer == event.id())?;
                if matches!(event.phase(), PointerPhase::Up(_)) {
                    if finite {
                        finger.now = event.pos();
                        self.transform.set(self.pinched(active));
                    }
                    self.stop(true);
                } else {
                    self.stop(false);
                }
                Some(Handled::Yes)
            }
            PointerPhase::Enter | PointerPhase::Leave => None,
        }
    }
}

/// The transform of `pan` with the pointer at `pos`.
fn panned(pan: Pan, pos: Point) -> ViewTransform {
    ViewTransform {
        scale: valid_scale(pan.transform.scale),
        offset: Point::new(
            pan.transform.offset.x + pos.x - pan.position.x,
            pan.transform.offset.y + pos.y - pan.position.y,
        ),
    }
}
