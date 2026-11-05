//! Role: PopupApi adds overlay focus, dismissal and restoration to a disclosure.
//!
//! The trigger binds the anchor, opens and closes on activation, takes
//! Expand and Collapse, and closes as `Tab` when Tab is released on it; the
//! content attaches to the caller's `overlay(..)`. Each bind of the surface
//! watches the focus range, the anchor's removal and the initial focus, under
//! the surface mount's owner, so the watchers end when it closes. A dismissal
//! is taken after the caller's own handlers (an error stops it) and becomes a
//! [`PopupCloseReason`]. One popup has one surface at a time: a second
//! simultaneous surface is a usage error.
//!
//! The popup owns the one [`FocusReturn`] policy of its surface: where the
//! focus goes (the anchor, or the node that had it before the surface
//! mounted, captured when the surface binds) and for which close reasons.
//! By default Escape, a selection and assistive technology restore the focus
//! to the anchor; an outside press and Tab keep the destination. A modal's
//! restoration belongs to the overlay's own cleanup. The restore goes through
//! the core's common restore request, so the popup does not check the
//! target's liveness, visibility or disabled state itself: when the caller's
//! writer removes the target in the same turn, the restore waiter is dropped,
//! and an explicit live focus request in the same flush wins. A dropped
//! restore is not reported as a failed explicit focus request.
//!
//! The policy is requested once per opening. Escape, a selection and
//! assistive technology request it when the close is accepted; any other
//! reason (a programmatic close, an unmount) requests it when the surface is
//! removed, and only if the focus was inside the surface or went with it.
//!
//! Once the focus has entered the surface, the focus range is the surface
//! together with the exact anchor: going back to the anchor does not close,
//! and a concrete focus target outside both closes as a Tab departure. While
//! the focus is briefly `None` (a tab switch, say) it stays open and waits
//! for the next tracked focus.

use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Owner, Part, PartState, focused_node, part, raise};
use kivact_core::state::{
    Binding, IntoValue, MachineHandle, Value, effect, machine, memo, on_cleanup, on_mount,
};
use kivact_core::view::{
    A11yAction, A11yRequest, DismissReason, Handled, Key, KeyEvent, NodeRef, node_ref,
};

use super::machine::{self, Event, Machine, State};
use crate::options_cell::OptionsCell;

use crate::disclosure::{DisclosureApi, use_disclosure};
use crate::focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason};

/// The open state and actual input state of a popup's trigger or surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopupState {
    pub input: PartState,
    /// The application's open state.
    pub open: bool,
}

/// Copy handle for a popup: a disclosure whose content floats in an overlay.
///
/// Attach [`trigger`](Self::trigger) to the caller's button and
/// [`content`](Self::content) to the caller's own `overlay(..)`, anchored to the
/// trigger part's `node_ref()`. [`show`](kivact_core::view::show) mounts the caller's content
/// in a fresh owner on every opening; the popup itself renders nothing.
///
/// One popup supports one mounted surface at a time; a second simultaneous
/// surface is a usage error. Copies share the same open binding, anchor and
/// surface reference; create separate popups for simultaneous surfaces.
/// Reopening rebinds the surface reference after the previous branch is
/// disposed. [`anchor`](Self::anchor), [`initial_focus`](Self::initial_focus)
/// and [`focus_return`](Self::focus_return) may be called before or after the
/// parts are made.
///
/// Once focus enters the surface, moving to a concrete target outside both the
/// surface and the exact anchor closes it as [`PopupCloseReason::Tab`] without
/// restoring focus. An absent focus target during a handoff keeps it open;
/// returning to the anchor also keeps it open. Nested overlays count as part of their
/// parent's surface. By default nonmodal Escape, selection and accessibility
/// dismissal restore a mounted anchor; outside presses and focus departure
/// preserve their destination. [`focus_return`](Self::focus_return) chooses
/// another target and other reasons. Modal restoration belongs exclusively to
/// the Overlay's own cleanup. Restoration is best effort: a target removed
/// before the next flush is skipped, and an explicit live focus request wins.
///
/// ```no_run
/// use kivact_core::{state::signal, view::{NodeBuilder, node, node_ref, text, Placement, Role, overlay, show}};
/// use kivact_interaction::use_popup;
/// let open = signal(false);
/// let menu = use_popup(open);
/// let trigger = menu.trigger();
/// let anchor = node_ref();
/// let view = (
///     node().node_ref(anchor).part(trigger).name("Options").child(text("Options")),
///     show(menu.is_open(), move || {
///         overlay(anchor, Placement::Below, text("Contents"))
///             .with_node(|node| node.part(menu.content()).role(Role::Dialog))
///             .name("Options")
///     }),
/// );
/// ```
#[derive(Clone, Copy)]
pub struct PopupApi {
    disclosure: DisclosureApi,
    content_state: kivact_core::state::Signal<Option<Value<PopupState>>>,
    trigger_state: kivact_core::state::Signal<Option<Value<PopupState>>>,
    /// The reference the trigger part binds.
    own_anchor: NodeRef,
    settings: OptionsCell<PopupOptions>,
    machine: MachineHandle<Machine>,
}

