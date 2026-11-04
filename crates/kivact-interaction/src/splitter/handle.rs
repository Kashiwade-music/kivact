//! Role: Attach splitter input, edit ownership, and semantics to a handle.
use super::machine;
use super::options::SplitterOptions;
use crate::EditSession;
use crate::axis::Axis;
use crate::drag::{DragCoordinates, DragUpdate, use_drag};
use crate::options_cell::OptionsCell;
use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use crate::press::is_double_press;
use crate::scalar::{ScalarOps, follow_range};
use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{MachineHandle, Signal, Value, machine, memo, signal};
use kivact_core::view::EditPhase;
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    A11yAction, A11yEvent, A11yRequest, Handled, Key, KeyEvent, Orientation, PointerEvent, Role,
};

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
/// `Orientation::Unspecified` is not a layout direction and is a usage error.
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
/// ([`use_direction`](kivact_core::view::use_direction)) side by side
/// panes lead from the right: the caller lays the leading pane out on the
/// right, ArrowRight shrinks it, ArrowLeft grows it, and a drag to the left
/// grows it. The arrows of the other axis are not handled. Home
/// and End jump to the range's minimum and maximum. Enter collapses the
/// leading pane to its minimum size and restores its previous size on the
/// next press. Every write is normalized by the
/// [`ScalarRange`](crate::ScalarRange) and offered to the binding's writer, exactly as
/// [`ScalarApi::set`](crate::ScalarApi::set) does. Increment, Decrement and SetValue from assistive
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
/// The range may follow the layout: pass a constant [`ScalarRange`](crate::ScalarRange), or a
/// closure, a `Signal` or a `Value` of one ([``]). The published
/// `value_min` and `value_max`, the key step (when not overridden), Home, End
/// and drag clamping all follow the current range. When the range changes and
/// the value is now outside it, the handle writes the value back inside
/// through the binding. A range that only changes while the value stays inside
/// writes nothing.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Callback, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, Component, IntoView, NodeBuilder, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
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
    observed: Signal<Option<Value<SplitterState>>>,
    ops: ScalarOps,
    axis: Axis,
    settings: OptionsCell<SplitterOptions>,
    machine: MachineHandle<machine::Machine>,
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
    value: impl kivact_core::state::IntoBinding<f64>,
    range: impl IntoValue<ScalarRange>,
    axis: Orientation,
) -> SplitterApi {
    let binding = value.into_binding();
    let range = range.into_value();
    let settings = OptionsCell::new(SplitterOptions::default());
    let ops = ScalarOps { binding, range };
    let state = machine(
        machine::State {
            panel: machine::PanelSize::Expanded(binding.get()),
            drag: machine::Drag::Idle,
        },
        move || machine::Machine {
            options: settings.get().snapshot(),
            range: range.get_untracked(),
            value: binding.get(),
        },
        move |_, effect| match effect {
            machine::Effect::Write(value) => {
                ops.set(value);
            }
        },
    );
    state.feed(binding.into_value(), machine::Event::Observe);
    SplitterApi {
        observed: signal(None),
        ops,
        axis: Axis::of(axis, "use_splitter"),
        settings,
        machine: state,
        edit: EditSession::new(move |start| binding.set(start)),
    }
}

impl SplitterApi {
    /// Drawing state of the actual mounted handle.
    pub fn control_state(self) -> Value<SplitterState> {
        memo(move || {
            self.observed
                .get()
                .map(|state| state.get())
                .unwrap_or(SplitterState {
                    input: PartState::default(),
                    value: self.ops.binding.get(),
                    dragging: false,
                })
        })
    }

    /// Override the positive keyboard and accessibility increment.
    ///
    /// The default is the current range's step, or one percent of the range
    /// when it is continuous. A non-finite or non-positive step is rejected as
    /// [`ScalarRange::with_step`](crate::ScalarRange::with_step) and [`ScalarApi::key_step`](crate::ScalarApi::key_step) do.
    /// Setup only.
    #[track_caller]
    pub fn key_step(self, step: impl Into<f64>) -> Result<Self, ScalarRangeError> {
        let step = step.into();
        if !step.is_finite() || step <= 0.0 {
            return Err(ScalarRangeError::InvalidStep);
        }
        Ok(self.key_step_value(step.into_value()))
    }

    /// The keyboard increment as a value that may change (the table's resize
    /// limits). The step is not validated here.
    pub(crate) fn key_step_value(self, step: Value<f64>) -> Self {
        self.settings
            .update(|settings| settings.key_step = Some(step));
        self
    }

    /// Multiply the step for shifted arrow keys (used by column resizers).
    pub(crate) fn shift_step_factor(self, factor: f64) -> Self {
        self.settings
            .update(|settings| settings.shift_step_factor = Some(factor));
        self
    }

    /// The value a double click moves to. `None` ignores double clicks.
    ///
    /// Setup only.
    #[track_caller]
    pub fn reset_to(self, value: impl IntoValue<Option<f64>>) -> Self {
        let value = value.into_value();
        self.settings
            .update(|settings| settings.reset = Some(value));
        self
    }

    /// Units of the value per pixel of pointer travel. The default is 1.0, so
    /// the value is in pixels.
    ///
    /// A closure can follow the layout, for example `100.0 / width` when the
    /// value is a percentage of a measured container. The scale is read when the
    /// pointer moves. Setup only.
    #[track_caller]
    pub fn pointer_scale(self, units_per_px: impl IntoValue<f64>) -> Self {
        let scale = units_per_px.into_value();
        self.settings
            .update(|settings| settings.scale = Some(scale));
        self
    }

