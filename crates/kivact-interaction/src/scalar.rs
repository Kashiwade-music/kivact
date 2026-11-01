//! Role: Binding-backed scalar operations and semantic control parts.
//!
//! A [`Scalar`] has no value of its own: the finite range, the min-anchored
//! step, the key step and assistive technology all write through the
//! caller's binding. A slider attaches [`Scalar::thumb`] and
//! [`Scalar::track`]; a number field attaches [`Scalar::input`] to its text
//! field; a canvas declares the thumb as a region and maps the pointer itself
//! with [`Scalar::scrub`]. The slider's thumb and the spin button share this
//! private implementation, except that the spin button leaves the value text
//! to the native editor. [`RangeScalar`](crate::RangeScalar) builds two ends
//! on the same plumbing. The reading direction a horizontal track mirrors in
//! comes from `kivact::view::ReadingDirection` (`provide_direction`,
//! `current_direction`).
//!
//! A spin button takes Up/Down, PageUp/PageDown (ten steps) and unmodified
//! Home/End; Left/Right and modified Home/End stay the native editor's caret
//! keys. In the engine's native editor, unmodified Home/End skip native
//! editing and go to the key handlers only when the editor, or a
//! non-focusable ancestor, is a `SpinButton` with an explicit SetValue
//! action; a `TextBox` inside a focusable numeric control (Creative's), an
//! editor that only has the `SpinButton` role and an ordinary `TextBox` keep
//! native caret movement. Assistive technology's SetValue on an editor that
//! is itself such a spin button goes to the explicit action handler (the
//! numeric writer) without passing through text input; an ordinary native
//! SetValue is edited by the engine and notified as `TextChanged`, and the
//! core's `run_a11y` hands a host SetValue to the explicit action handler
//! without sending it to text input again. The kits put the part on the real
//! field or on a
//! non-focusable row around it: the standard `number_input` makes the field
//! itself the spin button, and a component that keeps the field a `TextBox`
//! makes the non-focusable row the spin button (with the name and range,
//! without a value text); both focus the field and share these keys.
//!
//! Integer editing projects the binding to `f64` and uses a [`ScalarRange`]
//! with step 1: a finite assistive technology value is rounded to the
//! nearest whole step from the minimum and clamped, a non-finite one is not
//! written, and the field's typed text is parsed as `i32` by the kit. After
//! an external write left the value outside the range, a step first brings
//! the current value inside the range and then moves it.
//!
//! Invariants:
//! - A range change writes the value back only when it falls outside the new
//!   range, through one effect per scalar however many parts are attached.
//! - A key press, an assistive technology action and a track drag are each
//!   one edit of the scalar's [`EditSession`]; Escape, an engine Cancel and
//!   the focus leaving the thumb during a drag write the starting value back.

mod track;

use crate::drag::Drag;
use crate::edit::{EditPhase, EditSession};
use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use kivact_core::diagnostics::ErrorKind;
use kivact_core::draw::Rect;
use kivact_core::primitive::{Part, PartState, part, raise};
use kivact_core::view::{
    A11yAction, A11yEvent, Handled, HandlerResult, IntoValue, Key, KeyEvent, Modifiers, NodeRef,
    Orientation, Role, node_ref,
};
use kivact_core::{Binding, Signal, Value, effect, effect_on, memo, signal, var};
use std::ops::RangeInclusive;
use std::panic::Location;
pub(crate) use track::{BODY, fraction};
use track::{Pointer, Target};

/// A range a [`Scalar`] or [`SplitterApi`](crate::SplitterApi) reads: constant
/// bounds (`0.0..=100.0`), a constant [`ScalarRange`], or a closure, `Signal`
/// or `Value` that yields one and may change with the layout. Constant and
/// closure ranges need setup. The trait is sealed: those are its only forms.
pub trait IntoScalarRange: sealed::SealedRange {
    #[doc(hidden)]
    fn into_range(self) -> Value<ScalarRange>;
}

mod sealed {
    //! Role: Seal of `IntoScalarRange` to the range forms this crate accepts.
    pub trait SealedRange {}
}

