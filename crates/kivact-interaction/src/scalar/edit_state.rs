//! Role: Edit session, drag and focus count shared by the thumbs of one scalar.

use super::touch::{Touch, TouchSliderGesture};
use super::track::{self, Grip, Pointer};
use crate::drag::DragApi;
use crate::edit::EditSession;
use kivact_core::custom::Owner;
use kivact_core::geometry::Rect;
use kivact_core::state::{Signal, Value, Var, effect, effect_on, memo, on_cleanup, signal, var};
use kivact_core::view::{Handled, NodeRef, node_ref};

/// The edit plumbing one [`ScalarApi`] or both thumbs of a
/// [`RangeScalarApi`](crate::RangeScalarApi) share: the session, what it restores,
/// the track's geometry and the one drag of the thumbs and the track.
#[derive(Clone, Copy)]
pub(crate) struct Shared {
    pub(crate) session: EditSession<[f64; 2]>,
    pub(crate) snapshot: Value<[f64; 2]>,
    pub(crate) pointer: Pointer,
    pub(crate) drag: DragApi<Grip>,
    pub(crate) touch: Var<Option<Touch>>,
    /// The caller's disabled value, given to [`ScalarApi::disabled`].
    pub(crate) disabled: Signal<Option<Value<bool>>>,
    /// The keyboard increment given to `key_step`; the range's step else.
    pub(crate) key_step: Var<Option<(Value<f64>, Owner)>>,
    /// Whether each thumb has the focus; a drag the focus leaves is canceled.
    thumb_focus: Signal<Vec<Value<bool>>>,
}

impl Shared {
    /// Setup only.
    #[track_caller]
    pub(crate) fn new(snapshot: Value<[f64; 2]>, restore: impl Fn([f64; 2]) + 'static) -> Self {
        let session = EditSession::new(restore);
        let pointer = Pointer::new();
        let disabled = signal(None);
        let thumb_focus = signal(Vec::<Value<bool>>::new());
        let shared = Shared {
            session,
            snapshot,
            pointer,
            drag: track::drag(session, snapshot, pointer, disabled),
            disabled,
            touch: var(None),
            key_step: var(None),
            thumb_focus,
        };
        let any_focused = memo(move || thumb_focus.with(|focus| focus.iter().any(|v| v.get())));
        effect_on(any_focused, move |focused| {
            if !*focused {
                shared.drag.cancel();
                shared.finish_touch(true);
            }
        });
        shared
    }

    pub(crate) fn enable_touch(self, config: TouchSliderGesture) {
        if self.touch.get().is_none() {
            let touch = Touch::new(config, self.pointer, self.session, self.snapshot);
            self.touch.set(Some(touch));
            effect(move || {
                if self.disabled.get().is_some_and(|disabled| disabled.get()) {
                    touch.finish(false);
                }
            });
        }
    }

    pub(crate) fn finish_touch(self, restore: bool) -> bool {
        self.touch.get().is_some_and(|touch| touch.finish(restore))
    }

    pub(crate) fn touch_source(
        self,
        part: kivact_core::view::Node,
        grip: Grip,
    ) -> kivact_core::view::Node {
        if let Some(touch) = self.touch.get() {
            part.on_pointer(move |event| {
                if self.disabled.get().is_some_and(|disabled| disabled.get()) {
                    Handled::No
                } else {
                    touch.event(grip, event)
                }
            })
        } else {
            part
        }
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
        self.finish_touch(false);
        let mut accepted = false;
        self.session
            .once(self.snapshot.get_untracked(), || accepted = write());
        accepted
    }
}

/// The edit plumbing of one thumb.
#[derive(Clone, Copy)]
pub(crate) struct Edit {
    pub(crate) observed: Signal<Option<Value<super::data::ScalarState>>>,
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
            observed: signal(None),
            shared,
            index,
            thumb,
            thumb_rect: thumb.rect(),
            bounds,
        }
    }
}

/// Register whether a thumb has the focus (read reactively) with the thumbs
/// `shared` watches; the registration ends with the thumb's owner. Setup only.
#[track_caller]
pub(super) fn register_focus(shared: Shared, focused: Value<bool>) {
    shared.thumb_focus.update(|thumbs| thumbs.push(focused));
    on_cleanup(move || {
        if shared.thumb_focus.try_get_untracked().is_some() {
            shared
                .thumb_focus
                .update(|thumbs| thumbs.retain(|thumb| *thumb != focused));
        }
    });
}
