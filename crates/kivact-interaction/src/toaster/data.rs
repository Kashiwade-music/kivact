//! Role: Notification content, queue entries and focus attachments.
use super::timer::Machine as TimerMachine;
use crate::Tone;
use kivact_core::UiError;
use kivact_core::custom::Generation;
use kivact_core::state::{MachineHandle, Value};
use kivact_core::view::{HandlerResult, IntoView};
use std::rc::Rc;
use std::time::Duration;
/// How long a toast stays when [`Toast::duration`] is not set.
const DEFAULT_DURATION: Duration = Duration::from_secs(5);

/// How many toasts are shown at once when [`crate::Toaster::max_visible`] is not set.

pub(super) type Entries = Vec<(ToastId, Toast)>;
pub(super) type DismissHandler =
    Rc<dyn Fn(kivact_core::view::DismissReason) -> Result<(), UiError>>;

#[derive(Clone)]
pub(super) struct QueuedToast {
    pub id: ToastId,
    pub toast: Toast,
    pub options: ToastOptions,
    pub focus: FocusAttachment,
}

impl PartialEq for QueuedToast {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.focus == other.focus
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum FocusAttachment {
    Detached,
    Attached(ToastFocus),
}
impl FocusAttachment {
    pub(super) fn attached(self) -> Option<ToastFocus> {
        match self {
            Self::Detached => None,
            Self::Attached(focus) => Some(focus),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ToastFocus {
    pub(super) within: Value<bool>,
    pub(super) state: MachineHandle<TimerMachine>,
}

impl PartialEq for ToastFocus {
    fn eq(&self, other: &Self) -> bool {
        self.within == other.within && self.state.state() == other.state.state()
    }
}

/// Optional deduplication and dismissal behavior for a toast.
#[derive(Clone, Default)]
pub struct ToastOptions {
    pub(super) key: Option<String>,
    pub(super) on_dismiss: Option<DismissHandler>,
}

impl ToastOptions {
    /// Give the toast a key unique among the shown and waiting toasts.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Run `on_dismiss` once after a shown toast is removed from the queue.
    /// Removing a waiting toast does not run this callback. The closure belongs
    /// to the toast, so it can be set from a handler. Errors are reported
    /// through the App queue owner.
    pub fn on_dismiss<R: HandlerResult>(
        mut self,
        on_dismiss: impl Fn(kivact_core::view::DismissReason) -> R + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(move |reason| {
            on_dismiss(reason).into_outcome().map(|_| ())
        }));
        self
    }
}

/// The identity of a pushed toast, to [`dismiss`](crate::Toaster::dismiss) it or to
/// tell the toasts a kit draws apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(pub(super) Generation);

// xtask-allow(state-fields): Immutable public notification content and independent optional presentation, never lifecycle state.
/// The content of one notification: what it says, how urgent it is, how long
/// it stays and an optional action.
///
/// A toast is plain data built before [`crate::Toaster::push`]; it does not change
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
    /// ([`crate::Toaster::hold_while`]). `None` keeps it
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
}
