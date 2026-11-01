//! Role: Edit session, drag and focus count shared by the thumbs of one scalar.

use super::track::{self, Pointer};
use crate::drag::Drag;
use crate::edit::EditSession;
use kivact_core::draw::Rect;
use kivact_core::view::{NodeRef, node_ref};
use kivact_core::{Signal, Value, effect, effect_on, signal, var};

/// The edit plumbing one [`Scalar`] or both thumbs of a
/// [`RangeScalar`](crate::RangeScalar) share: the session, what it restores,
/// the track's geometry and the one drag of the thumbs and the track.
#[derive(Clone, Copy)]
pub(crate) struct Shared {
    pub(crate) session: EditSession<[f64; 2]>,
    pub(crate) snapshot: Value<[f64; 2]>,
    pub(crate) pointer: Pointer,
    pub(crate) drag: Drag<Option<usize>>,
    /// The caller's disabled value, given to [`Scalar::disabled`].
    pub(crate) disabled: Signal<Option<Value<bool>>>,
    /// How many thumbs have the focus; a drag the focus leaves is canceled.
    focused: Signal<usize>,
}

impl Shared {
    /// Setup only.
    #[track_caller]
    pub(crate) fn new(snapshot: Value<[f64; 2]>, restore: impl Fn([f64; 2]) + 'static) -> Self {
        let session = EditSession::new(restore);
        let pointer = Pointer::new();
        let disabled = signal(None);
        let shared = Shared {
            session,
            snapshot,
            pointer,
            drag: track::drag(session, snapshot, pointer, disabled),
            disabled,
            focused: signal(0),
        };
        effect_on(shared.focused, move |focused| {
            if *focused == 0 {
                shared.drag.cancel();
            }
        });
        shared
    }

    /// Whether the arrows of a thumb mirror: a horizontal track in a
    /// right-to-left subtree.
    pub(super) fn mirrored(self) -> bool {
        self.pointer
            .geometry
            .get()
            .is_some_and(|geometry| geometry.rtl)
    }

    /// Run one whole edit around `write` (or join the active one).
    pub(crate) fn once(self, write: impl FnOnce() -> bool) -> bool {
        let mut accepted = false;
        self.session
            .once(self.snapshot.get_untracked(), || accepted = write());
        accepted
    }
}

/// The edit plumbing of one thumb.
#[derive(Clone, Copy)]
pub(crate) struct Edit {
    pub(crate) shared: Shared,
    /// Which thumb of the slider this is, from 0.
    pub(crate) index: usize,
    /// The thumb's element, which a press on the track focuses.
    pub(crate) thumb: NodeRef,
    pub(crate) thumb_rect: Value<Option<Rect>>,
    /// The published bounds of a thumb that another thumb limits.
    pub(crate) bounds: Option<(Value<f64>, Value<f64>)>,
}

impl Edit {
    /// Setup only.
    #[track_caller]
    pub(crate) fn new(
        shared: Shared,
        index: usize,
        bounds: Option<(Value<f64>, Value<f64>)>,
    ) -> Self {
        let thumb = node_ref();
        Edit {
            shared,
            index,
            thumb,
            thumb_rect: thumb.rect(),
            bounds,
        }
    }
}

/// Count `focused` (read reactively) in the thumbs `shared` watches. Setup only.
#[track_caller]
pub(super) fn watch_focus(shared: Shared, focused: impl Fn() -> bool + 'static) {
    let was = var(false);
    effect(move || {
        let now = focused();
        if now != was.get() {
            was.set(now);
            shared.focused.update(|count| {
                *count = if now {
                    *count + 1
                } else {
                    count.saturating_sub(1)
                }
            });
        }
    });
}
