//! Role: Tooltip attaches a delayed hover and focus hint, one shown per App.
//!
//! A tooltip combines delayed visibility and popup part state with hover and focus triggering,
//! a delay and the App-wide rule of one shown tooltip. Its parts are the
//! trigger and the content, with [`PopupState`]; it adds
//! no wrapper box, so the trigger part binds beside the caller's own
//! `NodeRef`. The delay is a [`DelayedApi`](crate::DelayedApi) on owner-scoped machine timers, and
//! the hover reading is the host's hit test, so it works the same with the
//! keyboard focus and is not changed by a child's pointer handler or
//! capture. Because the press and wheel handling runs after the caller's
//! handlers, a press a child handles never reaches it. The text, the name,
//! the look and the placement are the caller's; `kivact::ui::tooltip` adds
//! the hint as the trigger's description only when the trigger has no
//! explicit description, and mounts the hint with `show(tooltip.is_open(), ..)`
//! in a new owner while it is shown.

use std::time::Duration;

use crate::options_cell::OptionsCell;
use kivact_core::custom::{Owner, Part, part};
use kivact_types::cell::Generation;

use crate::delayed::{DelayedApi, delayed};
use crate::popup::PopupState;
use kivact_core::state::IntoValue;
use kivact_core::state::{Signal, Value, memo, on_cleanup, signal};
use kivact_core::view::{
    A11yAction, Handled, Key, KeyEvent, Modifiers, NodeRef, PointerEvent, PointerPhase, Role,
    node_ref,
};

use super::machine::{Event, Machine, State as TooltipHub, TooltipId, Warmth};

/// The owner-scoped App hub and its independent identity allocator.
#[derive(Clone, Copy)]
struct Hub {
    machine: kivact_core::state::MachineHandle<Machine>,
    next: Signal<Generation>,
}
impl Hub {
    #[track_caller]
    fn of_app() -> Self {
        Owner::current();
        kivact_core::state::use_app_state(|| Hub {
            machine: kivact_core::state::machine(
                TooltipHub {
                    stack: Vec::new(),
                    warmth: Warmth::Cold,
                },
                || Machine,
                |_, effect| match effect {},
            ),
            next: signal(Generation::new()),
        })
    }
    fn take_id(&self) -> TooltipId {
        self.next.update(Generation::next)
    }
    fn current(&self) -> Option<TooltipId> {
        self.machine.get().stack.last().copied()
    }
    fn warm_for(&self, id: TooltipId) -> bool {
        let hub = self.machine.state().get_untracked();
        hub.stack.last().is_some_and(|visible| *visible != id)
            || matches!(hub.warmth, Warmth::Warm { excluded } if excluded != id)
    }
    fn want(&self, id: TooltipId) {
        self.machine.send(Event::Want(id));
    }
    fn release(&self, id: TooltipId, within: Duration) {
        self.machine.send(Event::Release { id, within });
    }
}

/// Copy handle for a tooltip: when a hint shows and what it describes.
///
/// Attach [`trigger`](Self::trigger) to the caller's element; no wrapper box is
/// added and the trigger keeps its handlers, its refs and its place in the
/// layout. [`show`](kivact_core::view::show) mounts the caller's hint while it is shown;
/// put the hint in an `overlay(..)` anchored to the trigger part's `node_ref()`
/// and attach [`content`](Self::content) to it. The text of the hint is the
/// caller's content, and so is the overlay's accessible name.
///
/// It shows when the pointer is over the trigger or the keyboard focus is on
/// it or inside it, after the [`crate::TooltipTiming::delay`] (500 ms by default).
/// The pointer is judged by what the host hits ([`NodeRef::hovered`]),
/// so a child with its own `on_pointer` or a pointer captured elsewhere does
/// not change it. While the pointer is over the hint itself it also stays
/// shown; it hides when the pointer leaves both and the trigger has no focus.
///
/// It hides at once on Escape (only while it is shown: Escape is consumed by
/// the tooltip then and goes on as usual otherwise), on a pointer press on the
/// trigger and on a wheel over it. Tab and modifier keys alone do not hide
/// it. After one of these it stays hidden until the condition stops and
/// becomes true again, as [`DelayedApi::dismiss`](crate::DelayedApi::dismiss) does. A child
/// of the trigger that handles pointer presses itself is the one that receives
/// them, so a press on it does not reach the trigger; Escape and a press
/// outside still hide the tooltip.
///
/// One tooltip is shown at a time in an App: showing a tooltip hides the one
/// before it. When a tooltip hid less than [`crate::TooltipTiming::skip_delay`] ago
/// (300 ms by default), or another is shown, the next one shows without the
/// delay. A tooltip that another one took over while it was still engaged
/// shows again when that one hides. This state is created by the first
/// tooltip, under the App's root owner, and is not made in an App without
/// tooltips.
///
/// Semantics: the surface is `Role::Tooltip` and cannot take the focus; the
/// trigger is `described_by` it while it is shown.
///
/// ```no_run
/// use kivact_core::view::{Element, node, node_ref, text};
/// use kivact_core::view::{Placement, Role, overlay, show};
/// use kivact_interaction::use_tooltip;
/// let tip = use_tooltip();
/// let trigger = tip.trigger();
/// let anchor = node_ref();
/// let view = (
///     node().part(trigger).role(Role::Button).name("Save").focusable(true).child(text("Save")),
///     show(tip.is_open(), move || {
///         overlay(anchor, Placement::Below, text("Save the document"))
///             .with_node(|node| node.part(tip.content()))
///             .name("Save the document")
///     }),
/// );
/// ```
#[derive(Clone, Copy)]
pub struct TooltipApi {
    options: OptionsCell<TooltipOptions>,
    // xtask-allow(state-fields): independent: set by TooltipApi::trigger at mount; the hint content is mounted only while the tooltip shows
    trigger_state: Signal<Option<Value<PopupState>>>,
    // xtask-allow(state-fields): independent: set by TooltipApi::content when the hint mounts after the show delay, after the trigger attached
    content_state: Signal<Option<Value<PopupState>>>,
    delayed: DelayedApi,
    visible: Value<bool>,
    anchor: NodeRef,
    surface: NodeRef,
}

