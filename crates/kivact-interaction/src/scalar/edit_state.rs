//! Role: Edit session, drag and focus count shared by the thumbs of one scalar.

use super::focus_machine::{Effect, Event, Machine};
use super::touch::{Touch, TouchSliderGesture};
use super::track::{self, Grip, Pointer};
use crate::drag::DragApi;
use crate::edit::EditSession;
use crate::options_cell::OptionsCell;
use kivact_core::custom::Owner;
use kivact_core::geometry::Rect;
use kivact_core::state::{MachineHandle, machine};
use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, signal};
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
    pub(crate) settings: OptionsCell<ScalarOptions>,
    focus: MachineHandle<Machine>,
    /// Whether each thumb has the focus; a drag the focus leaves is canceled.
    thumb_focus: Signal<Vec<Value<bool>>>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ScalarOptions {
    pub(crate) disabled: Option<Value<bool>>,
    pub(crate) key_step: Option<(Value<f64>, Owner)>,
    pub(crate) touch: Option<Touch>,
}

impl Shared {
    /// Setup only.
    #[track_caller]
    pub(crate) fn new(snapshot: Value<[f64; 2]>, restore: impl Fn([f64; 2]) + 'static) -> Self {
        let session = EditSession::new(restore);
        let pointer = Pointer::new();
        let settings = OptionsCell::new(ScalarOptions::default());
        let options = settings.value();
        let disabled = memo(move || options.get().disabled.is_some_and(|d| d.get()));
        let thumb_focus = signal(Vec::<Value<bool>>::new());
        let drag = track::drag(session, snapshot, pointer, disabled);
        let focus = machine(
            (),
            move || Machine {
                any_focused: thumb_focus.with(|focus| focus.iter().any(|v| v.get())),
            },
            move |_, Effect::Cancel| {
                drag.cancel();
                if let Some(touch) = settings.get().touch {
                    touch.finish(true);
                }
            },
        );
        Shared {
            session,
            snapshot,
            pointer,
            drag,
            settings,
            focus,
            thumb_focus,
        }
    }

    pub(crate) fn is_disabled(self) -> bool {
        self.settings
            .value()
            .get()
            .disabled
            .is_some_and(|disabled| disabled.get())
    }

    pub(crate) fn enable_touch(self, config: TouchSliderGesture) {
        if self.settings.get().touch.is_none() {
            let touch = Touch::new(config, self.pointer, self.session, self.snapshot);
            self.settings
                .update(|settings| settings.touch = Some(touch));
            effect(move || {
                if self.is_disabled() {
                    touch.finish(false);
                }
            });
        }
    }

    pub(crate) fn finish_touch(self, restore: bool) -> bool {
        self.settings
            .get()
            .touch
            .is_some_and(|touch| touch.finish(restore))
    }

    pub(crate) fn touch_source(
        self,
        part: kivact_core::view::Node,
        grip: Grip,
    ) -> kivact_core::view::Node {
        if let Some(touch) = self.settings.get().touch {
            part.on_pointer(move |event| {
                if self.is_disabled() {
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
        self.pointer.geometry().is_some_and(|geometry| geometry.rtl)
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
    // xtask-allow(state-fields): independently observed layout, not an edit lifecycle state.
    pub(crate) thumb_rect: Value<Option<Rect>>,
    // xtask-allow(state-fields): optional caller-supplied bounds are independent of the mounted-state observation.
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
    shared.focus.feed(focused, |focused| {
        if focused {
            Event::ThumbFocus
        } else {
            Event::ThumbBlur
        }
    });
    on_cleanup(move || {
        if shared.thumb_focus.try_get_untracked().is_some() {
            let was_focused = focused.get_untracked();
            shared
                .thumb_focus
                .update(|thumbs| thumbs.retain(|thumb| *thumb != focused));
            if was_focused {
                shared.focus.send(Event::ThumbBlur);
            }
        }
    });
}
