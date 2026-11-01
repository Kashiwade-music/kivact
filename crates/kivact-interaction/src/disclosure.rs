//! Role: Disclosure attaches an expandable trigger and its optionally delayed content.
//!
//! A disclosure is the first link of the opening chain: a popup
//! (`crate::popup`) adds overlay rules to it, and a tooltip and a dialog add
//! theirs to a popup. It keeps the requested state (the caller's
//! `Binding<bool>`) apart from the shown state: with a delay, each
//! false-to-true change arms one host-clock deadline as a task timer, with no
//! polling; going back to false or disposing the owner cancels it and hides
//! at once, and [`Disclosure::close`] keeps the content hidden until the next
//! false-to-true change (without a delay it writes false to the binding).
//! The parts are the trigger and the content, with [`DisclosureState`]
//! (`input`, `open`). Whether the content is unmounted with `show` or kept
//! mounted and hidden is the caller's choice; the kits' toggles and
//! accordions share these rules. A content that animates its closing (an
//! accordion's height) stays visible while [`Disclosure::leaving`] says the
//! animation runs; the trigger's `expanded` follows the shown state at once.

use std::time::Duration;

use kivact_core::primitive::{Part, PartState, part};
use kivact_core::task::{Task, sleep, spawn};
use kivact_core::view::{A11yAction, IntoValue, NodeRef, Role, node_ref};
use kivact_core::{Binding, Signal, Value, Var, effect, on_cleanup, signal, var};

/// The shown state and actual input state of a disclosure's trigger or content.
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DisclosureState {
    pub input: PartState,
    /// The content is shown (after the delay, when there is one).
    pub open: bool,
}

/// One host-clock deadline from the requested state to the shown state.
#[derive(Clone, Copy)]
struct Deadline {
    shown: Signal<bool>,
    generation: Var<u64>,
    pending: Var<Option<Task>>,
}

impl Deadline {
    fn cancel(self) {
        self.generation
            .update(|generation| *generation = generation.wrapping_add(1));
        if let Some(task) = self.pending.get() {
            task.cancel();
            self.pending.set(None);
        }
    }
}

/// Copy handle for a collapsible region: a trigger and the content it shows.
///
/// The disclosure edits one `bool`, the requested open state, through the
/// caller's [`Binding`]. The caller draws the trigger and the content and
/// decides whether the content is unmounted (`show(d.shown(), ..)`) or kept
/// mounted; [`content`](Self::content) hides a mounted content while closed.
///
/// The [`trigger`](Self::trigger) is a focusable `Role::Button` publishing
/// `expanded` and `controls` to the content. A click, Enter, Space and the
/// accessibility actions `Activate` and `Toggle` flip the state, while `Expand`
/// sets it and `Collapse` clears it. A [`disabled`](Self::disabled)
/// disclosure accepts no input or assistive technology action.
///
/// With a [`delay`](Self::delay), the shown state follows the requested state
/// late: each false-to-true change starts one host-clock deadline, and going
/// back to false cancels it and hides at once. No frame loop is used.
///
/// ```no_run
/// use kivact_core::{signal, control::{NodeBuilder, column, node, text}, view::show};
/// use kivact_interaction::use_disclosure;
/// let open = signal(false);
/// let details = use_disclosure(open);
/// let section = column((
///     node().part(details.trigger()).name("Details").child(text("Details")),
///     show(details.shown(), move || node().part(details.content()).child(text("The details"))),
/// ));
/// ```
#[derive(Clone, Copy)]
pub struct Disclosure {
    pub(crate) open: Binding<bool>,
    disabled: Option<Value<bool>>,
    shown: Value<bool>,
    deadline: Option<Deadline>,
    leaving: Option<Value<bool>>,
    pub(crate) content: NodeRef,
}

/// Create a disclosure whose requested state is `open`, the sole writer.
///
/// Creation does not write the state. Setup only.
#[track_caller]
pub fn use_disclosure(open: impl Into<Binding<bool>>) -> Disclosure {
    let open = open.into();
    Disclosure {
        open,
        disabled: None,
        shown: open.into_value(),
        deadline: None,
        leaving: None,
        content: node_ref(),
    }
}

