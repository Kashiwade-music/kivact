//! Role: Toast notifications with dismissal control.
//!
//! Unstyled behavior and semantics of toast notifications: a queue for the
//! whole App, timed dismissal that waits for the pointer and the keyboard
//! focus, and live-region announcements.
//!
//! [`toaster`] returns the App's shared queue, operated with `push`,
//! `dismiss`, `toasts` and `max_visible`. A [`Toast`] carries its id, [`Tone`],
//! duration, action and icon. The action is a closure the toast owns (an
//! `Rc`, not an arena `Callback`), so a toast with an action can be built
//! outside setup, in a press handler say, and the closure is dropped with
//! the toast; its error or panic (a write to a signal of a component that is
//! gone) is reported by the action button's press handler. The caller mounts
//! the polite and the assertive region from the start, renders each
//! region's `visible(assertive)` ids with `item(id)` on each card, and reads
//! the data with `toast(id)`. A region is a `Role::Group`, a warning or error
//! item a `Role::Alert` and any other a `Role::Status`, named by the title.
//! The countdown is an owner-scoped machine timer that stops while the toast is really
//! hovered or holds the focus and resumes with what is left; a toast with no
//! duration is never dismissed by time and uses no frame loop.
//! [`ToasterApi::hold_while`] lets a kit that stacks the toasts stop every
//! countdown at once (the true holds add up and end with their owner).
//! Placement is the caller's layout: there is no placement type or region
//! component here. A kit keeps only the card, the placement and its
//! defaults for the duration and `max_visible`, and does not implement the
//! queue or the timers again.

use std::time::Duration;

use kivact_core::ErrorKind;
use kivact_core::UiError;
use kivact_core::custom::{Owner, Part, PartState, part, raise};

use super::column::{Event as ColumnEvent, HoldId, Machine as ColumnMachine, State as ColumnState};
use super::timer::{
    Effect as TimerEffect, Event as TimerEvent, Machine as TimerMachine, State as TimerState,
};
use kivact_core::custom::Generation;
use kivact_core::state::IntoValue;
use kivact_core::state::{MachineHandle, machine};
use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, signal};
use kivact_core::view::{Handled, Key, KeyEvent, LiveRegion, Modifiers, Role};

use super::data::{
    Entries, FocusAttachment, QueuedToast, Toast, ToastFocus, ToastId, ToastOptions,
};
use crate::Tone;

impl Toast {
    /// Dismiss the toast `id`, then run the action, if any: what the action
    /// button a kit draws does when it is pressed. Return it from the press
    /// handler, which reports the action's error. Call it from a handler.
    pub fn activate(&self, id: ToastId) -> Result<(), UiError> {
        toaster().dismiss(id);
        match &self.action {
            Some((_, on)) => on.try_call(()),
            None => Ok(()),
        }
    }
}

/// The App-wide queue of notifications.
///
/// [`toaster`] returns the App's toaster, which the first call makes
/// under the App's root owner and which lives as long as the App; an App that
/// never calls it has none. The handle is `Copy` and can be used from setup,
/// handlers, tasks and effects.
///
/// [`push`](Self::push) adds a [`Toast`]. At most
/// [`max_visible`](Self::max_visible) (3 by default) are shown at once; the
/// rest wait in the queue, in the order they were pushed, and are shown as
/// the ones before them go. A waiting toast has not started its time.
///
/// The caller draws the shown toasts and places them in the window: two
/// [`region`](Self::region) parts (polite and assertive) that hold the rows
/// of [`visible`](Self::visible), each row with the [`item`](Self::item) part,
/// drawn from [`toast`](Self::toast). [`toasts`](Self::toasts) reads what is
/// shown without rendering it.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, Component, IntoView, NodeBuilder, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{Toast, Tone, toaster};
/// use kivact::prelude::*;
/// use kivact_core::style::Position;
/// use kivact_core::view::LiveRegion;
///
/// fn app() -> impl IntoView {
///     let toaster = toaster();
///     let group = move |assertive: bool, live: LiveRegion| {
///         column(for_each(toaster.visible(assertive), |id| *id, move |item| {
///             let id = item.get();
///             let title = toaster.toast(id).map(|toast| toast.title).unwrap_or_default();
///             node().part(toaster.item(id)).child(text(title))
///         }))
///         .part(toaster.region(live))
///     };
///     column((
///         node()
///             .role(Role::Button)
///             .name("Save")
///             .focusable(true)
///             .on_press(move || {
///                 toaster.push(Toast::new("Saved").tone(Tone::Success));
///             })
///             .child(text("Save")),
///         column((group(false, LiveRegion::Polite), group(true, LiveRegion::Assertive)))
///             .style(style().position(Position::Fixed).right(16).bottom(16)),
///     ))
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ToasterApi {
    /// Every toast not yet gone, shown or waiting, oldest first.
    column: MachineHandle<ColumnMachine>,
    /// The first `limit` of `queue`.
    visible: Value<Entries>,
    next: Signal<Generation>,
}

