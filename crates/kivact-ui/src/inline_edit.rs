//! Role: In-place renames: which item is being renamed, and the field that renames it.
use super::text_input::{TextInput, text_input};
use kivact_core::primitive::{NodeExt, part};
use kivact_core::view::{HandlerResult, IntoText, callback};
use kivact_core::{Binding, Signal, on_cleanup, on_mount};

/// Which item of a list, tab strip or tree is being renamed in place; a
/// `Copy` handle created by [`use_inline_edit`].
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::use_inline_edit;
///
/// fn rename() -> impl IntoView {
///     let title = signal(String::from("Untitled"));
///     let rename = use_inline_edit(signal(None::<()>));
///     column((
///         show(
///             move || !rename.is_open(&()),
///             move || button("Rename").on_press(move || rename.open(())),
///         ),
///         show(
///             move || rename.is_open(&()),
///             move || {
///                 rename.field((), "Title", signal(title.get_untracked()), move |text| {
///                     title.set(text);
///                 })
///             },
///         ),
///     ))
/// }
/// ```
pub struct InlineEdit<K: 'static> {
    editing: Binding<Option<K>>,
}

impl<K: 'static> Clone for InlineEdit<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for InlineEdit<K> {}

/// Track which item is being renamed. `editing` holds the key of that item
/// (`None` while none is); the application may own it, for instance to start
/// a rename from a menu. Setup only.
#[track_caller]
pub fn use_inline_edit<K: Clone + PartialEq + 'static>(
    editing: impl Into<Binding<Option<K>>>,
) -> InlineEdit<K> {
    InlineEdit {
        editing: editing.into(),
    }
}

impl<K: Clone + PartialEq + 'static> InlineEdit<K> {
    /// Rename `key`. Another rename's field is removed, which cancels its
    /// draft.
    pub fn open(self, key: K) {
        self.editing.set(Some(key));
    }

    /// Whether `key` is being renamed. Tracked.
    pub fn is_open(self, key: &K) -> bool {
        self.editing.with(|open| open.as_ref() == Some(key))
    }

    /// The field that renames `key`; build it inside
    /// `show(move || edit.is_open(&key), ..)`.
    ///
    /// It edits `draft` (create the signal there, from the current name),
    /// takes the focus with its text selected when it is mounted, commits
    /// with Enter or by leaving it changed (`on_commit` gets the text),
    /// cancels with Escape, and ends the rename either way. Removing the
    /// field (its row goes away) also ends the rename. An invalid draft
    /// stays open. Setup only.
    #[track_caller]
    pub fn field<R: HandlerResult>(
        self,
        key: K,
        label: impl IntoText,
        draft: Signal<String>,
        on_commit: impl Fn(String) -> R + 'static,
    ) -> TextInput {
        let focus = part(|input| input);
        let target = focus.node_ref();
        on_mount(move || {
            target.focus();
            target.select_all();
        });
        let close = {
            let key = key.clone();
            move || {
                if self
                    .editing
                    .with_untracked(|open| open.as_ref() == Some(&key))
                {
                    self.editing.set(None);
                }
            }
        };
        on_cleanup(close.clone());
        text_input(label, draft)
            .part(focus)
            .draft(on_commit)
            .on_draft_end(callback(move |()| close()))
    }
}