impl Disclosure {
    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the trigger disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = Some(disabled.into_value());
        self
    }

    /// Show the content `delay` after the requested state becomes true.
    ///
    /// Each false-to-true transition starts one host-clock deadline; returning
    /// to false cancels it and hides the content. Zero delay is immediate. The
    /// deadline and its task belong to the setup owner and are removed on
    /// cleanup. Call it once, before creating parts. Setup only.
    #[track_caller]
    pub fn delay(mut self, delay: Duration) -> Self {
        let requested = self.open;
        let deadline = Deadline {
            shown: signal(false),
            generation: var(0),
            pending: var(None),
        };
        effect(move || {
            let engaged = requested.get();
            deadline.cancel();
            deadline.shown.set(false);
            if !engaged {
                return;
            }
            if delay.is_zero() {
                deadline.shown.set(true);
                return;
            }
            let generation = deadline.generation.get();
            let task = spawn(async move {
                sleep(delay).await;
                if deadline.generation.get() == generation && requested.get() {
                    deadline.shown.set(true);
                }
            });
            deadline.pending.set(Some(task));
        });
        on_cleanup(move || deadline.cancel());
        self.shown = deadline.shown.into_value();
        self.deadline = Some(deadline);
        self
    }

    /// Keep the [`content`](Self::content) visible after it stops being shown
    /// while `leaving` is true: the caller's closing animation (a height that
    /// collapses) still runs. The trigger's `expanded` and the content's
    /// [`DisclosureState::open`] follow the shown state at once. Call it
    /// before creating the content part. Setup only.
    pub fn leaving(mut self, leaving: impl IntoValue<bool>) -> Self {
        self.leaving = Some(leaving.into_value());
        self
    }

    /// Whether the content is shown: the requested state, after the delay.
    pub fn shown(self) -> Value<bool> {
        self.shown
    }

    /// Hide the content now.
    ///
    /// Without a delay this writes false to the binding. With one it cancels
    /// the deadline and keeps the content hidden until the requested state goes
    /// false and then true again, without writing the binding.
    #[track_caller]
    pub fn close(self) {
        match self.deadline {
            Some(deadline) => {
                // Validate purity and borrowing before changing the pending deadline.
                deadline.shown.set(false);
                deadline.cancel();
            }
            None => self.open.set(false),
        }
    }

    fn write(self, open: Option<bool>) {
        if self.disabled.is_some_and(|d| d.get()) {
            return;
        }
        self.open.set(open.unwrap_or_else(|| !self.open.get()));
    }

    /// Trigger semantics shared with the popups built on a disclosure.
    #[track_caller]
    pub(crate) fn base_trigger<S: Clone + PartialEq + 'static>(
        self,
        state: impl Fn(PartState, bool) -> S + 'static,
    ) -> Part<S> {
        let shown = self.shown;
        let trigger = part(move |input| state(input, shown.get()))
            .role(Role::Button)
            .expanded(shown)
            .controls(self.content);
        match self.disabled {
            Some(disabled) => trigger
                .disabled(disabled)
                .focusable(move || !disabled.get()),
            None => trigger.focusable(true),
        }
    }

    /// The trigger: a button publishing `expanded` that toggles the state.
    ///
    /// The trigger's name, geometry and drawing stay the caller's. Activate,
    /// Toggle, Expand and Collapse run after the caller's own handlers.
    #[track_caller]
    pub fn trigger(self) -> Part<DisclosureState> {
        self.base_trigger(|input, open| DisclosureState { input, open })
            .on_press(move || self.write(None))
            .on_a11y(A11yAction::Toggle, move || self.write(None))
            .on_a11y(A11yAction::Expand, move || self.write(Some(true)))
            .on_a11y(A11yAction::Collapse, move || self.write(Some(false)))
    }

    /// The content the trigger `controls`, hidden while not shown (nor
    /// [`leaving`](Self::leaving)).
    #[track_caller]
    pub fn content(self) -> Part<DisclosureState> {
        let shown = self.shown;
        let leaving = self.leaving;
        part(move |input| DisclosureState {
            input,
            open: shown.get(),
        })
        .reference(self.content)
        .hidden(move || !shown.get() && !leaving.is_some_and(|leaving| leaving.get()))
    }
}