#[derive(Clone, Copy, Default)]
struct TooltipOptions {
    disabled: Option<Value<bool>>,
}

/// Create a tooltip under the current setup owner, reading [`crate::TooltipTiming`].
///
/// The first tooltip of an App creates the App's shared one-at-a-time state.
/// Setup only.
#[track_caller]
pub fn use_tooltip() -> TooltipApi {
    let options = OptionsCell::new(TooltipOptions::default());
    let config = kivact_core::style::use_theme();
    let hub = Hub::of_app();
    let id = hub.take_id();

    let anchor = node_ref();
    let surface = node_ref();
    let hover = anchor.hovered();
    let surface_hover = surface.hovered();
    let focus = anchor.focus_within();
    let engaged = memo(move || {
        !options
            .value()
            .get()
            .disabled
            .is_some_and(|disabled| disabled.get())
            && (hover.get() || surface_hover.get() || focus.get())
    });

    // The delay, or none while another tooltip is warm; both end with the
    // engagement. One tooltip at a time: the delayed machine's visibility
    // changes are the only events this tooltip sends to the App hub, so the
    // one shown last is visible, and one that another took over shows again
    // when that one hides.
    let delayed = delayed(
        engaged,
        move || {
            if hub.warm_for(id) {
                Duration::ZERO
            } else {
                config.get().ext::<crate::InteractionPolicy>().tooltip.delay
            }
        },
        Duration::ZERO,
        move |shown| {
            if shown {
                hub.want(id);
            } else {
                hub.release(
                    id,
                    config
                        .get_untracked()
                        .ext::<crate::InteractionPolicy>()
                        .tooltip
                        .skip_delay,
                );
            }
        },
    );
    let visible = memo(move || delayed.is_open().get() && hub.current() == Some(id));
    on_cleanup(move || {
        if hub.next.try_get().is_some() {
            hub.release(
                id,
                config
                    .get()
                    .ext::<crate::InteractionPolicy>()
                    .tooltip
                    .skip_delay,
            );
        }
    });
    TooltipApi {
        trigger_state: signal(None),
        content_state: signal(None),
        options,
        delayed,
        visible,
        anchor,
        surface,
    }
}

impl TooltipApi {
    /// Drawing state of the actual mounted trigger.
    pub fn trigger_state(self) -> Value<PopupState> {
        memo(move || {
            self.trigger_state
                .get()
                .map(|state| state.get())
                .unwrap_or(PopupState {
                    input: kivact_core::custom::PartState::default(),
                    open: self.visible.get(),
                })
        })
    }
    /// Drawing state of the actual mounted hint.
    pub fn content_state(self) -> Value<PopupState> {
        memo(move || {
            self.content_state
                .get()
                .map(|state| state.get())
                .unwrap_or(PopupState {
                    input: kivact_core::custom::PartState::default(),
                    open: self.visible.get(),
                })
        })
    }

    /// Suppress the hint while disabled.
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        self.options
            .update(|options| options.disabled = Some(disabled.into_value()));
        self
    }

    /// Whether the hint is shown now.
    pub fn is_open(self) -> Value<bool> {
        self.visible
    }

    /// Hide the hint until the trigger is engaged again.
    #[track_caller]
    fn dismiss(self) {
        self.delayed.dismiss();
    }

    /// The trigger whose hover and focus show the hint.
    ///
    /// The press, wheel and Escape handling run after the trigger's own
    /// handlers and never consume a press or a wheel. The trigger is
    /// `described_by` the surface while the hint is shown.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        let visible = self.visible;
        let input = part(move |input| PopupState {
            input,
            open: visible.get(),
        });
        let part_state = input.state();
        crate::part_slot::attach_part_state(self.trigger_state, part_state);
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(self.anchor)
                .described_by(self.surface)
                .on_pointer(move |event: PointerEvent| {
                    if matches!(event.phase(), PointerPhase::Down(_)) {
                        self.dismiss();
                    }
                    Handled::No
                })
                .on_wheel(move |_| {
                    self.dismiss();
                    Handled::No
                })
                .on_key(move |event: KeyEvent| {
                    if event.key() == Key::Escape
                        && event.modifiers() == Modifiers::default()
                        && visible.get()
                    {
                        self.dismiss();
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
        })
    }

    /// The hint's surface, for the caller's `overlay(..)`.
    ///
    /// It is `Role::Tooltip`, and an Escape that no handler consumed or
    /// assistive technology's Dismiss hides the hint.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        let visible = self.visible;
        let input = part(move |input| PopupState {
            input,
            open: visible.get(),
        });
        let part_state = input.state();
        crate::part_slot::attach_part_state(self.content_state, part_state);
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(self.surface)
                .role(Role::Tooltip)
                .a11y_action(A11yAction::Dismiss, move |_| self.dismiss())
        })
    }
}