/// The App's toaster, made on the first call. It works wherever the App
/// runs code (setup, a handler, a task, an effect), whether or not the
/// caller still has an owner; it needs no component to be alive, since
/// the toaster belongs to the App. Outside a running App it raises a
/// located `WrongApp` error.
///
/// Unlike the `use_*` functions, which are for setup only, this is the
/// application's own name for the shared queue: call it from a handler
/// whenever a toast is due.
#[track_caller]
pub fn toaster() -> ToasterApi {
    kivact_core::state::use_app_state(|| {
        let column = machine(
            ColumnState::default(),
            || ColumnMachine,
            |_, effect| match effect {
                super::column::Effect::Dismiss {
                    previous,
                    callback,
                    reason,
                } => {
                    if let Some(previous) = previous {
                        previous.restore_focus();
                    }
                    if let Some(callback) = callback
                        && let Err(error) = callback(reason)
                    {
                        Owner::root().report(error.kind().clone(), error.to_string());
                    }
                }
            },
        );
        ToasterApi {
            column,
            visible: memo(move || {
                let state = column.get();
                state
                    .queue
                    .iter()
                    .take(state.limit)
                    .map(|entry| (entry.id, entry.toast.clone()))
                    .collect()
            }),
            next: signal(Generation::new()),
        }
    })
}

impl ToasterApi {
    /// Queue `toast`. It is shown at once when fewer than
    /// [`max_visible`](Self::max_visible) are shown, and when the ones before
    /// it have gone otherwise.
    pub fn push(self, toast: Toast) -> ToastId {
        self.push_with(toast, ToastOptions::default())
    }

    /// Queue a toast with options. If a toast with the supplied key is shown
    /// or waiting, return its id and leave its original content and timer in place.
    pub fn push_with(self, toast: Toast, options: ToastOptions) -> ToastId {
        if let Some(key) = options.key.as_deref()
            && let Some(id) = self.column.state().with_untracked(|state| {
                let queue = &state.queue;
                queue
                    .iter()
                    .find(|entry| entry.options.key.as_deref() == Some(key))
                    .map(|entry| entry.id)
            })
        {
            return id;
        }
        let id = ToastId(self.next.update(Generation::next));
        self.column.send(ColumnEvent::Push(QueuedToast {
            id,
            toast,
            options,
            focus: FocusAttachment::Detached,
        }));
        id
    }

    /// Remove a toast, shown or waiting. If focus is inside it, request its
    /// return to the last focused element outside the toast.
    pub fn dismiss(self, id: ToastId) {
        self.dismiss_for(id, kivact_core::view::DismissReason::Accessibility);
    }

    fn dismiss_for(self, id: ToastId, reason: kivact_core::view::DismissReason) {
        let Some((_index, focus)) = self.column.state().with_untracked(|state| {
            let queue = &state.queue;
            queue
                .iter()
                .enumerate()
                .find(|(_, entry)| entry.id == id)
                .map(|(index, entry)| (index, entry.focus))
        }) else {
            return;
        };
        let previous = focus
            .attached()
            .filter(|focus| focus.within.get_untracked())
            .and_then(|focus| focus.state.state().get_untracked().return_to());
        self.column.send(ColumnEvent::Remove(id, reason, previous));
    }

