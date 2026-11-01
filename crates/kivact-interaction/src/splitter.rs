//! Role: Unstyled behavior and semantics of a splitter between two panes.
//!
//! [`use_splitter`] edits the leading pane's size with the same normalized
//! writer as a [`Scalar`], and its drag is a window-coordinate
//! [`Drag`](crate::Drag) made one edit with `Drag::edit`. Every kit's grip,
//! pane, dock split and column resize shares this part instead of copying
//! its keys: the table's [`resize`](crate::Table::resize) is a splitter, and
//! [`SplitterApi::handle`] is what `kivact::ui::splitter` draws. The cursor is
//! the caller's style.

use super::drag::{DragCoordinates, DragUpdate, use_drag};
use super::edit::{EditPhase, EditSession};
use super::scalar::{IntoScalarRange, Scalar, follow_range};
use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use kivact_core::primitive::{Part, PartState, part};
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    A11yAction, A11yEvent, Handled, IntoValue, Key, KeyEvent, MouseButton, Orientation,
    PointerEvent, PointerPhase, Role,
};
use kivact_core::{Binding, Value, var};

/// Reactive drawing state of an attached splitter handle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitterState {
    pub input: PartState,
    /// The size of the leading pane.
    pub value: f64,
    /// A pointer drag of this handle is in progress.
    pub dragging: bool,
}

/// Copy handle for the behavior and semantics of a splitter between two panes.
///
/// The splitter edits one number: the size of the leading pane (left in a row,
/// top in a column) in the caller's unit. The caller keeps drawing, geometry,
/// the cursor and the pane layout; the [`handle`](Self::handle) part adds the
/// role, value semantics, keyboard, accessibility and pointer behavior to the
/// element it is attached to.
///
/// `axis` is the direction the panes are laid out in, so `Orientation::Horizontal`
/// means side by side and the splitter moves along x.
/// `Orientation::Unspecified` is not a layout direction and is treated like
/// `Orientation::Horizontal`.
///
/// Semantics follow the ARIA convention that `orientation` is the direction of
/// the separator line, which is perpendicular to the way it moves: panes laid
/// out side by side (`Orientation::Horizontal`) publish `Orientation::Vertical`, and
/// panes stacked top to bottom (`Orientation::Vertical`) publish
/// `Orientation::Horizontal`. The cursor is the caller's style, conventionally
/// `Cursor::EwResize` for side by side panes and `Cursor::NsResize` for
/// stacked panes.
///
/// While focused, side by side panes use ArrowLeft (shrink) and ArrowRight
/// (grow); stacked panes use ArrowUp (shrink) and ArrowDown (grow), because the
/// leading pane is on top. In a right-to-left subtree
/// ([`current_direction`](kivact_core::view::current_direction)) side by side
/// panes lead from the right: the caller lays the leading pane out on the
/// right, ArrowRight shrinks it, ArrowLeft grows it, and a drag to the left
/// grows it. The arrows of the other axis are not handled. Home
/// and End jump to the range's minimum and maximum, and Enter moves to the
/// [`reset_to`](Self::reset_to) value, if any. Every write is normalized by the
/// [`ScalarRange`] and offered to the binding's writer, exactly as
/// [`Scalar::set`] does. Increment, Decrement and SetValue from assistive
/// technology take the same path.
///
/// A primary-button drag in window coordinates moves the value by the pointer's
/// travel along the axis, in pixels times [`pointer_scale`](Self::pointer_scale),
/// from the value at the start of the drag. Escape during a drag restores that
/// starting value and ends the drag, which needs the handle to be focused (a
/// press on a focusable handle focuses it). A double click moves to the
/// `reset_to` value. A disabled splitter accepts no input or assistive
/// technology action and cancels a drag in progress.
///
/// The range may follow the layout: pass a constant [`ScalarRange`], or a
/// closure, a `Signal` or a `Value` of one ([`IntoScalarRange`]). The published
/// `value_min` and `value_max`, the key step (when not overridden), Home, End
/// and drag clamping all follow the current range. When the range changes and
/// the value is now outside it, the handle writes the value back inside
/// through the binding. A range that only changes while the value stays inside
/// writes nothing.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{control, prelude, style};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{ScalarRange, use_splitter};
/// use kivact::prelude::*;
/// use kivact::style::Cursor;
/// let sidebar = signal(240.0);
/// let range = ScalarRange::new(120.0, 480.0).unwrap();
/// let handle = use_splitter(sidebar, range, Orientation::Horizontal)
///     .key_step(10.0)
///     .unwrap()
///     .reset_to(Some(240.0))
///     .handle();
/// let layout = row((
///     node().style(move || style().width(sidebar.get())),
///     node().part(handle).name("Sidebar width").style(
///         style()
///             .width(5)
///             .cursor(Cursor::EwResize)
///             .background(ColorToken::Border),
///     ),
///     node().style(style().grow(1)),
/// ));
/// ```
#[derive(Clone, Copy)]
pub struct SplitterApi {
    binding: Binding<f64>,
    range: Value<ScalarRange>,
    axis: Orientation,
    key_step: Option<f64>,
    reset: Option<Value<Option<f64>>>,
    scale: Option<Value<f64>>,
    disabled: Option<Value<bool>>,
    edit: EditSession<f64>,
}

