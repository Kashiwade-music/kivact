//! Role: A text input that edits a draft: Enter commits, Escape and blur end it.
//!
//! The draft is the text of the field while the user edits it, kept apart
//! from the value it will become. It lives in an [`EditSession`], so the
//! phases an app records (`Begin`, `Commit`, `Cancel`) and the rule that a
//! cancel writes the starting text back are the ones every other edit has.
use kivact_core::UiError;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::callback;
use kivact_core::state::{Binding, Callback, effect_on};
use kivact_core::view::{Handled, Key, KeyEvent, Modifiers, NodeRef};
use kivact_interaction::{EditPhase, EditSession};
use std::rc::Rc;

/// What `text_input(..).draft(..)` adds to a field, kept until the field is
/// built.
#[derive(Clone)]
pub(super) struct Draft {
    pub(super) commit: Callback<String>,
    pub(super) on_edit: Option<Callback<EditPhase>>,
    pub(super) return_focus: Option<NodeRef>,
    /// Called once the draft has ended with a commit or a cancel.
    pub(super) ended: Option<Callback<()>>,
}

/// The drafting of one field: its session, and the way out of it.
#[derive(Clone, Copy)]
struct Drafting {
    commit: Callback<String>,
    failed: Callback<UiError>,
    session: EditSession<String>,
    value: Binding<String>,
    return_focus: Option<NodeRef>,
}

impl Drafting {
    /// Start a session from `text`, unless one is running.
    fn begin(self, text: String) {
        if !self.session.is_active() {
            self.session.begin(text);
        }
    }

    /// Run the commit callback with the draft and end the session when it
    /// accepts; a failure is reported and the draft stays open. Returns
    /// whether the draft was committed.
    fn commit(self) -> bool {
        let text = self.value.get_untracked();
        match self.commit.try_call(text) {
            Ok(()) => {
                self.session.commit();
                true
            }
            Err(error) => {
                self.failed.call(error);
                false
            }
        }
    }

    /// Hand the focus on, once the draft has ended with Enter or Escape.
    fn leave(self) {
        if let Some(target) = self.return_focus {
            target.focus();
        }
    }
}

/// The part to attach to the field, and what Enter does.
pub(super) struct Installed {
    pub(super) part: Part<PartState>,
    pub(super) submit: Rc<dyn Fn()>,
    /// Called before the user's edit is written, to begin a draft.
    pub(super) edited: Rc<dyn Fn()>,
}

/// Draft the field bound to `value`, whose validity `invalid` reads. Setup
/// only.
pub(super) fn install(
    draft: Draft,
    value: Binding<String>,
    invalid: Rc<dyn Fn() -> bool>,
) -> Installed {
    let session = EditSession::new(move |text: String| value.set(text));
    let (on_edit, ended) = (draft.on_edit, draft.ended);
    session.on_edit(move |phase| {
        if let Some(on_edit) = on_edit {
            on_edit.call(phase);
        }
        if phase != EditPhase::Begin
            && let Some(ended) = ended
        {
            ended.call(());
        }
    });
    let drafting = Drafting {
        commit: draft.commit,
        failed: callback(Err::<(), UiError>),
        session,
        value,
        return_focus: draft.return_focus,
    };
    let attachment = part(|input| input);
    let focused = attachment.node_ref().focused();
    let blur_invalid = invalid.clone();
    effect_on(focused, move |&now| {
        if now {
            // The focus came in: the draft starts from the text as it is.
            drafting.begin(value.get_untracked());
        } else if session.is_active() {
            // The focus left. An unchanged draft ends without a write, a
            // changed and valid one is committed, and an invalid one stays
            // pending until Escape or until the field is gone.
            let unchanged = session
                .with_initial(|started| *started == value.get_untracked())
                .unwrap_or(false);
            if unchanged {
                session.cancel();
            } else if !blur_invalid() {
                drafting.commit();
            }
        }
    });
    let attachment = attachment.on_key(move |event: KeyEvent| {
        if event.key() == Key::Escape
            && event.modifiers() == Modifiers::default()
            && session.cancel()
        {
            drafting.leave();
            return Handled::Yes;
        }
        Handled::No
    });
    let submit: Rc<dyn Fn()> = Rc::new(move || {
        drafting.begin(value.get_untracked());
        if !invalid() && drafting.commit() {
            drafting.leave();
        }
    });
    // An edit of the user begins a draft in a field that kept the focus
    // after the last one was committed.
    let edited: Rc<dyn Fn()> = Rc::new(move || drafting.begin(value.get_untracked()));
    Installed {
        part: attachment,
        submit,
        edited,
    }
}