/// What the setters of a [`PopupApi`] store; read when a surface binds.
#[derive(Clone, Copy, Default)]
struct PopupOptions {
    anchor: Option<NodeRef>,
    initial_focus: InitialFocus,
    focus_return: Option<FocusReturn>,
    return_focus_target: Option<Value<NodeRef>>,
    anchor_starts_focus_range: Option<Value<bool>>,
    modal: bool,
}

#[derive(Clone, Copy, Default)]
enum InitialFocus {
    #[default]
    Preserve,
    Explicit(NodeRef),
    Restore(NodeRef),
    First,
}

/// Create a popup whose open state is `open`, the sole writer.
///
/// No implicit open signal is created and creation does not write it. Setup
/// only.
#[track_caller]
pub fn use_popup(open: impl kivact_core::state::IntoBinding<bool>) -> PopupApi {
    let disclosure = use_disclosure(open);
    let settings = OptionsCell::new(PopupOptions::default());
    let own_anchor = node_ref();
    let machine = machine(
        State::Closed,
        move || Machine {
            focused: focused_node(),
            focus_inside: disclosure
                .content
                .mounted_node()
                .is_some_and(|panel| disclosure.content.focus_was_in(panel)),
        },
        move |cx, effect| match effect {
            machine::Effect::Misuse => raise(
                ErrorKind::Message,
                "PopupApi: one surface at a time; create one popup per simultaneous surface",
            ),
            machine::Effect::WriteClose { reason, opening } => {
                disclosure.open.set(false);
                if !disclosure.is_open().get_untracked() {
                    let target = opening.and_then(|(opener, policy)| {
                        return_target(
                            policy.0,
                            reason,
                            opener,
                            settings.get().anchor.unwrap_or(own_anchor),
                            settings.get().modal || disclosure.content.is_modal(),
                        )
                    });
                    cx.send(Event::Accepted { reason, target });
                }
            }
            machine::Effect::FocusInitial => {
                if disclosure.is_open().get_untracked() {
                    match settings.get().initial_focus {
                        InitialFocus::Preserve => {}
                        InitialFocus::Explicit(target) => {
                            if target.mounted_node().is_some() {
                                target.focus();
                            }
                        }
                        InitialFocus::Restore(target) => target.restore_focus(),
                        InitialFocus::First => disclosure.content.focus_first_inside(),
                    }
                }
            }
            machine::Effect::Return(opener) => opener.restore_focus(),
            machine::Effect::Unmounted => cx.send(Event::Unmount {
                was_inside: false,
                target: None,
            }),
        },
    );
    PopupApi {
        disclosure,
        content_state: kivact_core::state::signal(None),
        trigger_state: kivact_core::state::signal(None),
        own_anchor,
        settings,
        machine,
    }
}

impl PopupApi {
    pub(crate) fn modal(self, modal: bool) -> Self {
        self.settings.update(|options| options.modal = modal);
        self
    }

    /// Drawing state of the mounted trigger.
    pub fn trigger_state(self) -> Value<PopupState> {
        memo(move || {
            self.trigger_state
                .get()
                .map(|state| state.get())
                .unwrap_or(PopupState {
                    input: PartState::default(),
                    open: self.is_open().get(),
                })
        })
    }
    /// Drawing state of the mounted surface.
    pub fn content_state(self) -> Value<PopupState> {
        memo(move || {
            self.content_state
                .get()
                .map(|state| state.get())
                .unwrap_or(PopupState {
                    input: PartState::default(),
                    open: self.is_open().get(),
                })
        })
    }

    /// Use this node as the anchor instead of the trigger part's node.
    ///
    /// The anchor is the node focus returns to and part of the popup's focus
    /// range; it may be called before or after the parts are made. Use it
    /// when the node that opens the popup carries no [`trigger`](Self::trigger).
    /// Setup only.
    pub fn anchor(self, anchor: NodeRef) -> Self {
        self.settings
            .update(|settings| settings.anchor = Some(anchor));
        self
    }

    /// Request this mounted node's focus after each opening is acknowledged.
    /// Without an initial target the popup preserves the current focus.
    pub fn initial_focus(self, node: NodeRef) -> Self {
        self.settings
            .update(|settings| settings.initial_focus = InitialFocus::Explicit(node));
        self
    }

    pub(crate) fn focus_first(self) -> Self {
        self.settings
            .update(|settings| settings.initial_focus = InitialFocus::First);
        self
    }