/// Use this binding as the sole writer of the leading pane's size.
///
/// `value` is the size of the leading pane (left in a row, top in a column)
/// in the caller's unit. `range` bounds it, and may be reactive (see
/// [`SplitterApi`]). `axis` is the direction the panes are laid out in:
/// `Orientation::Horizontal` is side by side and the splitter moves along x.
/// Creation does not write state; a range given as a closure becomes a memo.
/// Setup only.
#[track_caller]
pub fn use_splitter(
    value: impl Into<Binding<f64>>,
    range: impl IntoScalarRange,
    axis: Orientation,
) -> SplitterApi {
    let binding = value.into();
    SplitterApi {
        binding,
        range: range.into_range(),
        axis,
        key_step: None,
        reset: None,
        scale: None,
        disabled: None,
        edit: EditSession::new(move |start| binding.set(start)),
    }
}

impl SplitterApi {
    /// Override the positive keyboard and accessibility increment.
    ///
    /// The default is the current range's step, or one percent of the range
    /// when it is continuous. A non-finite or non-positive step is rejected as
    /// [`ScalarRange::with_step`] and [`Scalar::key_step`] do.
    pub fn key_step(mut self, step: f64) -> Result<Self, ScalarRangeError> {
        if !step.is_finite() || step <= 0.0 {
            return Err(ScalarRangeError::InvalidStep);
        }
        self.key_step = Some(step);
        Ok(self)
    }

    /// The value Enter and a double click move to. `None` ignores both.
    ///
    /// Setup only.
    #[track_caller]
    pub fn reset_to(mut self, value: impl IntoValue<Option<f64>>) -> Self {
        self.reset = Some(value.into_value());
        self
    }

    /// Units of the value per pixel of pointer travel. The default is 1.0, so
    /// the value is in pixels.
    ///
    /// A closure can follow the layout, for example `100.0 / width` when the
    /// value is a percentage of a measured container. The scale is read when the
    /// pointer moves. Setup only.
    #[track_caller]
    pub fn pointer_scale(mut self, units_per_px: impl IntoValue<f64>) -> Self {
        self.scale = Some(units_per_px.into_value());
        self
    }

