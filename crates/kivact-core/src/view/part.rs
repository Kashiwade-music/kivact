//! Role: Parts attach behavior and expose the state of their actual target.
//!
//! `primitive::part(|input: PartState| state)` makes a [`Part<S>`]: state,
//! input, focus and semantics for one target, without look, children or
//! drawing. It is moved once into an element's `.with(part)` or, on a
//! canvas, into the setup's `.region(id, part)`. Its `state()` follows the
//! pressed, hovered, focused, focus-visible, disabled and hidden state of
//! the target it is attached to; `map_state` changes the state type only,
//! keeping the target and the lifetime. `node_ref()` and `.reference(r)`
//! return or add references to that same target. Plain elements and
//! external components use the same contract.
//!
//! Invariants:
//! - `first.with(second)` attaches both to one target. The state type and
//!   the main reference stay `first`'s; a state or reference obtained from
//!   `second` before it was consumed follows the same target, and both bind
//!   hooks are kept. The default callbacks run `first`'s then `second`'s,
//!   after the caller's. Constraints and internal transport are flattened,
//!   so `.region(id, first.with(second))` follows the same rules as several
//!   `.with(..)` on a node.
//! - A part's disabled and hidden combine with the caller's by OR, its
//!   focusability by AND; a caller modifier written later does not lift
//!   them. `PartMount::eligible` is the caller's constraints only, so it
//!   does not loop through the part's own roving focus.
//! - `on_bind` runs in the setup of the owner the part was actually
//!   attached under, after the references bind and before the first
//!   commit.
//! - [`Part::modal`] asks the Overlay it is attached to for its modal focus
//!   and input confinement. The requests of several parts and the Overlay's
//!   own `modal` combine by OR (a later `.modal(false)` cannot clear one),
//!   and the focus restore on removal uses the same confinement. On a plain
//!   node or a canvas region it is reported as `Unsupported` at mount and
//!   ignored.
//! - The focus the engine keeps is one `FocusTarget { node, region }`;
//!   regions get no focus manager of their own, and keys, focus changes and
//!   accessibility relations keep that identity. `NodeRef` and `NodeToken`
//!   can name a node or a region. Focus-visible is kept apart from the
//!   focus identity: Tab and assistive technology show it, the pointer
//!   hides it, and a modal's restore uses the saved state. While an input
//!   bubbles, a focus change does not move its delivery target.
//! - [`Part::region_events`] (an approved core addition) delivers this
//!   part's default pointer, key and wheel handling, and only it, to the
//!   unhandled input of the same canvas's regions, in the order caller,
//!   region, opt-in part. It forwards nothing to other parts, activation or
//!   assistive technology. The capture keeps its target's identity, and a
//!   gesture that started is cancelled when its region retires or a
//!   terminal event is consumed. `use_zoom_pan` and the standard menu use
//!   it; kits do not duplicate the delivery.
//!
//! On a canvas, `Draw::region`, `region_shape` and `region_path` supply
//! the geometry only (`kivact_style::draw`); the drawing order decides hits
//! among overlapping regions, and Tab follows the order of the setup
//! declarations.

use kivact_runtime::{IntoText, IntoValue};
use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::EffectKind;
use kivact_runtime::HandlerResult;

use crate::view::common::Common;
use crate::view::input::{KeyEvent, PointerEvent, WheelEvent};
use crate::view::owner::Owner;
use crate::view::part_data::{PartBinding, PartMount};
use crate::view::{
    A11yAction, A11yEvent, ButtonPress, Key, LiveRegion, MouseButton, NodeRef, Orientation,
    PointerPhase, Role, SortDirection, node_ref,
};
use crate::{Value, memo, on_cleanup, signal};

// xtask-allow(state-fields): Public observation snapshot of independent input and availability facts.
/// Actual input and availability state of an attached node or canvas region.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartState {
    /// A primary pointer or focused activation key is held down. Always
    /// `false` while the target is disabled or hidden.
    pub pressed: bool,
    /// The pointer is over this exact target. Always `false` while the
    /// target is disabled or hidden.
    pub hovered: bool,
    /// This exact target has keyboard focus.
    pub focused: bool,
    /// The host requests a visible focus indication for this exact target.
    /// Implies `focused`.
    pub focus_visible: bool,
    /// A caller or attached behavior disables this target.
    pub disabled: bool,
    /// A caller or attached behavior hides this target.
    pub hidden: bool,
}

/// A move-only semantic and input attachment for one node or canvas region.
///
/// Obtain an attachment from [`part`](crate::primitive::part), read its state
/// before consuming it with an element's `with` or a canvas node's `region`.
/// Copies of the returned state and reference continue to follow that target.
/// Parts contain no style, layout, children or rendering implementation.
/// A modal requirement uses the attached Overlay's existing focus scope.
/// Combine attachments with [`with`](Self::with); independently obtained state
/// values and references continue to follow the shared target.
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
/// Read [`Part::state`](crate::primitive::Part::state) before consuming the attachment
/// with an element's `with` modifier or a canvas node's `region` declaration.
/// The closure receives the exact target's input, focus and availability state.
#[track_caller]
pub fn part<S: Clone + PartialEq + 'static>(state: impl Fn(PartState) -> S + 'static) -> Part<S> {
    Owner::current("part");
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
