//! Role: Tooltip attaches a delayed hover and focus hint, one shown per App.
//!
//! A tooltip is a popup (`crate::popup`) with hover and focus triggering,
//! a delay and the App-wide rule of one shown tooltip. Its parts are the
//! trigger and the content, with [`PopupState`]; it adds
//! no wrapper box, so the trigger part binds beside the caller's own
//! `NodeRef`. The delay is a [`DelayedApi`](crate::DelayedApi) on a task timer (no polling), and
//! the hover reading is the host's hit test, so it works the same with the
//! keyboard focus and is not changed by a child's pointer handler or
//! capture. Because the press and wheel handling runs after the caller's
//! handlers, a press a child handles never reaches it. The text, the name,
//! the look and the placement are the caller's; `kivact::ui::tooltip` adds
//! the hint as the trigger's description only when the trigger has no
//! explicit description, and mounts the hint with `show(tooltip.is_open(), ..)`
//! in a new owner while it is shown.

use std::time::Duration;

use kivact_core::custom::{Owner, Part, part};

use super::delayed::{DelayedApi, delayed};
use super::popup::PopupState;
use kivact_core::state::IntoValue;
use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, signal};
use kivact_core::view::{
    A11yAction, Handled, Key, KeyEvent, Modifiers, NodeRef, PointerEvent, PointerPhase, Role,
    node_ref,
};

/// Theme-wide tooltip timing, configured through [`crate::InteractionPolicy`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooltipTiming {
    /// How long the pointer or the keyboard focus has to stay before a
    /// tooltip shows. Zero shows it at once.
    pub delay: Duration,
    /// After a tooltip hides, a tooltip of another trigger that becomes
    /// engaged within this time shows without the delay, so moving along a
    /// toolbar does not wait at every button.
    pub skip_delay: Duration,
}

impl Default for TooltipTiming {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(500),
            skip_delay: Duration::from_millis(300),
        }
    }
}

/// What every tooltip of an App shares, created under the App's root owner by
/// the first tooltip and kept for the App's lifetime. An App without tooltips
/// has none of it.
#[derive(Clone, Copy)]
struct Hub {
    /// The tooltips that are shown by their own delay, oldest first; the last
    /// one is visible.
    wanting: Signal<Vec<u64>>,
    /// The visible tooltip that hid last and when, on the host clock.
    hidden: Signal<Option<(u64, u64)>>,
    /// The id the next tooltip takes.
    next: Signal<u64>,
}

impl Hub {
    /// The App's hub, made on first use. Setup only.
    #[track_caller]
    fn of_app() -> Self {
        Owner::current();
        kivact_core::state::use_app_state(|| Hub {
            wanting: signal(Vec::<u64>::new()),
            hidden: signal(None::<(u64, u64)>),
            next: signal(0u64),
        })
    }

    fn take_id(&self) -> u64 {
        let id = self.next.get_untracked();
        self.next.set(id + 1);
        id
    }

    /// The visible tooltip (tracked).
    fn current(&self) -> Option<u64> {
        self.wanting.with(|wanting| wanting.last().copied())
    }

    /// Whether a tooltip other than `id` is visible or hid less than `within`
    /// ago, so `id` shows without a delay (untracked).
    fn warm_for(&self, id: u64, within: Duration) -> bool {
        let visible = self
            .wanting
            .with_untracked(|wanting| wanting.last().copied());
        if visible.is_some_and(|visible| visible != id) {
            return true;
        }
        self.hidden.get_untracked().is_some_and(|(hid, at)| {
            hid != id && now_ns().saturating_sub(at) <= within.as_nanos() as u64
        })
    }

    /// `id` is shown by its own delay: it becomes the visible tooltip.
    fn want(&self, id: u64) {
        self.wanting.update(|wanting| {
            wanting.retain(|other| *other != id);
            wanting.push(id);
        });
    }

    /// `id` no longer wants to show: if it was the visible one, it hides now
    /// and the one it preempted is visible again.
    fn release(&self, id: u64) {
        if !self.wanting.with_untracked(|wanting| wanting.contains(&id)) {
            return;
        }
        let was_visible = self.wanting.update(|wanting| {
            let was_visible = wanting.last() == Some(&id);
            wanting.retain(|other| *other != id);
            was_visible
        });
        if was_visible {
            self.hidden.set(Some((id, now_ns())));
        }
    }
}

