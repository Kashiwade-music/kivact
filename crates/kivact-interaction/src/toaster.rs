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
//! The countdown is a task timer that stops while the toast is really
//! hovered or holds the focus and resumes with what is left; a toast with no
//! duration is never dismissed by time and uses no frame loop.
//! [`Toaster::hold_while`] lets a kit that stacks the toasts stop every
//! countdown at once (the true holds add up and end with their owner).
//! Placement is the caller's layout: there is no placement type or region
//! component here. A kit keeps only the card, the placement and its
//! defaults for the duration and `max_visible`, and does not implement the
//! queue or the timers again.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use kivact_core::ErrorKind;
use kivact_core::UiError;
use kivact_core::custom::{NodeToken, Owner, Part, PartState, part, raise};

use kivact_core::state::IntoValue;
use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, signal};
use kivact_core::task::{sleep, spawn};
use kivact_core::view::{
    Handled, HandlerResult, IntoView, Key, KeyEvent, LiveRegion, Modifiers, Role,
};

use super::Tone;

/// How long a toast stays when [`Toast::duration`] is not set.
const DEFAULT_DURATION: Duration = Duration::from_secs(5);

/// How many toasts are shown at once when [`Toaster::max_visible`] is not set.
const DEFAULT_MAX_VISIBLE: usize = 3;

type Entries = Vec<(ToastId, Toast)>;

struct QueuedToast {
    id: ToastId,
    toast: Toast,
    options: ToastOptions,
    focus: Rc<Cell<Option<ToastFocus>>>,
}

#[derive(Clone, Copy)]
struct ToastFocus {
    within: Value<bool>,
    previous: Signal<Option<NodeToken>>,
}

/// Optional deduplication and dismissal behavior for a toast.
#[derive(Clone, Default)]
pub struct ToastOptions {
    key: Option<String>,
    on_dismiss: Option<kivact_core::state::Callback<kivact_core::view::DismissReason>>,
}

impl ToastOptions {
    /// Give the toast a key unique among the shown and waiting toasts.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Run `on_dismiss` once after a shown toast is removed from the queue.
    /// Removing a waiting toast does not run this callback.
    pub fn on_dismiss<R: HandlerResult>(
        mut self,
        on_dismiss: impl Fn(kivact_core::view::DismissReason) -> R + 'static,
    ) -> Self {
        self.on_dismiss = Some(kivact_core::state::callback(on_dismiss));
        self
    }
}

/// The identity of a pushed toast, to [`dismiss`](Toaster::dismiss) it or to
/// tell the toasts a kit draws apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(u64);

/// The content of one notification: what it says, how urgent it is, how long
/// it stays and an optional action.
///
/// A toast is plain data built before [`Toaster::push`]; it does not change
/// afterwards. Its fields are what the caller's drawing of a toast reads.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use std::time::Duration;
/// use kivact::interaction::{Toast, Tone};
///
/// let saved = Toast::new("Saved")
///     .body("The document was written to disk.")
///     .tone(Tone::Success)
///     .duration(Some(Duration::from_secs(3)));
/// assert_eq!(saved.title, "Saved");
/// ```
#[derive(Clone)]
pub struct Toast {
    /// What the toast says. It names the toast for assistive technology.
    pub title: String,
    /// The second line, if there is one.
    pub body: Option<String>,
    /// The tone.
    pub tone: Tone,
    /// How long it stays; `None` until it is dismissed.
    pub duration: Option<Duration>,
    /// The label and the function of the action, if there is one: owned by
    /// the toast, run when its action button is pressed.
    #[allow(
        clippy::type_complexity,
        reason = "a plain owned closure, named where it is read"
    )]
    pub action: Option<(String, kivact_core::state::Callback<()>)>,
    /// A view a kit draws as the toast's icon instead of the icon of its
    /// tone, if there is one. It is called once, when the toast is shown.
    pub icon: Option<kivact_core::view::ViewFn<()>>,
}

