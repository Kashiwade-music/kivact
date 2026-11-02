//! Role: Shared view transforms and owner-scoped zoom and pan input.
//!
//! [`use_zoom_pan`] edits a caller's [`ViewTransform`] binding and shares
//! one input map: wheel zoom around the pointer, zoom and fit keys, a
//! middle-button or Space-plus-primary pan and a two-finger pinch (the
//! fingers' distance ratio scales, and the content point under their
//! starting midpoint follows the current midpoint). A plain primary press is
//! left to the application, for selection and gizmos. Each wheel notch or
//! key is one edit and each pan or pinch is one edit of an [`EditSession`];
//! the session is used directly rather than through a `Drag`, since these
//! gestures do not fit one primary-button drag (see `crate::drag`). While
//! not panning, Space presses and releases update the state and reach
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
use crate::edit::{EditPhase, EditSession};
use kivact_core::draw::{Point, Size};
use kivact_core::primitive::{Part, PartState, part};
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    Handled, IntoValue, Key, KeyEvent, MouseButton, PointerEvent, PointerId, PointerKind,
    PointerPhase, WheelEvent,
};
use kivact_core::{Binding, Value, Var, effect, var};

/// Content-to-view coordinates: `view = content * scale + offset`.
/// Invalid (non-positive or non-finite) scales are treated as 1 by operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub offset: Point,
    pub scale: f64,
}
kivact_core::view::constant_value!(ViewTransform);
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
            ((f64::from(view.x) - f64::from(self.offset.x)) / scale) as f32,
            ((f64::from(view.y) - f64::from(self.offset.y)) / scale) as f32,
        )
    }
    /// Map a content point into surface-local coordinates.
    pub fn to_view(&self, content: Point) -> Point {
        let scale = valid_scale(self.scale);
        Point::new(
            (f64::from(content.x) * scale + f64::from(self.offset.x)) as f32,
            (f64::from(content.y) * scale + f64::from(self.offset.y)) as f32,
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
        let scale = (f64::from(view.width) / f64::from(content.width))
            .min(f64::from(view.height) / f64::from(content.height));
        Self {
            scale,
            offset: Point::new(
                ((f64::from(view.width) - f64::from(content.width) * scale) / 2.0) as f32,
                ((f64::from(view.height) - f64::from(content.height) * scale) / 2.0) as f32,
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

/// A headless binding-backed zoom and pan builder. Attach [`Self::viewport`]
/// to the caller's node; drawing and content remain the caller's responsibility.
#[derive(Clone, Copy)]
pub struct ZoomPan {
    transform: Binding<ViewTransform>,
    min: f64,
    max: f64,
    content: Option<Value<Size>>,
    edit: EditSession<ViewTransform>,
}
/// Create shared zoom and pan behavior during setup.
#[track_caller]
pub fn use_zoom_pan(transform: impl Into<Binding<ViewTransform>>) -> ZoomPan {
    let transform = transform.into();
    ZoomPan {
        transform,
        min: 0.01,
        max: 100.0,
        content: None,
        edit: EditSession::new(move |initial| transform.set(initial)),
    }
}
#[derive(Clone, Copy)]
struct Drag {
    pointer: PointerId,
    position: Point,
    transform: ViewTransform,
    button: MouseButton,
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
    f64::from(a.x - b.x).hypot(f64::from(a.y - b.y))
}
impl ZoomPan {
    /// Positive finite scale bounds (default 0.01 through 100).
    /// Invalid bounds use their defaults; reversed bounds are swapped.
    pub fn scale_range(mut self, min: f64, max: f64) -> Self {
        let min = if min.is_finite() && min > 0.0 {
            min
        } else {
            0.01
        };
        let max = if max.is_finite() && max > 0.0 {
            max
        } else {
            100.0
        };
        self.min = min.min(max);
        self.max = min.max(max);
        self
    }
    /// Content dimensions used by the `0` key to fit the surface.
    pub fn content_size(mut self, size: impl IntoValue<Size>) -> Self {
        self.content = Some(size.into_value());
        self
    }
    /// Report one edit per wheel/key operation and one per complete pan.
    pub fn on_edit<R: HandlerResult>(self, observer: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.edit.on_edit(observer);
        self
    }
    /// The transform `pinch` makes: the content point under the fingers'
    /// starting midpoint follows their midpoint, scaled by how far apart
    /// they are now against at the start (clamped to the scale range).
    fn pinched(self, pinch: Pinch) -> ViewTransform {
        let [a, b] = pinch.fingers;
        let (start, now) = (distance(a.start, b.start), distance(a.now, b.now));
        let factor = if start > 0.0 && now.is_finite() {
            now / start
        } else {
            1.0
        };
        let scale = (valid_scale(pinch.transform.scale) * factor).clamp(self.min, self.max);
        let content = pinch.transform.to_content(midpoint(a.start, b.start));
        let anchor = midpoint(a.now, b.now);
        ViewTransform {
            scale,
            offset: Point::new(
                (f64::from(anchor.x) - f64::from(content.x) * scale) as f32,
                (f64::from(anchor.y) - f64::from(content.y) * scale) as f32,
            ),
        }
    }

    fn zoom(self, current: ViewTransform, anchor: Point, factor: f64) -> ViewTransform {
        let scale = (valid_scale(current.scale) * factor).clamp(self.min, self.max);
        let content = current.to_content(anchor);
        ViewTransform {
            scale,
            offset: Point::new(
                (f64::from(anchor.x) - f64::from(content.x) * scale) as f32,
                (f64::from(anchor.y) - f64::from(content.y) * scale) as f32,
            ),
        }
    }
    /// A focusable surface using the existing native capture and focus paths.
    ///
    /// Wheel zooms around the pointer by `exp(-delta.y * 0.001)`; `+` and `-`
    /// zoom around the surface center by 1.2 and its reciprocal. `0` fits the
    /// configured content. Zoom scales are clamped to [`Self::scale_range`].
    /// Middle-button drag pans. Plain primary presses are unhandled, leaving
    /// selection and gizmos to the caller; Space plus primary is the exception
    /// and pans while Space is held at the press. Releasing Space does not end
    /// an active pan. Space is cleared on key release, focus loss and Cancel.
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
        let drag = var(None::<Drag>);
        // The touch contacts down on the surface, at their last positions.
        let touches = var(Vec::<(PointerId, Point)>::new());
        let pinch = var(None::<Pinch>);
        let gesture = move || drag.with(Option::is_some) || pinch.with(Option::is_some);
        let space = var(false);
        let surface = part(|input| input).focusable(true).region_events();
        let state = surface.state();
        let rect = surface.node_ref().rect();
        let focus = surface.node_ref().focus_within();
        effect(move || {
            if !focus.get() {
                drag.set(None);
                pinch.set(None);
                space.set(false);
                edit.cancel();
            }
        });
        effect(move || {
            let state = state.get();
            if state.disabled || state.hidden {
                drag.set(None);
                pinch.set(None);
                space.set(false);
                edit.cancel();
            }
        });
        surface
            .on_pointer(move |event: PointerEvent| {
                if event.kind == PointerKind::Touch
                    && let Some(handled) = self.touch(event, touches, pinch, drag, edit, state)
                {
                    return handled;
                }
                if let PointerPhase::Down(press) = event.phase {
                    let button = press.button;
                    if gesture()
                        || state.get().disabled
                        || !finite(event.pos)
                        || !(button == MouseButton::Middle
                            || (button == MouseButton::Primary && space.get()))
                    {
                        return Handled::No;
                    }
                    let transform = value.get();
                    drag.set(Some(Drag {
                        pointer: event.id,
                        position: event.pos,
                        transform,
                        button,
                    }));
                    edit.begin(transform);
                    return Handled::Yes;
                }
                if event.phase == PointerPhase::Cancel {
                    space.set(false);
                }
                let Some(active) = drag.get() else {
                    return Handled::No;
                };
                if event.id != active.pointer {
                    return Handled::No;
                }
                match event.phase {
                    PointerPhase::Cancel => {
                        drag.set(None);
                        space.set(false);
                        edit.cancel();
                    }
                    PointerPhase::Up(press) if press.button == active.button => {
                        if finite(event.pos) {
                            binding.set(ViewTransform {
                                scale: valid_scale(active.transform.scale),
                                offset: Point::new(
                                    active.transform.offset.x + event.pos.x - active.position.x,
                                    active.transform.offset.y + event.pos.y - active.position.y,
                                ),
                            });
                        }
                        drag.set(None);
                        edit.commit();
                    }
                    PointerPhase::Move if finite(event.pos) => {
                        binding.set(ViewTransform {
                            scale: valid_scale(active.transform.scale),
                            offset: Point::new(
                                active.transform.offset.x + event.pos.x - active.position.x,
                                active.transform.offset.y + event.pos.y - active.position.y,
                            ),
                        });
                    }
                    _ => return Handled::No,
                }
                Handled::Yes
            })
            .on_wheel(move |event: WheelEvent| {
                if gesture() {
                    return Handled::Yes;
                }
                if !finite(event.pos)
                    || !event.delta.y.is_finite()
                    || event.delta.y == 0.0
                    || state.get().disabled
                {
                    return Handled::No;
                }
                let current = value.get();
                edit.once(current, || {
                    binding.set(self.zoom(
                        current,
                        event.pos,
                        (-f64::from(event.delta.y) * 0.001).exp(),
                    ));
                });
                Handled::Yes
            })
            .on_key(move |event: KeyEvent| {
                if event.key == Key::Space {
                    space.set(true);
                    return if gesture() {
                        Handled::Yes
                    } else {
                        Handled::PreventDefault
                    };
                }
                if event.key == Key::Escape && edit.is_active() {
                    drag.set(None);
                    pinch.set(None);
                    space.set(false);
                    edit.cancel();
                    return Handled::Yes;
                }
                if gesture() && matches!(event.key, Key::Char('+' | '-' | '0')) {
                    return Handled::Yes;
                }
                let current = value.get();
                let Some(rect) = rect.get() else {
                    return Handled::No;
                };
                let anchor = Point::new(rect.width / 2.0, rect.height / 2.0);
                let next = match event.key {
                    Key::Char('+') => self.zoom(current, anchor, 1.2),
                    Key::Char('-') => self.zoom(current, anchor, 1.0 / 1.2),
                    Key::Char('0') => {
                        let Some(content) = self.content else {
                            return Handled::No;
                        };
                        let fit =
                            ViewTransform::fit(content.get(), Size::new(rect.width, rect.height));
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
                if event.key == Key::Space {
                    space.set(false);
                    if gesture() {
                        Handled::Yes
                    } else {
                        Handled::PreventDefault
                    }
                } else {
                    Handled::No
                }
            })
    }

    /// A touch contact's event on the surface: the pinch it starts, moves or
    /// ends (`Some`), or `None` to go on as any pointer's event (a pan with
    /// Space, else unhandled).
    fn touch(
        self,
        event: PointerEvent,
        touches: Var<Vec<(PointerId, Point)>>,
        pinch: Var<Option<Pinch>>,
        drag: Var<Option<Drag>>,
        edit: EditSession<ViewTransform>,
        state: Value<PartState>,
    ) -> Option<Handled> {
        let finite = finite(event.pos);
        match event.phase {
            PointerPhase::Down(_) => {
                if finite {
                    touches.update(|t| t.push((event.id, event.pos)));
                }
                let others: Vec<(PointerId, Point)> = touches.with(|t| {
                    t.iter()
                        .copied()
                        .filter(|(id, _)| *id != event.id)
                        .collect()
                });
                if !finite
                    || others.len() != 1
                    || drag.with(Option::is_some)
                    || pinch.with(Option::is_some)
                    || state.get().disabled
                {
                    return None;
                }
                let (first, at) = others[0];
                let transform = self.transform.into_value().get();
                pinch.set(Some(Pinch {
                    fingers: [
                        Finger {
                            pointer: first,
                            start: at,
                            now: at,
                        },
                        Finger {
                            pointer: event.id,
                            start: event.pos,
                            now: event.pos,
                        },
                    ],
                    transform,
                }));
                edit.begin(transform);
                Some(Handled::Yes)
            }
            PointerPhase::Move => {
                if finite {
                    touches.update(|t| {
                        if let Some(touch) = t.iter_mut().find(|(id, _)| *id == event.id) {
                            touch.1 = event.pos;
                        }
                    });
                }
                let mut active = pinch.get()?;
                let finger = active.fingers.iter_mut().find(|f| f.pointer == event.id)?;
                if finite {
                    finger.now = event.pos;
                    pinch.set(Some(active));
                    self.transform.set(self.pinched(active));
                }
                Some(Handled::Yes)
            }
            PointerPhase::Up(_) | PointerPhase::Cancel => {
                touches.update(|t| t.retain(|(id, _)| *id != event.id));
                let mut active = pinch.get()?;
                let finger = active.fingers.iter_mut().find(|f| f.pointer == event.id)?;
                pinch.set(None);
                if matches!(event.phase, PointerPhase::Up(_)) {
                    if finite {
                        finger.now = event.pos;
                        self.transform.set(self.pinched(active));
                    }
                    edit.commit();
                } else {
                    edit.cancel();
                }
                Some(Handled::Yes)
            }
            PointerPhase::Enter | PointerPhase::Leave => None,
        }
    }
}