impl sealed::SealedRange for ScalarRange {}
impl sealed::SealedRange for RangeInclusive<f64> {}
impl sealed::SealedRange for Value<ScalarRange> {}
impl sealed::SealedRange for Signal<ScalarRange> {}
impl<F: Fn() -> ScalarRange + 'static> sealed::SealedRange for F {}

impl IntoScalarRange for ScalarRange {
    #[track_caller]
    fn into_range(self) -> Value<ScalarRange> {
        self.into_value()
    }
}

/// Continuous bounds from `min..=max`. Bounds that are not finite, or a
/// `min` above `max`, are a usage error located at the caller.
impl IntoScalarRange for RangeInclusive<f64> {
    #[track_caller]
    fn into_range(self) -> Value<ScalarRange> {
        let (min, max) = self.into_inner();
        match ScalarRange::new(min, max) {
            Ok(range) => range.into_value(),
            Err(error) => raise(
                ErrorKind::Message,
                format!("the range {min}..={max} is not valid: {error:?}"),
                Location::caller(),
            ),
        }
    }
}

impl IntoScalarRange for Value<ScalarRange> {
    fn into_range(self) -> Value<ScalarRange> {
        self
    }
}

impl IntoScalarRange for Signal<ScalarRange> {
    fn into_range(self) -> Value<ScalarRange> {
        self.into_value()
    }
}

impl<F: Fn() -> ScalarRange + 'static> IntoScalarRange for F {
    #[track_caller]
    fn into_range(self) -> Value<ScalarRange> {
        memo(self)
    }
}

/// Reactive drawing state of an attached scalar thumb.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScalarState {
    pub input: PartState,
    /// The caller's current value.
    pub value: f64,
    /// Where the value lies in the current range, from 0 at the minimum to 1
    /// at the maximum. A value outside the range is clamped; an empty range or
    /// a non-finite value is 0.
    pub fraction: f64,
}

/// The edit plumbing one [`Scalar`] or both thumbs of a
/// [`RangeScalar`](crate::RangeScalar) share: the session, what it restores,
/// the track's geometry and the one drag of the thumbs and the track.
#[derive(Clone, Copy)]
pub(crate) struct Shared {
    pub(crate) session: EditSession<[f64; 2]>,
    pub(crate) snapshot: Value<[f64; 2]>,
    pub(crate) pointer: Pointer,
    pub(crate) drag: Drag<Option<usize>>,
    /// The caller's disabled value, given to [`Scalar::disabled`].
    pub(crate) disabled: Signal<Option<Value<bool>>>,
    /// How many thumbs have the focus; a drag the focus leaves is canceled.
    focused: Signal<usize>,
}

impl Shared {
    /// Setup only.
    #[track_caller]
    pub(crate) fn new(snapshot: Value<[f64; 2]>, restore: impl Fn([f64; 2]) + 'static) -> Self {
        let session = EditSession::new(restore);
        let pointer = Pointer::new();
        let disabled = signal(None);
        let shared = Shared {
            session,
            snapshot,
            pointer,
            drag: track::drag(session, snapshot, pointer, disabled),
            disabled,
            focused: signal(0),
        };
        effect_on(shared.focused, move |focused| {
            if *focused == 0 {
                shared.drag.cancel();
            }
        });
        shared
    }

    /// Whether the arrows of a thumb mirror: a horizontal track in a
    /// right-to-left subtree.
    fn mirrored(self) -> bool {
        self.pointer
            .geometry
            .get()
            .is_some_and(|geometry| geometry.rtl)
    }

    /// Run one whole edit around `write` (or join the active one).
    pub(crate) fn once(self, write: impl FnOnce() -> bool) -> bool {
        let mut accepted = false;
        self.session
            .once(self.snapshot.get_untracked(), || accepted = write());
        accepted
    }
}

/// The edit plumbing of one thumb.
#[derive(Clone, Copy)]
pub(crate) struct Edit {
    pub(crate) shared: Shared,
    /// Which thumb of the slider this is, from 0.
    pub(crate) index: usize,
    /// The thumb's element, which a press on the track focuses.
    pub(crate) thumb: NodeRef,
    pub(crate) thumb_rect: Value<Option<Rect>>,
    /// The published bounds of a thumb that another thumb limits.
    pub(crate) bounds: Option<(Value<f64>, Value<f64>)>,
}