    /// Ignore input and assistive technology while true, and cancel a drag in
    /// progress.
    ///
    /// Also marks the handle disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = Some(disabled.into_value());
        self
    }

    /// Report the start and end of each edit as an [`EditPhase`], so the app
    /// can make one undo step of it.
    ///
    /// A drag is one edit: `Begin` when the pointer is pressed, `Commit` when it
    /// is released. Escape during the drag, or the engine canceling the pointer,
    /// writes the value from the start of the drag back and reports `Cancel`.
    /// Each arrow, Home, End or Enter key press and each assistive technology
    /// action is its own edit: `Begin`, the write, `Commit`. A double click
    /// that resets the value is one such edit too. Disabling the splitter during
    /// a drag ends the drag where it is, which commits it; disposing the owner
    /// during a drag cancels it. [`EditPhase`] has the rules every component
    /// follows. Setup only.
    pub fn on_edit<R: HandlerResult>(self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.edit.on_edit(f);
        self
    }

    fn is_disabled(self) -> bool {
        self.disabled.is_some_and(|d| d.get_untracked())
    }

    /// Whether the panes are side by side.
    fn side_by_side(self) -> bool {
        !matches!(self.axis, Orientation::Vertical)
    }

    /// The scalar operations for the range as it is now.
    fn scalar(self) -> Scalar {
        let range = self.range.get_untracked();
        let default_step = range.step().unwrap_or((range.max() - range.min()) / 100.0);
        let step = self
            .key_step
            .unwrap_or(if default_step.is_finite() && default_step > 0.0 {
                default_step
            } else {
                1.0
            });
        Scalar {
            binding: self.binding,
            range: self.range,
            key_step: Some(step),
            disabled: None,
            edit: None,
        }
    }

    /// Create the semantic and input attachment of the boundary between the
    /// panes. Read its state before moving it.
    ///
    /// The part sets the role, focusability, value range and orientation. Its
    /// key and pointer handlers run after the caller's own, so a caller handler
    /// can stop them. Semantic modifiers written after the attachment override
    /// the part's. Setup only.
    #[track_caller]
    pub fn handle(self) -> Part<SplitterState> {
        let value = self.binding.into_value();
        let (range, side_by_side) = (self.range, self.side_by_side());
        let rtl = side_by_side
            && kivact_core::view::current_direction() == kivact_core::view::ReadingDirection::Rtl;
        follow_range(self.binding, range);
        let edit = self.edit;
        // One key press or assistive technology action is a whole edit.
        let once = move |write: &dyn Fn(Scalar) -> bool| -> bool {
            let mut accepted = false;
            edit.once(value.get_untracked(), || accepted = write(self.scalar()));
            accepted
        };
        // Only an action that will write is an edit: a value that is not a
        // number reports nothing.
        let act = move |event: A11yEvent| {
            let writes = match event.action {
                A11yAction::SetValue => event
                    .value
                    .as_deref()
                    .and_then(|v| v.parse::<f64>().ok())
                    .is_some_and(f64::is_finite),
                _ => true,
            };
            if writes && !self.is_disabled() {
                once(&|scalar| scalar.a11y(&event));
            }
        };
        let reset = move || -> bool {
            self.reset
                .and_then(|reset| reset.get_untracked())
                .is_some_and(|reset| once(&|scalar| scalar.set(reset)))
        };
        // The value at the press; external writes during the drag do not move it.
        let origin = var(0.0);
        let session = use_drag::<()>(DragCoordinates::Window);
        let session = match self.disabled {
            Some(disabled) => session.disabled(disabled),
            None => session,
        }
        .edit(edit, move || value.get_untracked())
        .on_start(move |_| origin.set(value.get()))
        .on_update(move |update: DragUpdate<()>| {
            let travel = match (side_by_side, rtl) {
                (true, false) => update.delta.x,
                (true, true) => -update.delta.x,
                (false, _) => update.delta.y,
            };
            let scale = self.scale.map_or(1.0, |scale| scale.get());
            self.scalar().set(origin.get() + f64::from(travel) * scale);
        });
        let source = session.source(());
        let dragging = source.state();
        let (shrink_key, grow_key) = match (side_by_side, rtl) {
            (true, false) => (Key::ArrowLeft, Key::ArrowRight),
            (true, true) => (Key::ArrowRight, Key::ArrowLeft),
            (false, _) => (Key::ArrowUp, Key::ArrowDown),
        };
        let mut handle = part(move |input| SplitterState {
            input,
            value: value.get(),
            dragging: dragging.get().active,
        })
        .role(Role::Splitter)
        .orientation(if side_by_side {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        })
        .focusable(true)
        .value_min(move || range.get().min())
        .value_max(move || range.get().max())
        .value_now(value)
        .on_key(move |event: KeyEvent| {
            if self.is_disabled() {
                return Handled::No;
            }
            let accepted = match event.key {
                key if key == shrink_key => once(&|scalar| scalar.step_by(-1.0)),
                key if key == grow_key => once(&|scalar| scalar.step_by(1.0)),
                Key::Home => once(&|scalar| scalar.set(range.get_untracked().min())),
                Key::End => once(&|scalar| scalar.set(range.get_untracked().max())),
                Key::Enter => reset(),
                _ => false,
            };
            if accepted { Handled::Yes } else { Handled::No }
        })
        // A double click resets before the drag source sees the press.
        .on_pointer(move |event: PointerEvent| {
            if event.phase == PointerPhase::Down
                && event.button == Some(MouseButton::Primary)
                && event.click_count > 1
                && !self.is_disabled()
                && !session.is_active()
                && reset()
            {
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .on_a11y_event(A11yAction::Increment, act)
        .on_a11y_event(A11yAction::Decrement, act)
        .on_a11y_event(A11yAction::SetValue, act);
        if let Some(disabled) = self.disabled {
            handle = handle.disabled(disabled);
        }
        handle.with(source)
    }
}
