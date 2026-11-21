//! Role: Notification content, queue entries and focus attachments.
use super::timer::Machine as TimerMachine;
use crate::Tone;
use kivact_core::UiError;
use kivact_core::custom::Generation;
use kivact_core::state::{MachineHandle, Value};
use kivact_core::view::HandlerResult;
use std::rc::Rc;
use std::time::Duration;
/// How long a toast stays when [`Toast::duration`] is not set.
const DEFAULT_DURATION: Duration = Duration::from_secs(5);

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

/// The identity of a pushed toast, to [`dismiss`](crate::ToasterApi::dismiss) it or to
/// tell the toasts a kit draws apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(pub(super) Generation);

/// The content of one notification: what it says, how urgent it is, how long
/// it stays and an optional action.
///
/// A toast is plain data built before [`crate::ToasterApi::push`]; it does not change
/// afterwards. Its fields are what the caller's drawing of a toast reads.
///
/// ```no_run
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
    /// The name in the theme's icon set of the icon a kit draws instead of
    /// the icon of its tone, if there is one.
    pub icon_name: Option<&'static str>,
}

/// Toasts are equal when their text, tone and duration are, their icon names
/// are and their action is the same function (a closure is equal to a clone
/// of itself), as for a [`ColumnDef`](crate::ColumnDef).
impl PartialEq for Toast {
    fn eq(&self, other: &Self) -> bool {
        let action = self.action == other.action;
        let icon = self.icon_name == other.icon_name;
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
            icon_name: None,
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
    /// ([`crate::ToasterApi::hold_while`]). `None` keeps it
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
    pub fn action<M>(
        mut self,
        label: impl Into<String>,
        on: impl kivact_core::state::IntoHandler<(), M>,
    ) -> Self {
        let on = kivact_core::state::Callback::from_handler(
            kivact_core::state::IntoHandler::into_handler(on),
        );
        self.action = Some((label.into(), on));
        self
    }

    /// The name in the theme's icon set of an icon drawn instead of the icon
    /// of its tone.
    pub fn icon_name(mut self, name: &'static str) -> Self {
        self.icon_name = Some(name);
        self
    }
}
