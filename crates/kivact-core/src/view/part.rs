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

use kivact_reactive::{IntoText, IntoValue};
use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::EffectKind;
use kivact_reactive::HandlerResult;

use crate::view::common::Common;
use crate::view::input::{KeyEvent, KeyHandlerResult, PointerEvent, WheelEvent};
use crate::view::owner::Owner;
use crate::view::part_data::{PartBinding, PartMount};
use crate::view::{
    A11yAction, A11yEvent, ButtonPress, Key, LiveRegion, MouseButton, NodeRef, Orientation,
    PointerPhase, Role, SortDirection, node_ref,
};
use crate::{Signal, Value, memo, on_cleanup, signal};

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
pub struct Part<S: Clone + PartialEq + 'static> {
    pub(crate) common: Common,
    state: Value<S>,
    node: NodeRef,
    binding: PartBinding,
}

#[derive(Clone, Copy)]
struct InputState {
    pointer_pressed: Signal<bool>,
    key_pressed: Signal<bool>,
    hovered: Signal<bool>,
    disabled: Signal<bool>,
    hidden: Signal<bool>,
}

impl InputState {
    fn enabled(self) -> bool {
        !self.disabled.get_untracked() && !self.hidden.get_untracked()
    }

    fn clear_pressed(self) {
        self.pointer_pressed.set(false);
        self.key_pressed.set(false);
    }

    fn clear(self) {
        self.clear_pressed();
        self.hovered.set(false);
    }
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
    let hovered = node.hovered();
    let focus_visible = node.focus_visible();
    let input = InputState {
        pointer_pressed: signal(false),
        key_pressed: signal(false),
        hovered: signal(false),
        disabled: signal(false),
        hidden: signal(false),
    };
    let state = memo(move || {
        let disabled = input.disabled.get();
        let hidden = input.hidden.get();
        let enabled = !disabled && !hidden;
        state(PartState {
            pressed: enabled && (input.pointer_pressed.get() || input.key_pressed.get()),
            hovered: enabled && hovered.get(),
            focused: focused.get(),
            focus_visible: focus_visible.get(),
            disabled,
            hidden,
        })
    });
    let mounted = Rc::new(move |mount: PartMount| {
        input.disabled.set(mount.disabled.get_untracked());
        input.hidden.set(mount.hidden.get_untracked());
        let previously_focused = Cell::new(focused.get_untracked());
        let at = Location::caller();
        let owner = kivact_reactive::Owner::current("part binding", at);
        owner.effect(
            EffectKind::Internal,
            move || {
                let disabled = mount.disabled.get();
                let hidden = mount.hidden.get();
                input.disabled.set(disabled);
                input.hidden.set(hidden);
                let now = focused.get();
                if disabled || hidden {
                    input.clear();
                } else if previously_focused.replace(now) && !now {
                    input.clear_pressed();
                }
            },
            at,
        );
        on_cleanup(move || input.clear());
    });
    let mut common = Common::default();
    common.handlers.key.observer(move |event| {
        if matches!(event.key, Key::Space | Key::Enter)
            && focused.get_untracked()
            && input.enabled()
        {
            input.key_pressed.set(true);
        }
    });
    common.handlers.key_up.observer(move |event| {
        if matches!(event.key, Key::Space | Key::Enter) {
            input.key_pressed.set(false);
        }
    });
    common
        .handlers
        .pointer
        .observer(move |event| match event.phase {
            PointerPhase::Down(ButtonPress {
                button: MouseButton::Primary,
                ..
            }) if input.enabled() => {
                input.pointer_pressed.set(true);
            }
            PointerPhase::Up(ButtonPress {
                button: MouseButton::Primary,
                ..
            }) => {
                input.pointer_pressed.set(false);
            }
            PointerPhase::Cancel => input.clear(),
            PointerPhase::Enter if input.enabled() => input.hovered.set(true),
            PointerPhase::Leave => input.hovered.set(false),
            _ => {}
        });
    let binding = PartBinding {
        node,
        hovered: input.hovered.into_value(),
        hooks: vec![Rc::new(move |mount| {
            mounted(mount);
            Ok(())
        })],
    };
    Part {
        common,
        state,
        node,
        binding,
    }
}

