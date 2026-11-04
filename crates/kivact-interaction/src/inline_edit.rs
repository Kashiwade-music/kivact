//! Role: Own an inline rename request and its mounted draft session.

use crate::{EditSession, TextEditApi, use_text_edit};
use kivact_core::state::{
    IntoBinding, IntoText, IntoValue, ItemKey, Machine, MachineHandle, Step, binding, callback,
    machine, on_cleanup, var,
};
use kivact_types::cell::Sent;

// A request precedes the conditional field's construction: its draft does not
// exist until the caller mounts field(). It is an actual pending rename request.
enum InlineEdit<K: ItemKey> {
    Idle,
    Requested {
        key: K,
    },
    Editing {
        key: K,
        draft: String,
        session: EditSession<String>,
    },
}
impl<K: ItemKey> Clone for InlineEdit<K> {
    fn clone(&self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Requested { key } => Self::Requested { key: key.clone() },
            Self::Editing {
                key,
                draft,
                session,
            } => Self::Editing {
                key: key.clone(),
                draft: draft.clone(),
                session: *session,
            },
        }
    }
}
impl<K: ItemKey> PartialEq for InlineEdit<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Idle, Self::Idle) => true,
            (Self::Requested { key: a }, Self::Requested { key: b }) => a == b,
            (
                Self::Editing {
                    key: a, draft: x, ..
                },
                Self::Editing {
                    key: b, draft: y, ..
                },
            ) => a == b && x == y,
            _ => false,
        }
    }
}
impl<K: ItemKey> InlineEdit<K> {
    fn key(&self) -> Option<K> {
        match self {
            Self::Idle => None,
            Self::Requested { key } | Self::Editing { key, .. } => Some(key.clone()),
        }
    }
}
enum Event<K: ItemKey> {
    Request(Option<K>),
    Mounted {
        key: K,
        draft: String,
        session: EditSession<String>,
    },
    Typed {
        key: K,
        draft: String,
    },
    Ended(K),
}
struct Rename<K: ItemKey>(std::marker::PhantomData<K>);
impl<K: ItemKey> Machine for Rename<K> {
    type State = InlineEdit<K>;
    type Event = Event<K>;
    type Effect = Option<K>;
    fn transition(
        &self,
        state: &InlineEdit<K>,
        event: Event<K>,
    ) -> Step<InlineEdit<K>, Event<K>, Option<K>> {
        match event {
            Event::Request(key) if state.key() == key => Step::stay(),
            Event::Request(Some(key)) => {
                Step::to(InlineEdit::Requested { key: key.clone() }).with(Some(key))
            }
            Event::Request(None) => Step::to(InlineEdit::Idle).with(None),
            Event::Mounted {
                key,
                draft,
                session,
            } if state.key().as_ref() == Some(&key) => Step::to(InlineEdit::Editing {
                key,
                draft,
                session,
            }),
            Event::Typed { key, draft } => match state {
                InlineEdit::Editing {
                    key: active,
                    session,
                    ..
                } if *active == key => Step::to(InlineEdit::Editing {
                    key,
                    draft,
                    session: *session,
                }),
                _ => Step::stay(),
            },
            Event::Ended(key) if state.key().as_ref() == Some(&key) => {
                Step::to(InlineEdit::Idle).with(None)
            }
            _ => Step::stay(),
        }
    }
}
/// Tracks the item being renamed without drawing a field.
pub struct InlineEditApi<K: ItemKey> {
    rename: MachineHandle<Rename<K>>,
}
impl<K: ItemKey> Clone for InlineEditApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: ItemKey> Copy for InlineEditApi<K> {}

/// Track the caller-owned key of the item being renamed.
#[track_caller]
pub fn use_inline_edit<K: ItemKey>(editing: impl IntoBinding<Option<K>>) -> InlineEditApi<K> {
    let editing = editing.into_binding();
    let initial = editing.get_untracked();
    let sent = var(Sent::new());
    sent.update(|sent| {
        sent.update(initial.clone());
    });
    let rename = machine(
        initial.map_or(InlineEdit::Idle, |key| InlineEdit::Requested { key }),
        || Rename(std::marker::PhantomData),
        move |_, key| {
            if sent.update(|sent| sent.update(key.clone()).is_some()) {
                editing.set(key);
            }
        },
    );
    rename.feed(editing.into_value(), Event::Request);
    InlineEditApi { rename }
}
impl<K: ItemKey> InlineEditApi<K> {
    /// Request renaming this key; the conditional field supplies its draft on mount.
    pub fn open(self, key: K) {
        self.rename.send(Event::Request(Some(key)));
    }
    /// Read whether this key is requested or being renamed.
    pub fn is_open(self, key: &K) -> bool {
        self.rename.get().key().as_ref() == Some(key)
    }
    /// Create a headless editor; attach `.on_draft_commit(..).control()` to the drawn node.
    /// Mounting focuses and selects the text. Commit, cancel and removal end the rename.
    #[track_caller]
    pub fn field(
        self,
        key: K,
        label: impl IntoText,
        draft: impl IntoBinding<String>,
    ) -> TextEditApi {
        let draft = draft.into_binding();
        let restore_key = key.clone();
        let session = EditSession::new(move |text: String| {
            self.rename.send(Event::Typed {
                key: restore_key.clone(),
                draft: text.clone(),
            });
            draft.set(text);
        });
        self.rename.send(Event::Mounted {
            key: key.clone(),
            draft: draft.get_untracked(),
            session,
        });
        let read_key = key.clone();
        let write_key = key.clone();
        let projected = binding(
            move || match self.rename.get() {
                InlineEdit::Editing { key, draft, .. } if key == read_key => draft,
                _ => draft.get(),
            },
            move |text: String| {
                self.rename.send(Event::Typed {
                    key: write_key.clone(),
                    draft: text.clone(),
                });
                draft.set(text);
            },
        );
        let observed_key = key.clone();
        self.rename
            .feed(draft.into_value(), move |text| Event::Typed {
                key: observed_key.clone(),
                draft: text,
            });
        let close = move || self.rename.send(Event::Ended(key.clone()));
        on_cleanup(close.clone());
        use_text_edit(projected)
            .draft_session(session)
            .inline(label, callback(move |()| close()))
    }
}
