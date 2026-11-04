//! Role: In-place renaming shares the native text editor's draft lifecycle.

use crate::{TextEditApi, use_text_edit};
use kivact_core::state::{Binding, IntoBinding, IntoText, ItemKey, callback, on_cleanup};

/// Tracks the item being renamed without drawing a field.
pub struct InlineEditApi<K: ItemKey> {
    editing: Binding<Option<K>>,
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
    InlineEditApi {
        editing: editing.into_binding(),
    }
}
impl<K: ItemKey> InlineEditApi<K> {
    /// Begin renaming this key.
    pub fn open(self, key: K) {
        self.editing.set(Some(key));
    }
    /// Read whether this key is being renamed.
    pub fn is_open(self, key: &K) -> bool {
        self.editing.with(|open| open.as_ref() == Some(key))
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
        let close = move || {
            if self
                .editing
                .with_untracked(|open| open.as_ref() == Some(&key))
            {
                self.editing.set(None);
            }
        };
        on_cleanup(close.clone());
        use_text_edit(draft).inline(label, callback(move |()| close()))
    }
}
