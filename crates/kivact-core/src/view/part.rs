//! Role: Parts attach behavior and expose the state of their actual target.
//!
//! `Part::new(state, apply)` records one node transformation. `Node::part`
//! defers it until mounting, in order with the surrounding caller modifiers.
//! The state remains readable before the attachment is consumed. Canvas regions
//! apply their attachment once while creating their retained template; adoption
//! only copies the resulting semantic and input transport.
//!
//! `part` derives state from the target's actual pointer, keyboard, focus and
//! availability facts. `Node::on_attach` observes the bound reference and final
//! constraints under the mounted target's setup owner.

use std::panic::Location;
use std::rc::Rc;

use crate::state::{Value, memo, on_cleanup, signal};
use crate::view::node::Common;
use crate::view::owner::Owner;
use crate::view::part_data::{PartBinding, PartMount};
use crate::view::{ButtonPress, Key, MouseButton, PointerPhase, node_ref};
use kivact_runtime::EffectKind;

pub use super::part_data::PartState;

/// A move-only node transformation with an exposed reactive state.
/// Read the state before consuming this attachment with `Node::part` or `Node::region`.
#[must_use]
pub struct Part<S: kivact_runtime::PropValue> {
    state: Value<S>,
    apply: Box<dyn FnOnce(crate::view::node::Node) -> crate::view::node::Node>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct PartInput {
    press: Press,
    hover: Hover,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Press {
    #[default]
    Idle,
    Pointer(crate::view::PointerId),
    Key(Key),
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Hover {
    #[default]
    Outside,
    Inside,
}

/// Create a setup-owned semantic and input attachment for one target.
///
/// Read [`Part::state`](crate::custom::Part::state) before consuming the attachment
/// with an element's `with` modifier or a canvas node's `region` declaration.
/// The closure receives the exact target's input, focus and availability state.
#[track_caller]
pub fn part<S: kivact_runtime::PropValue>(state: impl Fn(PartState) -> S + 'static) -> Part<S> {
    Owner::current();
    let node = node_ref();
    let focused = node.focused();
    let focus_visible = node.focus_visible();
    let input = signal(PartInput::default());
    let availability = signal(None::<(Value<bool>, Value<bool>)>);
    let enabled = move || {
        availability
            .get_untracked()
            .is_none_or(|(disabled, hidden)| !disabled.get_untracked() && !hidden.get_untracked())
    };
    let state = memo(move || {
        let (disabled, hidden) = availability
            .get()
            .map(|(d, h)| (d.get(), h.get()))
            .unwrap_or_default();
        let input = input.get();
        state(PartState {
            pressed: !disabled && !hidden && input.press != Press::Idle,
            hovered: !disabled && !hidden && input.hover == Hover::Inside,
            focused: focused.get(),
            focus_visible: focus_visible.get(),
            disabled,
            hidden,
        })
    });
    let mounted = Rc::new(move |mount: PartMount| {
        availability.set(Some((mount.disabled, mount.hidden)));
        let at = Location::caller();
        let owner = kivact_runtime::Owner::current("part binding", at);
        let hover = node.hovered();
        owner.effect(
            EffectKind::Internal,
            move || {
                let disabled = mount.disabled.get();
                let hidden = mount.hidden.get();
                let hovered = hover.get();
                let focused = focused.get();
                input.update(|input| {
                    if disabled || hidden {
                        *input = PartInput::default();
                    } else {
                        input.hover = if hovered {
                            Hover::Inside
                        } else {
                            Hover::Outside
                        };
                        if !focused && matches!(input.press, Press::Key(_)) {
                            input.press = Press::Idle;
                        }
                    }
                });
            },
            at,
        );
        on_cleanup(move || {
            input.set(PartInput::default());
            availability.set(None);
        });
    });
    let mut common = Common::default();
    common.handlers.key.observer(move |event| {
        if matches!(event.key(), Key::Space | Key::Enter) && focused.get_untracked() && enabled() {
            input.update(|input| input.press = Press::Key(event.key()));
        }
    });
    common.handlers.key_up.observer(move |event| {
        input.update(|input| {
            if input.press == Press::Key(event.key()) {
                input.press = Press::Idle;
            }
        });
    });
    common
        .handlers
        .pointer
        .observer(move |event| match event.phase() {
            PointerPhase::Down(ButtonPress {
                button: MouseButton::Primary,
                ..
            }) if enabled() => {
                input.update(|input| input.press = Press::Pointer(event.id()));
            }
            PointerPhase::Up(ButtonPress {
                button: MouseButton::Primary,
                ..
            }) => {
                input.update(|input| {
                    if input.press == Press::Pointer(event.id()) {
                        input.press = Press::Idle;
                    }
                });
            }
            PointerPhase::Cancel => input.update(|input| {
                if input.press == Press::Pointer(event.id()) {
                    input.press = Press::Idle;
                }
            }),
            _ => {}
        });
    let binding = PartBinding {
        node,
        hooks: vec![Rc::new(move |mount| {
            mounted(mount);
            Ok(())
        })],
    };
    Part::new(state, move |mut target| {
        common.node_ref.insert(0, node);
        common.part_bindings.insert(0, binding);
        target.common.attach_part(common);
        target
    })
}

impl<S: kivact_runtime::PropValue> Part<S> {
    /// Describe behavior to apply when this attachment is mounted.
    pub fn new(
        state: Value<S>,
        apply: impl FnOnce(crate::view::node::Node) -> crate::view::node::Node + 'static,
    ) -> Self {
        Self {
            state,
            apply: Box::new(apply),
        }
    }
    /// The state exposed by this attachment.
    pub fn state(&self) -> Value<S> {
        self.state
    }
    /// Transform the exposed state while retaining the attachment.
    pub fn map_state<T: kivact_runtime::PropValue>(self, f: impl Fn(S) -> T + 'static) -> Part<T> {
        Part {
            state: memo(move || f(self.state.get())),
            apply: self.apply,
        }
    }
    pub(crate) fn into_apply(
        self,
    ) -> Box<dyn FnOnce(crate::view::node::Node) -> crate::view::node::Node> {
        self.apply
    }
    pub(crate) fn into_common(self) -> Common {
        let mut common = Common::default();
        common.deferred_parts.push(self.apply);
        common
    }
}