    pub(crate) fn initial_focus_restore(self, target: NodeRef) -> Self {
        self.settings
            .update(|settings| settings.initial_focus = InitialFocus::Restore(target));
        self
    }

    pub(crate) fn anchor_starts_focus_range(self, enabled: impl IntoValue<bool>) -> Self {
        let enabled = enabled.into_value();
        self.settings
            .update(|settings| settings.anchor_starts_focus_range = Some(enabled));
        self
    }

    /// Choose where the focus goes when the surface closes and for which
    /// reasons. Without it the anchor takes the focus on Escape, a selection
    /// and assistive technology's dismissal. A modal's restoration stays the
    /// overlay's own. Copies of the popup share the policy.
    pub fn focus_return(self, policy: FocusReturn) -> Self {
        self.settings
            .update(|settings| settings.focus_return = Some(policy));
        self
    }

    pub(crate) fn return_focus_target(self, target: Value<NodeRef>) -> Self {
        self.settings
            .update(|settings| settings.return_focus_target = Some(target));
        self
    }

    /// Replace the policy of the opening in progress, for its close. Runs
    /// while the surface is bound.
    pub(crate) fn set_opening_policy(self, policy: FocusReturn) {
        self.machine.send(Event::Policy(policy));
    }

    /// The caller's policy, or the anchor on `DISMISS`.
    fn base_policy(self) -> FocusReturn {
        if let Some(target) = self.settings.get().return_focus_target {
            return FocusReturn {
                target: FocusReturnTarget::Anchor(target.get_untracked()),
                on: crate::ReasonSet::DISMISS
                    .union(crate::ReasonSet::PROGRAMMATIC)
                    .union(crate::ReasonSet::UNMOUNT),
            };
        }
        self.settings
            .get()
            .focus_return
            .unwrap_or_else(|| FocusReturn::anchor(self.anchor_ref()))
    }

    /// The application's authoritative open state.
    pub fn is_open(self) -> Value<bool> {
        self.disclosure.is_open()
    }

    /// The anchor: the node given to [`anchor`](Self::anchor), else the
    /// trigger part's.
    pub(crate) fn anchor_ref(self) -> NodeRef {
        self.settings.get().anchor.unwrap_or(self.own_anchor)
    }

    pub(crate) fn anchor_value(self) -> Value<NodeRef> {
        memo(move || {
            self.settings
                .value()
                .get()
                .anchor
                .unwrap_or(self.own_anchor)
        })
    }

    pub(crate) fn open_binding(self) -> Binding<bool> {
        self.disclosure.open
    }

    /// The reference the surface part binds, for crate behaviors that build
    /// their own trigger.
    pub(crate) fn content_ref(self) -> NodeRef {
        self.disclosure.content
    }

    /// Request closure and apply the focus-return policy only after the
    /// controlled binding accepts it. A missing target receives no focus request.
    #[track_caller]
    pub fn close(self, reason: PopupCloseReason) {
        if !self.is_open().get_untracked() {
            return;
        }
        self.machine.send(Event::Close(reason));
    }

    fn finish_close(self, reason: PopupCloseReason) {
        let modal = self.content_ref().is_modal();
        let open = self.is_open();
        if open.get_untracked() {
            self.disclosure.open.set(false);
        }
        if open.get_untracked() || modal {
            return;
        }
        self.machine.send(Event::Accepted {
            reason,
            target: self.return_target(reason),
        });
    }

    fn return_target(self, reason: PopupCloseReason) -> Option<kivact_core::custom::NodeToken> {
        let State::Open { opener, policy, .. } = self.machine.state().get_untracked() else {
            return None;
        };
        return_target(
            policy.0,
            reason,
            opener,
            self.anchor_ref(),
            self.settings.get().modal || self.content_ref().is_modal(),
        )
    }

    fn toggle(self) {
        if self.is_open().get() {
            self.close(PopupCloseReason::Programmatic);
        } else {
            self.disclosure.open.set(true);
        }
    }

