//! Role: The `Scalar` handle: binding writes, key steps, assistive actions and its parts.

use super::edit_state::{Edit, Shared, register_focus};
use super::ops::ScalarOps;
use super::range::{IntoScalarRange, follow_range};
use super::track::{Grip, Target, fraction};
use crate::axis::Axis;
use crate::drag::Drag;
use crate::edit::EditPhase;
use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use kivact_core::primitive::{Part, PartState, part};
use kivact_core::view::{
    A11yAction, A11yEvent, A11yRequest, Handled, HandlerResult, IntoValue, Key, KeyEvent,
    Modifiers, Orientation, Role,
};
use kivact_core::{Binding, Value, memo};

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
    pub(crate) ops: ScalarOps,
    pub(crate) edit: Edit,
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
///         let left = 144.0 * state.get().fraction;
///         style().size(16, 16).position(Position::Absolute).left(left).top(4)
///     }));
/// ```
#[track_caller]
pub fn use_scalar(value: impl Into<Binding<f64>>, range: impl IntoScalarRange) -> Scalar {
    let binding = value.into();
    let snapshot = memo(move || [binding.get(), 0.0]);
    let shared = Shared::new(snapshot, move |[start, _]| binding.set(start));
    let scalar = Scalar {
        ops: ScalarOps {
            binding,
            range: range.into_range(),
        },
        edit: Edit::new(shared, 0, None),
    };
    follow_range(binding, scalar.ops.range);
    shared.pointer.thumbs.set(vec![scalar.target()]);
    scalar
}

impl Scalar {
    /// Override the positive keyboard and accessibility increment without
    /// changing quantization. The default is the range's step, or 1 when the
    /// range is continuous.
    pub fn key_step(self, step: f64) -> Result<Self, ScalarRangeError> {
        if !step.is_finite() || step <= 0.0 {
            return Err(ScalarRangeError::InvalidStep);
        }
        self.edit.shared.key_step.set(Some(step));
        Ok(self)
    }

    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the thumb disabled and makes it unfocusable; the track
    /// ignores the pointer, and a drag in progress ends where it is. Setup only.
    #[track_caller]
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        let disabled = disabled.into_value();
        self.edit
            .shared
            .disabled
            .update(|current| *current = Some(disabled));
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
        self.edit.shared.session.on_edit(f);
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
    /// [`Drag::edit`].
    /// Setup only.
    pub fn scrub<T: Clone + 'static>(self, drag: Drag<T>) -> Drag<T> {
        let edit = self.edit;
        let snapshot = edit.shared.snapshot;
        drag.edit(edit.shared.session, move || snapshot.get_untracked())
    }

    /// The caller's value, including any direct external binding writes.
    pub fn value(self) -> Value<f64> {
        self.ops.binding.into_value()
    }

    /// The bounds and optional quantization, following a reactive range.
    pub fn range(self) -> Value<ScalarRange> {
        self.ops.range
    }

    /// Normalize by the current range and invoke the binding writer.
    ///
    /// `true` means normalization succeeded and the writer was called; a
    /// projected binding may reject the edit, so this does not guarantee that
    /// the binding adopted it. `false` rejects a non-finite input without
    /// writing. A direct call is not an edit and reports no [`EditPhase`].
    #[track_caller]
    pub fn set(self, value: f64) -> bool {
        self.ops.set(value)
    }

    /// Change the current normalized value by keyboard step units.
    #[track_caller]
    pub fn step_by(self, steps: f64) -> bool {
        self.ops.step_by(steps, self.current_key_step())
    }

    /// The keyboard increment: the one given to [`key_step`](Self::key_step),
    /// else the range's step, else 1.
    fn current_key_step(self) -> f64 {
        self.edit
            .shared
            .key_step
            .get()
            .or(self.ops.range.get_untracked().step())
            .unwrap_or(1.0)
    }

    /// This scalar as a thumb the pointer moves.
    pub(crate) fn target(self) -> Target {
        Target {
            binding: self.ops.binding,
            range: self.ops.range,
            thumb: self.edit.thumb,
            rect: self.edit.thumb_rect,
            bounds: self.edit.bounds,
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
        self.edit.shared.once(write)
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

    /// Opt into direct manipulation and coast for touch pointers.
    pub fn touch_gesture(self, config: super::TouchSliderGesture) -> Self {
        self.edit.shared.enable_touch(config);
        self
    }

    /// Whether this thumb is held by a touch contact, when opted in.
    pub fn touch_lifted(self) -> Option<Value<bool>> {
        self.edit
            .shared
            .touch
            .get()
            .map(|touch| touch.lifted(self.edit.index))
    }

    /// Create the pointer attachment of the slider's track: a primary press
    /// on it moves the value to the pointer, focuses the thumb, and dragging
    /// follows the pointer until the release.
    ///
    /// `axis` is the direction the value grows in: `Orientation::Horizontal` from
    /// the left edge (from the right edge in a right-to-left subtree,
    /// [`current_direction`](kivact_core::view::current_direction)), `Orientation::Vertical` from the bottom edge.
    /// `Orientation::Unspecified` is a usage error. The value spans the
    /// track's length minus the thumb's, so the thumb's center reaches the
    /// value: draw the thumb at `fraction * (track - thumb)` inside the track.
    /// A press on the thumb keeps the distance from the pointer to the
    /// thumb's center, so grabbing it does not move the value. The drag is
    /// one edit ([`on_edit`](Self::on_edit)). Setup only.
    #[track_caller]
    pub fn track(self, axis: Orientation) -> Part<PartState> {
        let axis = Axis::of(axis, "Scalar::track");
        let edit = self.edit;
        let shared = edit.shared;
        let surface = part(|input| input).focus_on_press(edit.thumb);
        let surface = shared.touch_source(surface, Grip::Track);
        shared.pointer.attach_track(surface.node_ref().rect(), axis);
        surface.with(shared.drag.source(Grip::Track))
    }

    #[track_caller]
    fn part(self, role: Role) -> Part<ScalarState> {
        let (value, range) = (self.value(), self.ops.range);
        let shared = self.edit.shared;
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
            if event.key() == Key::Escape
                && (shared.finish_touch(true) || shared.drag.cancel().is_some())
            {
                return Handled::Yes;
            }
            if role == Role::SpinButton
                && (matches!(event.key(), Key::ArrowLeft | Key::ArrowRight)
                    || (matches!(event.key(), Key::Home | Key::End)
                        && event.modifiers() != Modifiers::default()))
            {
                return Handled::No;
            }
            let mirror = role == Role::Slider && shared.mirrored();
            match self.ops.key_op(event.key(), mirror) {
                Some(op) if self.once(|| self.ops.apply(op, self.current_key_step())) => {
                    Handled::Yes
                }
                _ => Handled::No,
            }
        });
        control = match self.edit.bounds {
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
                let writes = match &event.request {
                    A11yRequest::SetValue(v) => v.parse::<f64>().ok().is_some_and(f64::is_finite),
                    _ => true,
                };
                if writes {
                    self.once(|| self.ops.a11y(&event, self.current_key_step()));
                }
            });
        }
        let state = control.state();
        let focused = memo(move || state.get().input.focused);
        register_focus(shared, focused);
        control = control
            .disabled(move || shared.disabled.with(|d| d.is_some_and(|d| d.get())))
            .reference(self.edit.thumb);
        if role == Role::Slider {
            let grip = Grip::Thumb(self.edit.index);
            control = shared
                .touch_source(control, grip)
                .with(shared.drag.source(grip));
        }
        control
    }
}