fn now_ns() -> u64 {
    kivact_core::custom::clock_now()
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
/// it or inside it, after the [`TooltipTiming::delay`] (500 ms by default).
/// The pointer is judged by what the host hits ([`NodeRef::hovered`]),
/// so a child with its own `on_pointer` or a pointer captured elsewhere does
/// not change it. When the condition stops it hides at once.
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
/// before it. When a tooltip hid less than [`TooltipTiming::skip_delay`] ago
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
/// use kivact_core::view::{NodeBuilder, node, text};
/// use kivact_core::view::NodeBuilder;
/// use kivact_core::view::{Placement, Role, overlay, show};
/// use kivact_interaction::use_tooltip;
/// let tip = use_tooltip();
/// let trigger = tip.trigger();
/// let anchor = node_ref();
/// let view = (
///     node().part(trigger).role(Role::Button).name("Save").focusable(true).child(text("Save")),
///     show(tip.is_open(), move || {
///         overlay(anchor, Placement::Below, text("Save the document"))
///             .part(tip.content())
///             .name("Save the document")
///     }),
/// );
/// ```
#[derive(Clone, Copy)]
pub struct TooltipApi {
    disabled: Signal<Option<Value<bool>>>,
    trigger_state: Signal<Option<Value<PopupState>>>,
    content_state: Signal<Option<Value<PopupState>>>,
    delayed: DelayedApi,
    visible: Value<bool>,
    anchor: NodeRef,
    surface: NodeRef,
}

/// Create a tooltip under the current setup owner, reading [`TooltipTiming`].
///
/// The first tooltip of an App creates the App's shared one-at-a-time state.
/// Setup only.
#[track_caller]
pub fn use_tooltip() -> TooltipApi {
    let disabled = signal(None::<Value<bool>>);
    let config = kivact_core::style::use_theme();
    let hub = Hub::of_app();
    let id = hub.take_id();

    let anchor = node_ref();
    let hover = anchor.hovered();
    let focus = anchor.focus_within();
    let engaged = memo(move || {
        !disabled.with(|disabled| disabled.is_some_and(|disabled| disabled.get()))
            && (hover.get() || focus.get())
    });

    // The delay, or none while another tooltip is warm; both end with the
    // engagement.
    let delayed = delayed(
        engaged,
        move || {
            if hub.warm_for(
                id,
                config
                    .get()
                    .ext::<crate::InteractionPolicy>()
                    .tooltip
                    .skip_delay,
            ) {
                Duration::ZERO
            } else {
                config.get().ext::<crate::InteractionPolicy>().tooltip.delay
            }
        },
        Duration::ZERO,
    );

    // One tooltip at a time: the one that was shown last is visible, and one
    // that another took over shows again when that one hides.
    effect(move || {
        if delayed.is_open().get() {
            hub.want(id);
        } else {
            hub.release(id);
        }
    });
    let visible = memo(move || delayed.is_open().get() && hub.current() == Some(id));
    on_cleanup(move || {
        if hub
            .wanting
            .try_get()
            .is_some_and(|wanting| wanting.contains(&id))
        {
            hub.release(id);
        }
    });
    TooltipApi {
        trigger_state: signal(None),
        content_state: signal(None),
        disabled,
        delayed,
        visible,
        anchor,
        surface: node_ref(),
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
        self.disabled.set(Some(disabled.into_value()));
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
        self.trigger_state.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |mut host| {
            let caller_disabled = host.accessible_disabled();
            let own_disabled = self.disabled.get_untracked();
            self.disabled.set(Some(memo(move || {
                caller_disabled.is_some_and(|value| value.get())
                    || own_disabled.is_some_and(|value| value.get())
            })));
            host.part(input)
                .node_ref(target)
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
        self.content_state.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .node_ref(self.surface)
                .role(Role::Tooltip)
                .a11y_action(A11yAction::Dismiss, move |_| self.dismiss())
        })
    }
}