impl Edit {
    /// Setup only.
    #[track_caller]
    pub(crate) fn new(
        shared: Shared,
        index: usize,
        bounds: Option<(Value<f64>, Value<f64>)>,
    ) -> Self {
        let thumb = node_ref();
        Edit {
            shared,
            index,
            thumb,
            thumb_rect: thumb.rect(),
            bounds,
        }
    }
}

/// Copy handle for scalar operations writing an explicitly supplied binding,
/// the behavior of react-aria's `useSlider` and `useNumberField`.
///
/// There is no hidden value or synchronization effect. Direct writes to the
/// binding remain possible and are not validated by this handle. Route edits
/// through [`set`](Self::set) when finite bounds and quantization must be
/// enforced. The range and keyboard step are separate: continuous dragging can
/// coexist with stepped keyboard editing.
///
/// Attach the [`thumb`](Self::thumb) to the element that takes the focus, and
/// the [`track`](Self::track) to the element the pointer moves along, with the
/// thumb inside it. Without a track the pointer mapping stays the caller's:
/// a canvas declares the thumb as a region with `.region(id, ..)`.
///
/// Every write of a key press, an assistive technology action or a track drag
/// is one edit ([`on_edit`](Self::on_edit)). Escape, an engine Cancel and the
/// focus leaving the thumb during a drag write the value from before the drag
/// back.
#[derive(Clone, Copy)]
pub struct Scalar {
    pub(crate) binding: Binding<f64>,
    pub(crate) range: Value<ScalarRange>,
    pub(crate) key_step: Option<f64>,
    pub(crate) disabled: Option<Value<bool>>,
    pub(crate) edit: Option<Edit>,
}

/// Use this binding as the sole writer of a scalar value.
///
/// `range` bounds and quantizes every write. It may be reactive: pass a
/// constant [`ScalarRange`], or a closure, a `Signal` or a `Value` of one
/// ([`IntoScalarRange`]). The published bounds and Home/End follow the current
/// range; when the range changes and the value is now outside it, the scalar
/// writes the value back inside through the binding, once however many parts
/// are attached. Creation does not write state; a range given as a closure
/// becomes a memo. Setup only.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{control, prelude, style};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{EditPhase, ScalarRange, use_scalar};
/// use kivact::prelude::*;
/// use kivact::style::Position;
/// let volume = signal(0.5);
/// let range = ScalarRange::new(0.0, 1.0).unwrap();
/// let slider = use_scalar(volume, range)
///     .key_step(0.05)
///     .unwrap()
///     .on_edit(|phase: EditPhase| println!("{phase:?}"));
/// let thumb = slider.thumb();
/// let state = thumb.state();
/// let view = node()
///     .part(slider.track(Orientation::Horizontal))
///     .style(style().size(160, 24))
///     .child(node().part(thumb).name("Volume").style(move || {
///         let left = 144.0 * state.get().fraction as f32;
///         style().size(16, 16).position(Position::Absolute).left(left).top(4)
///     }));
/// ```
#[track_caller]
pub fn use_scalar(value: impl Into<Binding<f64>>, range: impl IntoScalarRange) -> Scalar {
    let binding = value.into();
    let snapshot = memo(move || [binding.get(), 0.0]);
    let shared = Shared::new(snapshot, move |[start, _]| binding.set(start));
    let scalar = Scalar {
        binding,
        range: range.into_range(),
        key_step: None,
        disabled: None,
        edit: Some(Edit::new(shared, 0, None)),
    };
    follow_range(binding, scalar.range);
    shared.pointer.thumbs.set(vec![scalar.target()]);
    scalar
}

/// What one key press does to the value.
#[derive(Clone, Copy)]
pub(crate) enum KeyOp {
    Steps(f64),
    To(f64),
}