    /// The toasts shown now, oldest first.
    pub fn toasts(self) -> Value<Vec<(ToastId, Toast)>> {
        self.visible
    }

    /// How many toasts are shown at once; the rest wait. The default is 3. Showing
    /// none (`0`) is a usage error.
    #[track_caller]
    pub fn max_visible(self, n: usize) -> Self {
        if n == 0 {
            raise(
                ErrorKind::Message,
                "ToasterApi::max_visible: show at least one toast",
            );
        }
        self.column.send(ColumnEvent::Limit(n));
        self
    }

    /// Stop the countdown of every shown toast while `held` is true, on top
    /// of the pointer and focus rules of [`item`](Self::item): a kit that
    /// draws the toasts as one stack, which opens while the pointer is over
    /// any part of it, holds them all then. What each toast has left resumes
    /// when no hold and no toast pointer or focus remains. Several holds add
    /// up. The hold ends with the owner it was made in. Setup only.
    #[track_caller]
    pub fn hold_while(self, held: impl IntoValue<bool>) {
        let held = held.into_value();
        let id = HoldId(self.next.update(Generation::next));
        effect(move || self.column.send(ColumnEvent::Hold(id, held.get())));
        on_cleanup(move || self.column.send(ColumnEvent::Hold(id, false)));
    }

    /// The toast `id` if it is shown now.
    fn is_open(self, id: ToastId) -> Option<Toast> {
        self.visible.with_untracked(|visible| {
            visible
                .iter()
                .find(|(shown, _)| *shown == id)
                .map(|(_, toast)| toast.clone())
        })
    }

    fn track_toast(
        self,
        id: ToastId,
        within: Value<bool>,
        duration: Option<Duration>,
        engaged: Value<bool>,
    ) {
        let owner = Owner::current();
        let left_ns = duration.map_or(0, |duration| {
            duration.as_nanos().min(u128::from(u64::MAX)) as u64
        });
        let state = machine(
            TimerState::new(left_ns),
            move || TimerMachine {
                now_ns: now_ns(),
                timed: duration.is_some(),
            },
            move |_, effect| match effect {
                TimerEffect::Dismiss => {
                    self.dismiss_for(id, kivact_core::view::DismissReason::Timeout)
                }
            },
        );
        self.column.send(ColumnEvent::Focus(
            id,
            FocusAttachment::Attached(ToastFocus { within, state }),
        ));
        on_cleanup(move || {
            self.column
                .send(ColumnEvent::Focus(id, FocusAttachment::Detached))
        });
        effect(move || {
            state.send(TimerEvent::Observe {
                inside: within.get(),
                focused: owner.focused_node(),
                engaged: engaged.get(),
            })
        });
    }
}

/// The tone and actual input state of a toast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToastState {
    pub input: PartState,
    /// The toast's tone; [`Tone::Neutral`] once it is gone.
    pub tone: Tone,
    /// The pointer is over the toast or the keyboard focus is in it, or a
    /// [`ToasterApi::hold_while`] is true, which stops its countdown.
    pub engaged: bool,
    /// How many newer toasts are shown in the same live region: 0 for the
    /// newest, which a stacked look draws in front of the others.
    pub depth: usize,
}

impl ToasterApi {
    /// The toast `id` while it is shown, for drawing it.
    pub fn toast(self, id: ToastId) -> Option<Toast> {
        self.is_open(id)
    }

    /// The ids of the toasts shown now whose tone is announced assertively
    /// (`true`: [`Warning`](Tone::Warning) and [`Danger`](Tone::Danger))
    /// or politely (`false`), oldest first, for the caller's `for_each` in
    /// the matching [`region`](Self::region). Setup only.
    #[track_caller]
    pub fn visible(self, assertive: bool) -> Value<Vec<ToastId>> {
        let shown = self.visible;
        memo(move || shown.with(|shown| ids(shown, assertive)))
    }