/// Toasts are equal when their text, tone and duration are, and their action
/// and icon are the same functions (a closure is equal to a clone of itself),
/// as for a [`ColumnDef`](crate::ColumnDef).
impl PartialEq for Toast {
    fn eq(&self, other: &Self) -> bool {
        let action = self.action == other.action;
        let icon = self.icon == other.icon;
        self.title == other.title
            && self.body == other.body
            && self.tone == other.tone
            && self.duration == other.duration
            && action
            && icon
    }
}

impl Toast {
    /// A toast that says `title`, [`Tone::Neutral`], gone after 5 seconds.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: None,
            tone: Tone::Neutral,
            duration: Some(DEFAULT_DURATION),
            action: None,
            icon: None,
        }
    }

    /// A second line below the title.
    pub fn body(mut self, text: impl Into<String>) -> Self {
        self.body = Some(text.into());
        self
    }

    /// The tone, which decides the announcement. The default is
    /// [`Tone::Neutral`].
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// How long the toast stays once it is shown, not counting the time the
    /// pointer is over it, the keyboard focus is in it or the toaster is held
    /// ([`Toaster::hold_while`]). `None` keeps it
    /// until it is dismissed. The default is 5 seconds.
    pub fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    /// One action button, such as "Undo", that calls `on` when it is pressed.
    /// The kit draws the button; the toast goes away after `on` has run. A
    /// second call replaces the first. `on` belongs to the toast and is
    /// dropped with it, so a toast can be made anywhere, a press handler
    /// included. `on` may return `()` or `Result<(), UiError>`; an error, or
    /// a panic such as writing a signal whose component is gone, is
    /// reported where the action button's press is.
    pub fn action(
        mut self,
        label: impl Into<String>,
        on: impl Into<kivact_core::state::Callback<()>>,
    ) -> Self {
        self.action = Some((label.into(), on.into()));
        self
    }

    /// An icon for this toast, which a kit draws instead of the icon of its
    /// tone. `icon` is called once, when the toast is shown, in the toast's
    /// owner.
    pub fn icon<V: IntoView>(mut self, icon: impl kivact_core::view::IntoViewFn<V>) -> Self {
        self.icon = Some(icon.into_view_fn());
        self
    }

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
pub struct Toaster {
    /// Every toast not yet gone, shown or waiting, oldest first.
    queue: Signal<Vec<QueuedToast>>,
    /// The first `limit` of `queue`.
    visible: Value<Entries>,
    next: Signal<u64>,
    limit: Signal<usize>,
    /// How many [`hold_while`](Self::hold_while) values are true now.
    holds: Signal<usize>,
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
pub fn toaster() -> Toaster {
    kivact_core::state::use_app_state(|| {
        let queue = signal(Vec::<QueuedToast>::new());
        let limit = signal(DEFAULT_MAX_VISIBLE);
        Toaster {
            queue,
            visible: memo(move || {
                let limit = limit.get();
                queue.with(|queue| {
                    queue
                        .iter()
                        .take(limit)
                        .map(|entry| (entry.id, entry.toast.clone()))
                        .collect()
                })
            }),
            next: signal(0u64),
            limit,
            holds: signal(0usize),
        }
    })
}

impl Toaster {
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
            && let Some(id) = self.queue.with_untracked(|queue| {
                queue
                    .iter()
                    .find(|entry| entry.options.key.as_deref() == Some(key))
                    .map(|entry| entry.id)
            })
        {
            return id;
        }
        let id = ToastId(self.next.get_untracked());
        self.next.set(id.0 + 1);
        self.queue.update(|queue| {
            queue.push(QueuedToast {
                id,
                toast,
                options,
                focus: Rc::new(Cell::new(None)),
            })
        });
        id
    }