    /// Ignore input and assistive technology while true, and cancel a drag in
    /// progress.
    ///
    /// Also marks the handle disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        let disabled = disabled.into_value();
        self.settings
            .update(|settings| settings.disabled = Some(disabled));
        self
    }

    /// Collapse on release below this fraction of the current range (0 through 1).
    /// Setup only.
    #[track_caller]
    pub fn collapsible(self, threshold: impl Into<f64>) -> Self {
        let threshold = threshold.into();
        if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
            kivact_core::custom::raise(
                kivact_core::ErrorKind::Message,
                "SplitterApi::collapsible: threshold must be finite and between 0 and 1",
            );
        }
        self.settings
            .update(|options| options.collapse = Some(threshold));
        self
    }

    /// Snap on release to the nearest finite fraction in the current range.
    /// Stops outside 0 through 1 are ignored. Setup only.
    #[track_caller]
    pub fn snaps(self, stops: impl IntoValue<Vec<f64>>) -> Self {
        let stops = stops.into_value();
        self.settings.update(|options| options.snaps = Some(stops));
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
        self.settings
            .get()
            .disabled
            .is_some_and(|d| d.get_untracked())
    }

    /// Whether the panes are side by side.
    fn side_by_side(self) -> bool {
        self.axis.is_horizontal()
    }

    /// The keyboard increment as it is now: the one given to
    /// [`key_step`](Self::key_step), else the current range's step, else one
    /// percent of the range.
    fn current_key_step(self) -> f64 {
        if let Some(step) = self
            .settings
            .get()
            .key_step
            .map(|step| step.get_untracked())
        {
            return step;
        }
        let range = self.ops.range.get_untracked();
        let default_step = range.step().unwrap_or((range.max() - range.min()) / 100.0);
        if default_step.is_finite() && default_step > 0.0 {
            default_step
        } else {
            1.0
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
        let value = self.ops.binding.into_value();
        let (range, side_by_side) = (self.ops.range, self.side_by_side());
        let rtl = side_by_side
            && kivact_core::view::use_direction().get() == kivact_core::view::ReadingDirection::Rtl;
        follow_range(self.ops.binding, range);
        let edit = self.edit;
        // One key press or assistive technology action is a whole edit.
        let once = move |write: &dyn Fn(ScalarOps, f64) -> bool| -> bool {
            let mut accepted = false;
            edit.once(value.get_untracked(), || {
                accepted = write(self.ops, self.current_key_step());
            });
            accepted
        };
        // Only an action that will write is an edit: a value that is not a
        // number reports nothing.
        let act = move |event: A11yEvent| {
            let writes = match &event.request {
                A11yRequest::SetValue(v) => v.parse::<f64>().ok().is_some_and(f64::is_finite),
                _ => true,
            };
            if writes && !self.is_disabled() {
                once(&|ops, step| ops.a11y(&event, step));
            }
        };
        let reset = move || -> bool {
            self.settings
                .get()
                .reset
                .and_then(|reset| reset.get_untracked())
                .is_some_and(|reset| once(&|ops, _| ops.set(reset)))
        };
        let settings = self.settings;
        let machine = self.machine;
        let (shrink_key, grow_key) = match (side_by_side, rtl) {
            (true, false) => (Key::ArrowLeft, Key::ArrowRight),
            (true, true) => (Key::ArrowRight, Key::ArrowLeft),
            (false, _) => (Key::ArrowUp, Key::ArrowDown),
        };
        let input = part(move |input| SplitterState {
            input,
            value: value.get(),
            dragging: matches!(machine.get().drag, machine::Drag::Dragging { .. }),
        });
        let part_state = input.state();
        self.observed.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |mut host| {
            let caller_disabled = host.accessible_disabled();
            let options = settings.value();
            let disabled = memo(move || {
                caller_disabled.is_some_and(|value| value.get())
                    || options.get().disabled.is_some_and(|value| value.get())
            });
            machine.feed(disabled, machine::Event::Disabled);
            let session = use_drag::<()>(DragCoordinates::Window)
                .disabled(disabled)
                .edit(edit, move || value.get_untracked())
                .on_start(move |_| machine.send(machine::Event::Begin))
                .on_update(move |update: DragUpdate<()>| {
                    let travel = match (side_by_side, rtl) {
                        (true, false) => update.delta.x,
                        (true, true) => -update.delta.x,
                        (false, _) => update.delta.y,
                    };
                    machine.send(machine::Event::Move(travel));
                })
                .on_end(move |update| {
                    machine.send(machine::Event::End(update.phase == crate::DragPhase::End))
                });
            let source = session.source(());
            let handle = host
                .part(input)
                .node_ref(target)
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
                    let factor = if event.modifiers().shift {
                        self.settings.get().shift_step_factor.unwrap_or(1.0)
                    } else {
                        1.0
                    };
                    let accepted = match event.key() {
                        key if key == shrink_key => {
                            once(&|ops, step| ops.step_by(-1.0, step * factor))
                        }
                        key if key == grow_key => {
                            once(&|ops, step| ops.step_by(1.0, step * factor))
                        }
                        Key::Home => once(&|ops, _| ops.set(range.get_untracked().min())),
                        Key::End => once(&|ops, _| ops.set(range.get_untracked().max())),
                        Key::Enter => once(&|_, _| {
                            machine.send(machine::Event::Toggle);
                            true
                        }),
                        _ => false,
                    };
                    if accepted { Handled::Yes } else { Handled::No }
                })
                // A double click resets before the drag source sees the press.
                .on_pointer(move |event: PointerEvent| {
                    if is_double_press(&event)
                        && !self.is_disabled()
                        && !session.is_active()
                        && reset()
                    {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .a11y_action(A11yAction::Increment, act)
                .a11y_action(A11yAction::Decrement, act)
                .a11y_action(A11yAction::SetValue, act)
                .disabled(disabled);
            handle.part(source)
        })
    }
}