    /// A live region for the toasts of one urgency: a `Role::Group` that
    /// announces [`LiveRegion::Polite`] or [`LiveRegion::Assertive`]
    /// additions. Mount both regions from the start, so that assistive
    /// technology knows them before the first toast appears, and render the
    /// toasts of [`visible`](Self::visible) inside. Placement in the window
    /// is the caller's layout.
    #[track_caller]
    pub fn region(self, live: LiveRegion) -> Part<PartState> {
        let input = part(|input| input);
        let part_state = input.state();
        Part::new(part_state, move |host| {
            host.part(input).role(Role::Group).live_region(live)
        })
    }

    /// One shown toast: a `Role::Status` (polite) or `Role::Alert`
    /// (assertive) named by its title.
    ///
    /// Its time is counted with an owner-scoped machine timer, not by frames, from when it is
    /// mounted. It stops while the pointer is over the toast and while the
    /// keyboard focus is on the toast or inside it, and what is left resumes
    /// when both end. Escape, while the focus is in the toast, dismisses it.
    /// A toast with a `duration` of `None` stays until it is dismissed. The
    /// caller draws a close button with [`dismiss`](Self::dismiss) and the
    /// action, if any, with [`Toast::activate`]. Setup only, in the toast's
    /// row.
    #[track_caller]
    pub fn item(self, id: ToastId) -> Part<ToastState> {
        self.item_with_hold(id, None)
    }

    /// Attach a toast with an additional, per-item reason to pause its timer.
    #[track_caller]
    pub fn item_while(self, id: ToastId, held: impl IntoValue<bool>) -> Part<ToastState> {
        self.item_with_hold(id, Some(held.into_value()))
    }

    fn item_with_hold(self, id: ToastId, held: Option<Value<bool>>) -> Part<ToastState> {
        let toast = self.is_open(id);
        let tone = toast.as_ref().map_or(Tone::Neutral, |toast| toast.tone);
        let duration = toast.as_ref().and_then(|toast| toast.duration);
        let title = toast.map(|toast| toast.title).unwrap_or_default();
        let target = kivact_core::view::node_ref();
        let hover = target.hovered();
        let focus = target.focus_within();
        let engaged = memo(move || {
            hover.get()
                || focus.get()
                || !self.column.get().holds.is_empty()
                || held.is_some_and(|held| held.get())
        });
        let shown = self.visible;
        let depth = memo(move || {
            shown.with(|shown| {
                let same = ids(shown, tone.is_urgent());
                same.iter()
                    .position(|other| *other == id)
                    .map_or(0, |at| same.len() - 1 - at)
            })
        });
        let input = part(move |input| ToastState {
            input,
            tone,
            engaged: engaged.get(),
            depth: depth.get(),
        });
        let part_state = input.state();
        Part::new(part_state, move |host| {
            let attachment = host.part(input).node_ref(target);
            let anchor = target;
            attachment
                .role(if tone.is_urgent() {
                    Role::Alert
                } else {
                    Role::Status
                })
                .name(title)
                .on_key(move |event: KeyEvent| {
                    if event.key() == Key::Escape && event.modifiers() == Modifiers::default() {
                        self.dismiss_for(id, kivact_core::view::DismissReason::Escape);
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .on_attach(move |_| {
                    self.track_toast(id, anchor.focus_within(), duration, engaged);
                })
        })
    }
}

/// The ids among `shown` whose tone is (not) assertive.
fn ids(shown: &[(ToastId, Toast)], assertive: bool) -> Vec<ToastId> {
    shown
        .iter()
        .filter(|(_, toast)| toast.tone.is_urgent() == assertive)
        .map(|(id, _)| *id)
        .collect()
}

fn now_ns() -> u64 {
    kivact_core::custom::clock_now()
}