impl<S: Clone + PartialEq + 'static> Part<S> {
    /// Combine another part while retaining this part's state type and reference.
    ///
    /// Both parts' observers, references and mount hooks follow the same target.
    /// This part's callbacks run before the added part's callbacks, after caller
    /// customizations. Read the added part's state or reference before consuming it.
    pub fn with<T: Clone + PartialEq + 'static>(mut self, other: Part<T>) -> Self {
        other.attach(&mut self.common);
        self
    }

    /// Also receive unhandled pointer, key and wheel events from this canvas's
    /// declared regions. Caller handlers and the region's defaults run first.
    /// The original region identity and pointer capture are retained. Removed
    /// or disabled captured regions cancel the gesture instead of retargeting it.
    /// Consuming a primary press adopts the gesture, so its pointer release does
    /// not also activate the region. AT and keyboard activation stay independent.
    ///
    /// This opts in only this part's callbacks, not other composed parts. It
    /// does not forward activation or accessibility actions, or pointer events
    /// from separate child nodes. On a region attachment it has no extra effect.
    pub fn region_events(mut self) -> Self {
        self.common.part_region_events = true;
        self
    }

    /// Require an Overlay attachment to confine input and restore focus.
    ///
    /// Multiple requirements combine with OR; a caller cannot erase one with
    /// `Overlay::modal(false)`. A true requirement on a node or canvas region
    /// reports `Unsupported` at mount and is ignored.
    pub fn modal(mut self, modal: bool) -> Self {
        self.common.part_modal |= modal;
        self
    }

    /// Reactive state derived from the attached target's actual input state.
    pub fn state(&self) -> Value<S> {
        self.state
    }

    /// Map exposed state while retaining the same attachment and target reference.
    #[track_caller]
    pub fn map_state<T: Clone + PartialEq + 'static>(
        self,
        f: impl Fn(S) -> T + 'static,
    ) -> Part<T> {
        let Part {
            common,
            state,
            node,
            binding,
        } = self;
        Part {
            common,
            state: memo(move || f(state.get())),
            node,
            binding,
        }
    }

    /// Reference for focus and semantic relationships, bound on attachment.
    pub fn node_ref(&self) -> NodeRef {
        self.node
    }

    /// Bind another relationship reference to the same node or canvas region.
    ///
    /// This adds a reference without replacing the part's own reference.
    pub fn reference(mut self, reference: NodeRef) -> Self {
        self.common.node_ref.push(reference);
        self
    }

    /// Direct the first uncaptured pointer press to another focus target.
    ///
    /// This applies to every pointer button, including presses on disabled
    /// sources and descendants. The destination must be enabled, shown,
    /// focusable and inside the current modal. Keys, accessibility focus,
    /// activation and the field's caret are unaffected. Later attached parts
    /// override this declaration; the destination is followed only once.
    /// An unbound reference permits ancestor lookup, while a bound but
    /// unavailable destination blocks ancestor lookup and falls back to ordinary
    /// native pointer focus.
    #[track_caller]
    pub fn focus_on_press(mut self, destination: NodeRef) -> Self {
        let at = Location::caller();
        let owner = kivact_reactive::Owner::current("Part::focus_on_press", at);
        let runtime = owner.runtime();
        if destination.app != runtime.id() {
            runtime.report(
                Some(owner),
                kivact_reactive::UiError::new(
                    kivact_reactive::ErrorKind::WrongApp,
                    "Part::focus_on_press got a NodeRef of another App; declaration ignored",
                    Some(at),
                ),
            );
            return self;
        }
        // Match existing relation reads for a disposed reference before mutation.
        let _ = destination.target_value(at);
        self.common.focus_on_press = Some(destination);
        self
    }

    /// Register under the mounted target's owner after final constraints resolve.
    ///
    /// Hooks run once per adopted attachment lifetime, in registration order,
    /// after references bind and before the first host commit. A keyed region
    /// keeps its item owner while temporarily shadowed by an earlier duplicate;
    /// re-adoption binds the same part again and starts a new hook lifetime.
    /// Cleanup registered by a hook belongs to that mounted owner, including
    /// when the part was created by an ancestor. `f` may return `()` or
    /// `Result<(), UiError>`; an `Err` goes to that owner's error handler.
    pub fn on_bind<R: HandlerResult>(mut self, f: impl Fn(PartMount) -> R + 'static) -> Self {
        self.binding
            .hooks
            .push(Rc::new(move |mount| f(mount).into_result()));
        self
    }

    #[track_caller]
    pub(crate) fn into_common(mut self) -> Common {
        let at = Location::caller();
        let owner = kivact_reactive::Owner::current("Part attachment", at);
        let runtime = owner.runtime();
        if self.node.app != runtime.id() {
            runtime.report(
                Some(owner),
                kivact_reactive::UiError::new(
                    kivact_reactive::ErrorKind::WrongApp,
                    "Part belongs to another App; attachment ignored",
                    Some(at),
                ),
            );
            return Common::default();
        }
        self.common.node_ref.insert(0, self.node);
        self.common.part_bindings.insert(0, self.binding);
        self.common
    }

    #[track_caller]
    pub(crate) fn attach(self, common: &mut Common) {
        common.attach_part(self.into_common());
    }

    /// Accessibility role, overriding the builder's default role.
    ///
    /// Same as [`NodeExt::role`](crate::primitive::NodeExt::role).
    pub fn role(mut self, r: Role) -> Self {
        self.common.role = Some(r);
        self
    }

    /// Accessible name.
    ///
    /// Same as [`NodeBuilder::name`](crate::control::NodeBuilder::name).
    pub fn name(mut self, v: impl IntoText) -> Self {
        self.common.name = Some(v.into_text());
        self
    }

    /// Accessible description.
    ///
    /// Same as [`NodeBuilder::description`](crate::control::NodeBuilder::description).
    pub fn description(mut self, v: impl IntoText) -> Self {
        self.common.description = Some(v.into_text());
        self
    }

    /// Accessible value text (e.g. of a slider or progress bar).
    ///
    /// Same as [`NodeExt::value_text`](crate::primitive::NodeExt::value_text).
    pub fn value_text(mut self, v: impl IntoText) -> Self {
        self.common.value = Some(v.into_text());
        self
    }

    /// Selected state (semantics and the `selected` style).
    ///
    /// Same as [`NodeExt::selected`](crate::primitive::NodeExt::selected).
    pub fn selected(mut self, v: impl IntoValue<bool>) -> Self {
        self.common.selected = Some(v.into_prop());
        self
    }

    /// Disabled state (semantics and the `disabled` style).
    ///
    /// Same as [`NodeBuilder::disabled`](crate::control::NodeBuilder::disabled).
    pub fn disabled(mut self, v: impl IntoValue<bool>) -> Self {
        self.common.disabled = Some(v.into_prop());
        self
    }

    /// Hide the node and its subtree.
    ///
    /// Same as [`NodeBuilder::hidden`](crate::control::NodeBuilder::hidden).
    pub fn hidden(mut self, v: impl IntoValue<bool>) -> Self {
        self.common.hidden = Some(v.into_prop());
        self
    }

    /// Whether the pointer hits the node and its subtree (default `true`).
    ///
    /// Same as [`NodeBuilder::hit_testable`](crate::control::NodeBuilder::hit_testable).
    pub fn hit_testable(mut self, on: bool) -> Self {
        self.common.hit_testable = Some(on);
        self
    }

    /// Whether the node takes keyboard focus, overriding the builder's default.
    ///
    /// Same as [`NodeBuilder::focusable`](crate::control::NodeBuilder::focusable).
    pub fn focusable(mut self, on: impl IntoValue<bool>) -> Self {
        self.common.focusable = Some(on.into_prop());
        self
    }

    /// Binary checked state for a check box or switch.
    ///
    /// Same as [`NodeExt::checked`](crate::primitive::NodeExt::checked).
    pub fn checked(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.checked = Some(value.into_prop());
        self
    }

    /// Expanded state of a disclosure, menu or combo box.
    ///
    /// Same as [`NodeExt::expanded`](crate::primitive::NodeExt::expanded).
    pub fn expanded(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.expanded = Some(value.into_prop());
        self
    }

    /// Mixed checked state, like an HTML checkbox's `indeterminate`.
    ///
    /// Same as [`NodeExt::indeterminate`](crate::primitive::NodeExt::indeterminate).
    pub fn indeterminate(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.indeterminate = Some(value.into_prop());
        self
    }

    /// Set value_min semantics, constant or reactive.
    ///
    /// Same as [`NodeExt::value_min`](crate::primitive::NodeExt::value_min).
    pub fn value_min(mut self, value: impl IntoValue<f64>) -> Self {
        self.common.semantics_mut().value_min = Some(value.into_prop());
        self
    }

    /// Set value_max semantics, constant or reactive.
    ///
    /// Same as [`NodeExt::value_max`](crate::primitive::NodeExt::value_max).
    pub fn value_max(mut self, value: impl IntoValue<f64>) -> Self {
        self.common.semantics_mut().value_max = Some(value.into_prop());
        self
    }

    /// Set value_now semantics, constant or reactive.
    ///
    /// Same as [`NodeExt::value_now`](crate::primitive::NodeExt::value_now).
    pub fn value_now(mut self, value: impl IntoValue<f64>) -> Self {
        self.common.semantics_mut().value_now = Some(value.into_prop());
        self
    }

    /// Set live_region semantics, constant or reactive.
    ///
    /// Same as [`NodeExt::live_region`](crate::primitive::NodeExt::live_region).
    pub fn live_region(mut self, value: impl IntoValue<LiveRegion>) -> Self {
        self.common.semantics_mut().live_region = Some(value.into_prop());
        self
    }

    /// Set sort semantics, constant or reactive.
    ///
    /// Same as [`NodeExt::sort`](crate::primitive::NodeExt::sort).
    pub fn sort(mut self, value: impl IntoValue<SortDirection>) -> Self {
        self.common.semantics_mut().sort = Some(value.into_prop());
        self
    }

    /// Set orientation semantics, constant or reactive.
    ///
    /// Same as [`NodeExt::orientation`](crate::primitive::NodeExt::orientation).
    pub fn orientation(mut self, value: impl IntoValue<Orientation>) -> Self {
        self.common.semantics_mut().orientation = Some(value.into_prop());
        self
    }

    /// Identify the active option while focus stays on this control, constant or reactive.
    ///
    /// Same as [`NodeExt::active_descendant`](crate::primitive::NodeExt::active_descendant).
    pub fn active_descendant(mut self, descendant: impl IntoValue<Option<NodeRef>>) -> Self {
        self.common.semantics_mut().active_descendant = Some(descendant.into_prop());
        self
    }

    /// Set the depth of this node in a hierarchy (`aria-level`), counted from 1: a heading's level, or a tree item's depth.
    ///
    /// Same as [`NodeExt::level`](crate::primitive::NodeExt::level).
    pub fn level(mut self, value: impl IntoValue<u32>) -> Self {
        self.common.semantics_mut().level = Some(value.into_prop());
        self
    }

    /// Set how many items the set this node belongs to or holds has (`aria-setsize`).
    ///
    /// Same as [`NodeExt::set_size`](crate::primitive::NodeExt::set_size).
    pub fn set_size(mut self, value: impl IntoValue<u32>) -> Self {
        self.common.semantics_mut().set_size = Some(value.into_prop());
        self
    }

    /// Set the position of this node in its set (`aria-posinset`), counted from 1 and not above `.set_size(..)`.
    ///
    /// Same as [`NodeExt::pos_in_set`](crate::primitive::NodeExt::pos_in_set).
    pub fn pos_in_set(mut self, value: impl IntoValue<u32>) -> Self {
        self.common.semantics_mut().pos_in_set = Some(value.into_prop());
        self
    }

    /// State that this list, tree or grid allows more than one selected item (`aria-multiselectable`).
    ///
    /// Same as [`NodeExt::multiselectable`](crate::primitive::NodeExt::multiselectable).
    pub fn multiselectable(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.semantics_mut().multiselectable = Some(value.into_prop());
        self
    }

    /// Whether assistive technology should wait for updates to finish.
    pub fn busy(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.semantics_mut().busy = Some(value.into_prop());
        self
    }

    /// Whether a value is required.
    pub fn required(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.semantics_mut().required = Some(value.into_prop());
        self
    }

    /// Whether assistive technology should report the value as read-only.
    /// This changes semantics only; configure editing behavior on the editor.
    pub fn semantics_read_only(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.semantics_mut().read_only = Some(value.into_prop());
        self
    }

    /// The type of popup this control opens.
    /// Pass `None` to leave it unstated.
    pub fn has_popup(mut self, value: impl IntoValue<Option<crate::view::PopupKind>>) -> Self {
        self.common.semantics_mut().has_popup = Some(value.into_prop());
        self
    }

    /// The meaning of the current item in a related set.
    /// Pass `None` to leave it unstated.
    pub fn current_item(mut self, value: impl IntoValue<Option<crate::view::CurrentItem>>) -> Self {
        self.common.semantics_mut().current_item = Some(value.into_prop());
        self
    }

    /// Name a node that describes this one (`aria-describedby`), for text such as a dialog's message or a field's hint.
    ///
    /// Same as [`NodeExt::described_by`](crate::primitive::NodeExt::described_by).
    pub fn described_by(mut self, target: NodeRef) -> Self {
        self.common.semantics_mut().described_by.push(target);
        self
    }

    /// Name a node this one controls (`aria-controls`), such as the panel a button expands.
    ///
    /// Same as [`NodeExt::controls`](crate::primitive::NodeExt::controls).
    pub fn controls(mut self, target: NodeRef) -> Self {
        self.common.semantics_mut().controls.push(target);
        self
    }

    /// Name a node that labels this one (`aria-labelledby`), for a visible heading or caption that should be the accessible name.
    ///
    /// Same as [`NodeExt::labelled_by`](crate::primitive::NodeExt::labelled_by).
    pub fn labelled_by(mut self, target: NodeRef) -> Self {
        self.common.semantics_mut().labelled_by.push(target);
        self
    }

    /// Report an invalid value to assistive technology.
    ///
    /// Same as [`NodeBuilder::invalid`](crate::control::NodeBuilder::invalid).
    pub fn invalid(mut self, value: impl IntoValue<bool>) -> Self {
        self.common.invalid = Some(value.into_prop());
        self
    }

    /// Handle keys reported by the host before existing key handlers, default activation and Escape dismissal.
    ///
    /// Same as [`NodeBuilder::on_key`](crate::control::NodeBuilder::on_key).
    pub fn on_key<F, R>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.common.prepend_key(f);
        self
    }

    /// Key release handler; releases do not edit or activate controls.
    ///
    /// Same as [`NodeBuilder::on_key_up`](crate::control::NodeBuilder::on_key_up).
    pub fn on_key_up<F, R>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.common.prepend_key_up(f);
        self
    }

    /// Run before existing pointer behavior, in node-local coordinates.
    ///
    /// Same as [`NodeBuilder::on_pointer`](crate::control::NodeBuilder::on_pointer).
    pub fn on_pointer<F, R>(mut self, f: F) -> Self
    where
        F: Fn(PointerEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.common.prepend_pointer(f);
        self
    }

    /// Run before existing wheel behavior and native scrolling.
    ///
    /// Same as [`NodeBuilder::on_wheel`](crate::control::NodeBuilder::on_wheel).
    pub fn on_wheel<F, R>(mut self, f: F) -> Self
    where
        F: Fn(WheelEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.common.prepend_wheel(f);
        self
    }

    /// Set the shared pointer, Enter/Space and accessibility Activate action.
    ///
    /// Same as [`NodeBuilder::on_press`](crate::control::NodeBuilder::on_press).
    pub fn on_press<F, R>(mut self, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        self.common.set_press(f);
        self
    }

    /// Offer an assistive-technology action.
    ///
    /// Same as [`NodeExt::on_a11y`](crate::primitive::NodeExt::on_a11y).
    #[track_caller]
    pub fn on_a11y<F, R>(mut self, action: A11yAction, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        self.common.offer_a11y(action, f);
        self
    }

    /// Offer an action and receive its [`A11yEvent`]: the region the request names and the request.
    ///
    /// Same as [`NodeExt::on_a11y_event`](crate::primitive::NodeExt::on_a11y_event).
    pub fn on_a11y_event<F, R>(mut self, action: A11yAction, f: F) -> Self
    where
        F: Fn(A11yEvent) -> R + 'static,
        R: HandlerResult,
    {
        self.common.offer_a11y_event(action, f);
        self
    }
}