    /// Remove a toast, shown or waiting. If focus is inside it, request its
    /// return to the last focused element outside the toast.
    pub fn dismiss(self, id: ToastId) {
        self.dismiss_for(id, kivact_core::view::DismissReason::Accessibility);
    }

    fn dismiss_for(self, id: ToastId, reason: kivact_core::view::DismissReason) {
        let Some((index, focus)) = self.queue.with_untracked(|queue| {
            queue
                .iter()
                .enumerate()
                .find(|(_, entry)| entry.id == id)
                .map(|(index, entry)| (index, entry.focus.get()))
        }) else {
            return;
        };
        let shown = index < self.limit.get_untracked();
        let previous = focus
            .filter(|focus| focus.within.get_untracked())
            .and_then(|focus| focus.previous.get_untracked());
        let entry = self.queue.update(|queue| queue.remove(index));
        if let Some(previous) = previous {
            previous.restore_focus();
        }
        if shown && let Some(on_dismiss) = entry.options.on_dismiss {
            on_dismiss.call(reason);
        }
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
                "Toaster::max_visible: show at least one toast",
            );
        }
        self.limit.set(n);
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
        effect(move || {
            if held.get() {
                self.holds.set(self.holds.get_untracked() + 1);
                on_cleanup(move || self.holds.set(self.holds.get_untracked().saturating_sub(1)));
            }
        });
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

    fn track_focus(self, id: ToastId, within: Value<bool>) {
        let slot = self.queue.with_untracked(|queue| {
            queue
                .iter()
                .find(|entry| entry.id == id)
                .map(|entry| entry.focus.clone())
        });
        let Some(slot) = slot else { return };
        let owner = Owner::current();
        let previous = signal(None);
        slot.set(Some(ToastFocus { within, previous }));
        on_cleanup(move || slot.set(None));
        effect(move || {
            let inside = within.get();
            let focused = owner.focused_node();
            if !inside && focused.is_some() {
                previous.set(focused);
            }
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
    /// [`Toaster::hold_while`] is true, which stops its countdown.
    pub engaged: bool,
    /// How many newer toasts are shown in the same live region: 0 for the
    /// newest, which a stacked look draws in front of the others.
    pub depth: usize,
}

impl Toaster {
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
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .role(Role::Group)
                .live_region(live)
        })
    }

    /// One shown toast: a `Role::Status` (polite) or `Role::Alert`
    /// (assertive) named by its title.
    ///
    /// Its time is counted with a task timer, not by frames, from when it is
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
        let engaged = signal(false);
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
        let target = kivact_core::view::node_ref();
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
                    let hover = anchor.hovered();
                    let focus = anchor.focus_within();
                    self.track_focus(id, focus);
                    let within = memo(move || hover.get() || focus.get());
                    let stopped = memo(move || {
                        within.get() || self.holds.get() > 0 || held.is_some_and(|held| held.get())
                    });
                    effect(move || engaged.set(stopped.get()));
                    count_down(self, id, duration, stopped);
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

/// DismissOn `id` after `duration` of time not `engaged`. The time already
/// counted is kept while it is engaged and the rest resumes after it.
fn count_down(toaster: Toaster, id: ToastId, duration: Option<Duration>, engaged: Value<bool>) {
    let Some(duration) = duration else { return };
    let left = Rc::new(Cell::new(
        duration.as_nanos().min(u128::from(u64::MAX)) as u64
    ));
    effect(move || {
        if engaged.get() {
            return;
        }
        let started = now_ns();
        let wait = Duration::from_nanos(left.get());
        let task = spawn(async move {
            sleep(wait).await;
            toaster.dismiss_for(id, kivact_core::view::DismissReason::Timeout);
        });
        let left = left.clone();
        on_cleanup(move || {
            task.cancel();
            left.set(left.get().saturating_sub(now_ns().saturating_sub(started)));
        });
    });
}

fn now_ns() -> u64 {
    kivact_core::custom::clock_now()
}
