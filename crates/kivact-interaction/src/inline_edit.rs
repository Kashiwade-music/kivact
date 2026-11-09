//! Role: Own an inline rename request and its mounted draft session.

use crate::{EditSession, TextEditApi, use_text_edit};
use kivact_core::state::{
    IntoBinding, IntoText, IntoValue, ItemKey, Machine, MachineHandle, Step, binding, callback,
    machine, on_cleanup,
};

// A request precedes the conditional field's construction: its draft does not
// exist until the caller mounts field(). It is an actual pending rename request.
enum Phase<K: ItemKey> {
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
impl<K: ItemKey> Clone for Phase<K> {
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
impl<K: ItemKey> PartialEq for Phase<K> {
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
impl<K: ItemKey> Phase<K> {
    fn key(&self) -> Option<K> {
        match self {
            Self::Idle => None,
            Self::Requested { key } | Self::Editing { key, .. } => Some(key.clone()),
        }
    }
}

/// The rename request and the key last written to the caller's binding.
struct InlineEdit<K: ItemKey> {
    phase: Phase<K>,
    // xtask-allow(state-fields): projection: the key last written to the caller's binding, written only by the machine's request transition
    /// The key the machine last reported to the caller's `editing` binding;
    /// a request that differs from it is written once, and its echo is a
    /// request for the state the machine already holds.
    reported: Option<K>,
}
impl<K: ItemKey> Clone for InlineEdit<K> {
    fn clone(&self) -> Self {
        Self {
            phase: self.phase.clone(),
            reported: self.reported.clone(),
        }
    }
}
impl<K: ItemKey> PartialEq for InlineEdit<K> {
    fn eq(&self, other: &Self) -> bool {
        self.phase == other.phase && self.reported == other.reported
    }
}
impl<K: ItemKey> InlineEdit<K> {
    fn key(&self) -> Option<K> {
        self.phase.key()
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
            Event::Request(Some(key)) if state.key().as_ref() == Some(&key) => Step::stay(),
            Event::Request(key) => {
                let write = state.reported != key;
                let mut next = state.clone();
                next.phase = match &key {
                    Some(key) => Phase::Requested { key: key.clone() },
                    None => Phase::Idle,
                };
                if write {
                    next.reported = key.clone();
                }
                let step = Step::to(next);
                if write { step.with(key) } else { step }
            }
            Event::Mounted {
                key,
                draft,
                session,
            } if state.key().as_ref() == Some(&key) => {
                let mut next = state.clone();
                next.phase = Phase::Editing {
                    key,
                    draft,
                    session,
                };
                Step::to(next)
            }
            Event::Typed { key, draft } => match &state.phase {
                Phase::Editing {
                    key: active,
                    session,
                    ..
                } if *active == key => {
                    let mut next = state.clone();
                    next.phase = Phase::Editing {
                        key,
                        draft,
                        session: *session,
                    };
                    Step::to(next)
                }
                _ => Step::stay(),
            },
            Event::Ended(key) if state.key().as_ref() == Some(&key) => {
                let mut next = state.clone();
                next.phase = Phase::Idle;
                next.reported = None;
                Step::to(next).with(None)
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
    let rename = machine(
        InlineEdit {
            phase: initial
                .clone()
                .map_or(Phase::Idle, |key| Phase::Requested { key }),
            reported: initial,
        },
        || Rename(std::marker::PhantomData),
        // The machine is the one writer of the request it reports; the echo of
        // its own write comes back as a request for the state it holds.
        move |_, key| editing.set(key),
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
            move || match self.rename.get().phase {
                Phase::Editing { key, draft, .. } if key == read_key => draft,
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