impl Scalar {
    /// Override the positive keyboard and accessibility increment without
    /// changing quantization. The default is the range's step, or 1 when the
    /// range is continuous.
    pub fn key_step(mut self, step: f64) -> Result<Self, ScalarRangeError> {
        if !step.is_finite() || step <= 0.0 {
            return Err(ScalarRangeError::InvalidStep);
        }
        self.key_step = Some(step);
        Ok(self)
    }

    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the thumb disabled and makes it unfocusable; the track
    /// ignores the pointer, and a drag in progress ends where it is. Setup only.
    #[track_caller]
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        let disabled = disabled.into_value();
        self.disabled = Some(disabled);
        if let Some(edit) = self.edit {
            edit.shared
                .disabled
                .update(|current| *current = Some(disabled));
        }
        self
    }

    /// Report the start and end of each edit as an [`EditPhase`], so the app
    /// can make one undo step of it.
    ///
    /// A drag on the [`track`](Self::track) is one edit: `Begin` when the
    /// pointer is pressed, `Commit` when it is released. Escape during the
    /// drag, the engine canceling the pointer, or the focus leaving the thumb
    /// writes the value from before the press back and reports `Cancel`.
    /// Each key press and each assistive technology action is its own edit:
    /// `Begin`, the write, `Commit`. Disabling the scalar during a drag ends
    /// the drag where it is, which commits it; disposing the owner during a
    /// drag cancels it. [`EditPhase`] has the rules every component follows.
    /// A later call replaces the earlier one. Setup only.
    pub fn on_edit<R: HandlerResult>(self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        if let Some(edit) = self.edit {
            edit.shared.session.on_edit(f);
        }
        self
    }

    /// Make each drag of `drag` one edit of this scalar, for a component
    /// that maps the pointer to the value itself (a relative scrub, a dial,
    /// a thumb drawn as a canvas region) instead of the [`track`](Self::track).
    ///
    /// The drag is the same edit as the keys and the track: it begins at the
    /// press, before `drag`'s [`on_start`](Drag::on_start), and commits after
    /// the release. Escape on a source of `drag`, an engine Cancel and
    /// [`Drag::cancel`] write the value from before the press back and report
    /// `Cancel`, as [`Drag::edit`] does. Use this instead of
    /// [`Drag::edit`]; configure it before attaching the drag's sources.
    /// Setup only.
    pub fn scrub<T: Clone + 'static>(self, drag: Drag<T>) -> Drag<T> {
        let edit = self
            .edit
            .expect("invariant: use_scalar gives every Scalar an edit");
        let snapshot = edit.shared.snapshot;
        drag.edit(edit.shared.session, move || snapshot.get_untracked())
    }

    /// The caller's value, including any direct external binding writes.
    pub fn value(self) -> Value<f64> {
        self.binding.into_value()
    }

    /// The bounds and optional quantization, following a reactive range.
    pub fn range(self) -> Value<ScalarRange> {
        self.range
    }

    /// Normalize by the current range and invoke the binding writer.
    ///
    /// `true` means normalization succeeded and the writer was called; a
    /// projected binding may reject the edit, so this does not guarantee that
    /// the binding adopted it. `false` rejects a non-finite input without
    /// writing. A direct call is not an edit and reports no [`EditPhase`].
    #[track_caller]
    pub fn set(self, value: f64) -> bool {
        let Some(value) = self.range.get_untracked().normalize(value) else {
            return false;
        };
        self.binding.set(value);
        true
    }

    /// Change the current normalized value by keyboard step units.
    #[track_caller]
    pub fn step_by(self, steps: f64) -> bool {
        if !steps.is_finite() {
            return false;
        }
        let range = self.range.get_untracked();
        let current = self.value().get_untracked();
        let amount = steps * self.current_key_step(range);
        let next = if amount.is_finite() {
            range.offset(current, amount)
        } else {
            range.normalize(current).and_then(|_| {
                range.normalize(if steps > 0.0 {
                    range.max()
                } else {
                    range.min()
                })
            })
        };
        let Some(next) = next else { return false };
        self.binding.set(next);
        true
    }

    fn current_key_step(self, range: ScalarRange) -> f64 {
        self.key_step.or(range.step()).unwrap_or(1.0)
    }

    /// Directional increments, ten steps for PageUp/PageDown, and Home/End;
    /// `mirror` swaps ArrowLeft and ArrowRight.
    pub(crate) fn key_op(self, key: Key, mirror: bool) -> Option<KeyOp> {
        let range = self.range.get();
        Some(match key {
            Key::ArrowRight if mirror => KeyOp::Steps(-1.0),
            Key::ArrowLeft if mirror => KeyOp::Steps(1.0),
            Key::ArrowRight | Key::ArrowUp => KeyOp::Steps(1.0),
            Key::ArrowLeft | Key::ArrowDown => KeyOp::Steps(-1.0),
            Key::PageUp => KeyOp::Steps(10.0),
            Key::PageDown => KeyOp::Steps(-10.0),
            Key::Home => KeyOp::To(range.min()),
            Key::End => KeyOp::To(range.max()),
            _ => return None,
        })
    }

    /// This scalar as a thumb the pointer moves.
    pub(crate) fn target(self) -> Target {
        let edit = self
            .edit
            .expect("invariant: a pointer target carries an edit");
        Target {
            binding: self.binding,
            range: self.range,
            thumb: edit.thumb,
            rect: edit.thumb_rect,
        }
    }

    #[track_caller]
    pub(crate) fn apply(self, op: KeyOp) -> bool {
        match op {
            KeyOp::Steps(steps) => self.step_by(steps),
            KeyOp::To(value) => self.set(value),
        }
    }

    /// Run `write` as one edit ([`on_edit`](Self::on_edit)): `Begin`, the
    /// write, `Commit`, as a key press does. Inside an edit already in
    /// progress (a drag) the write joins it and nothing more is reported.
    /// Returns what `write` returns, so a refused write such as a
    /// non-finite [`set`](Self::set) reads `false`; the edit is reported
    /// either way. For the writes of a component's own keys or wheel that the
    /// thumb does not handle. Call it from a handler.
    pub fn once(self, write: impl FnOnce() -> bool) -> bool {
        match self.edit {
            Some(edit) => edit.shared.once(write),
            None => write(),
        }
    }

    /// Increment, Decrement and SetValue through the same writer. Returns
    /// whether the writer was offered a value; an invalid or missing value is not.
    #[track_caller]
    pub(crate) fn a11y(self, event: &A11yEvent) -> bool {
        match event.action {
            A11yAction::Increment => self.step_by(1.0),
            A11yAction::Decrement => self.step_by(-1.0),
            A11yAction::SetValue => event
                .value
                .as_deref()
                .and_then(|v| v.parse::<f64>().ok())
                .is_some_and(|v| self.set(v)),
            _ => false,
        }
    }

    /// Create the semantic and input attachment of a slider's thumb. Read its
    /// state before moving it.
    ///
    /// The part sets `Role::Slider`, focusability, the value text, the value
    /// range and current value, the arrow, PageUp/PageDown and Home/End keys,
    /// and the Increment, Decrement and SetValue actions. On a horizontal
    /// [`track`](Self::track) in a right-to-left subtree
    /// ([`current_direction`](kivact_core::view::current_direction)) ArrowLeft increases and ArrowRight decreases.
    /// Escape cancels a track drag. The name, drawing and geometry stay the
    /// caller's. Caller key handlers run first and can stop the default keys;
    /// semantic modifiers written after the attachment override the part's
    /// (for example a formatted `value`). Setup only.
    #[track_caller]
    pub fn thumb(self) -> Part<ScalarState> {
        self.part(Role::Slider)
    }

    /// Create a spin button with the same value and accessibility operations
    /// as [`thumb`](Self::thumb), without replacing native value text, for a
    /// number field's text input.
    /// Up/Down change one step, PageUp/PageDown
    /// ten, and unmodified Home/End select the bounds. Left/Right remain
    /// available for native caret movement. Setup only.
    #[track_caller]
    pub fn input(self) -> Part<ScalarState> {
        self.part(Role::SpinButton)
    }

    /// Create the pointer attachment of the slider's track: a primary press
    /// on it moves the value to the pointer, focuses the thumb, and dragging
    /// follows the pointer until the release.
    ///
    /// `axis` is the direction the value grows in: `Orientation::Horizontal` from
    /// the left edge (from the right edge in a right-to-left subtree,
    /// [`current_direction`](kivact_core::view::current_direction)), `Orientation::Vertical` from the bottom edge.
    /// `Orientation::Unspecified` is treated like `Orientation::Horizontal`. The value spans the
    /// track's length minus the thumb's, so the thumb's center reaches the
    /// value: draw the thumb at `fraction * (track - thumb)` inside the track.
    /// A press on the thumb keeps the distance from the pointer to the
    /// thumb's center, so grabbing it does not move the value. The drag is
    /// one edit ([`on_edit`](Self::on_edit)). Setup only.
    #[track_caller]
    pub fn track(self, axis: Orientation) -> Part<PartState> {
        let edit = self
            .edit
            .expect("invariant: use_scalar gives every Scalar an edit");
        track::track(edit.shared.pointer, edit.shared.drag, edit.thumb, axis)
    }

    #[track_caller]
    fn part(self, role: Role) -> Part<ScalarState> {
        let (value, range) = (self.value(), self.range);
        let mut control = part(move |input| {
            let value = value.get();
            ScalarState {
                input,
                value,
                fraction: fraction(value, range.get()),
            }
        })
        .role(role)
        .focusable(true)
        .value_now(value)
        .on_key(move |event: KeyEvent| {
            if event.key == Key::Escape
                && let Some(edit) = self.edit
                && edit.shared.drag.cancel().is_some()
            {
                return Handled::Yes;
            }
            if role == Role::SpinButton
                && (matches!(event.key, Key::ArrowLeft | Key::ArrowRight)
                    || (matches!(event.key, Key::Home | Key::End)
                        && event.modifiers != Modifiers::default()))
            {
                return Handled::No;
            }
            let mirror =
                role == Role::Slider && self.edit.is_some_and(|edit| edit.shared.mirrored());
            match self.key_op(event.key, mirror) {
                Some(op) if self.once(|| self.apply(op)) => Handled::Yes,
                _ => Handled::No,
            }
        });
        control = match self.edit.and_then(|edit| edit.bounds) {
            Some((min, max)) => control.value_min(min).value_max(max),
            None => control
                .value_min(move || range.get().min())
                .value_max(move || range.get().max()),
        };
        if role == Role::Slider {
            control = control.value_text(move || value.get().to_string());
        }
        for action in [
            A11yAction::Increment,
            A11yAction::Decrement,
            A11yAction::SetValue,
        ] {
            control = control.on_a11y_event(action, move |event: A11yEvent| {
                // Only an action that will write is an edit: a value that is
                // not a number reports nothing.
                let writes = event.action != A11yAction::SetValue
                    || event
                        .value
                        .as_deref()
                        .and_then(|v| v.parse::<f64>().ok())
                        .is_some_and(f64::is_finite);
                if writes {
                    self.once(|| self.a11y(&event));
                }
            });
        }
        if let Some(disabled) = self.disabled {
            control = control.disabled(disabled);
        }
        if let Some(edit) = self.edit {
            watch_focus(edit.shared, control.state());
            control = control.reference(edit.thumb);
            if role == Role::Slider {
                control = control.with(edit.shared.drag.source(Some(edit.index)));
            }
        }
        control
    }
}

/// Count `state`'s focus in the thumbs `shared` watches. Setup only.
#[track_caller]
fn watch_focus(shared: Shared, state: Value<ScalarState>) {
    let was = var(false);
    effect(move || {
        let now = state.get().input.focused;
        if now != was.get() {
            was.set(now);
            shared.focused.update(|count| {
                *count = if now {
                    *count + 1
                } else {
                    count.saturating_sub(1)
                }
            });
        }
    });
}

/// The value a changed range moves `now` to: `None` while it is non-finite or
/// still inside the range.
pub(crate) fn pulled_inside(now: f64, range: ScalarRange) -> Option<f64> {
    if now.is_finite() && (now < range.min() || now > range.max()) {
        range.normalize(now)
    } else {
        None
    }
}

/// Write a value that a changed range left outside back inside it. A range
/// change that keeps the value inside writes nothing. One effect per scalar,
/// made where the scalar is. Setup only.
#[track_caller]
pub(crate) fn follow_range(binding: Binding<f64>, range: Value<ScalarRange>) {
    effect_on(range, move |range| {
        if let Some(inside) = pulled_inside(binding.get_untracked(), *range) {
            binding.set(inside);
        }
    });
}

kivact_core::view::constant_value!(ScalarRange);