    /// The trigger: a button publishing `expanded` and `controls` that toggles.
    ///
    /// Activation opens a closed popup and closes an open one as
    /// [`PopupCloseReason::Programmatic`]; Expand opens and Collapse closes as
    /// [`PopupCloseReason::Accessibility`]; the release of Tab on the trigger
    /// closes as [`PopupCloseReason::Tab`]. They run after the caller's own
    /// handlers. The part binds the popup's anchor.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        let base = self.base_trigger();
        Part::new(base.state(), move |host| {
            host.part(base)
                .on_press(move || self.toggle())
                .a11y_action(A11yAction::Expand, move |_| self.disclosure.open.set(true))
                .a11y_action(A11yAction::Collapse, move |_| {
                    self.close(PopupCloseReason::Accessibility)
                })
        })
    }

    /// Shared trigger anatomy and Tab departure; its caller supplies activation.
    #[track_caller]
    pub(crate) fn base_trigger(self) -> Part<PopupState> {
        let base = self
            .disclosure
            .base_trigger(|input, open| PopupState { input, open });
        let part_state = base.state();
        self.trigger_state.set(Some(part_state));
        Part::new(part_state, move |host| {
            host.part(base)
                .node_ref(self.own_anchor)
                // The host consumes Tab to move focus, so only its release is
                // seen, by the element that has the focus now. Shift+Tab from the
                // surface back to the trigger leaves the surface, which stays open
                // as the anchor is part of its focus range.
                .on_key_up(move |event: KeyEvent| {
                    if event.key() == Key::Tab && self.is_open().get() {
                        self.close(PopupCloseReason::Tab);
                    }
                    Handled::No
                })
        })
    }

    /// The floating surface, for the caller's `overlay(..)`.
    ///
    /// It connects the overlay's dismissal to this popup: Escape, an outside
    /// press and assistive technology's Dismiss close it with the matching
    /// reason after the caller's own dismissal handlers, and an error from one
    /// of those stops the close. It also requests the initial focus, closes on
    /// focus departure and anchor removal, and binds the reference the trigger
    /// `controls`. Roles, names, placement and modality stay the caller's.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        self.content_with(|_| true)
    }

    /// The surface whose dismissal closes only for the reasons `allow` admits.
    #[track_caller]
    pub(crate) fn content_with(
        self,
        allow: impl Fn(DismissReason) -> bool + 'static,
    ) -> Part<PopupState> {
        let open = self.is_open();
        let input = part(move |input| PopupState {
            input,
            open: open.get(),
        });
        let part_state = input.state();
        self.content_state.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .node_ref(self.content_ref())
                .a11y_action(A11yAction::Dismiss, move |event| {
                    let A11yRequest::Dismiss(reason) = event.request else {
                        unreachable!(
                            "invariant: the DismissOn slot receives only DismissOn requests"
                        )
                    };
                    if !allow(reason) {
                        return;
                    }
                    // An earlier caller dismissal handler may already have closed the
                    // binding. Its surface is still mounted during this same handler
                    // chain, so the reason's focus policy still applies.
                    self.finish_close(match reason {
                        DismissReason::Escape => PopupCloseReason::Escape,
                        DismissReason::OutsidePress => PopupCloseReason::OutsidePress,
                        DismissReason::Accessibility => PopupCloseReason::Accessibility,
                        DismissReason::Timeout => PopupCloseReason::Programmatic,
                    });
                })
                .on_attach(move |_| self.watch())
        })
    }

    /// Per-mount focus range, anchor removal and initial focus. Setup only,
    /// under the mounted surface's owner.
    fn watch(self) {
        let owner = Owner::current();
        self.machine.send(Event::Open {
            opener: focused_node(),
            policy: self.base_policy(),
        });
        let surface = self.content_ref();
        let panel = surface.mounted_node();
        on_mount(move || {
            if surface.is_modal() {
                self.machine.send(Event::Policy(FocusReturn::preserve()));
            }
        });
        on_cleanup(move || {
            self.machine.send(Event::Unmount {
                was_inside: panel.is_some_and(|node| surface.focus_was_in(node)),
                target: self.return_target(PopupCloseReason::Unmount),
            });
        });
        let within = self.content_ref().focus_within();
        let anchor = self.anchor_ref();
        let anchor_focused = anchor.focused();
        let open = self.is_open();
        let anchor_starts = self.settings.get().anchor_starts_focus_range;
        effect(move || {
            if !open.get() || self.settings.get().modal {
                return;
            }
            let focused = owner.focused_node();
            let anchor_present = anchor.tracked_node().is_some();
            let anchor_focused = anchor_focused.get();
            let entered = within.get()
                || (anchor_starts.is_some_and(|enabled| enabled.get()) && anchor_focused);
            self.machine.send(Event::Focus {
                anchor_present,
                entered,
                outside: !anchor_focused && focused.is_some(),
            });
        });
        self.machine.feed(open, |_| Event::FocusInitial);
        on_mount(move || self.machine.send(Event::FocusInitial));
    }
}

fn return_target(
    policy: FocusReturn,
    reason: PopupCloseReason,
    opener: Option<kivact_core::custom::NodeToken>,
    anchor: NodeRef,
    modal: bool,
) -> Option<kivact_core::custom::NodeToken> {
    if modal || !policy.restores(reason) {
        return None;
    }
    match policy.target {
        FocusReturnTarget::Opener => opener.or_else(|| anchor.mounted_node()),
        FocusReturnTarget::Anchor(reference) => reference.mounted_node(),
    }
}
