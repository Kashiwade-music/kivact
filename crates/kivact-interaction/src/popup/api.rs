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
    IntoValue, MachineHandle, Signal, Value, effect, machine, on_cleanup, on_mount,
};
use kivact_core::view::{
    A11yAction, A11yRequest, DismissReason, Element, Handled, IntoView, Key, KeyEvent, NodeRef,
    Overlay, Role, SurfacePlacement, View, node_ref, overlay, show,
};

use super::machine::{self, Event, Machine, State};
use crate::options_cell::OptionsCell;

use crate::disclosure::{DisclosureApi, use_disclosure};
use crate::focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason};

/// Gives an overlay the align and offset its surface asked for; a bare
/// placement keeps the overlay's own.
pub(crate) fn with_placement(layer: Overlay, placement: SurfacePlacement) -> Overlay {
    let layer = match placement.align {
        Some(align) => layer.align(align),
        None => layer,
    };
    match placement.offset {
        Some(offset) => layer.offset(offset),
        None => layer,
    }
}

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
/// use kivact::{state::signal, view::{Element, node, text, Placement}};
/// use kivact::interaction::use_popup;
/// let open = signal(false);
/// let menu = use_popup(open);
/// let view = (
///     node().part(menu.trigger()).name("Options").child(text("Options")),
///     menu.surface(Placement::Below, || text("Contents")),
/// );
/// ```
#[derive(Clone, Copy)]
pub struct PopupApi {
    disclosure: DisclosureApi,
    /// The reference the trigger part binds.
    own_anchor: NodeRef,
    /// The surface's title, a heading the surface is `labelled_by`.
    title: NodeRef,
    /// The surface's description, which the surface is `described_by`.
    description: NodeRef,
    settings: OptionsCell<PopupOptions>,
    machine: MachineHandle<Machine>,
}

/// What the setters of a [`PopupApi`] store; read when a surface binds.
#[derive(Clone, Copy, Default)]
struct PopupOptions {
    anchor: Option<NodeRef>,
    initial_focus: InitialFocus,
    focus_return: Option<FocusReturn>,
    anchor_starts_focus_range: Option<Value<bool>>,
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
pub fn use_popup(open: kivact_core::state::Signal<bool>) -> PopupApi {
    let disclosure = use_disclosure(open);
    let settings = OptionsCell::new(PopupOptions::default());
    let own_anchor = node_ref();
    let machine = machine(
        State::Closed,
        move || Machine {
            focus: match focused_node() {
                Some(_)
                    if disclosure
                        .content
                        .mounted_node()
                        .is_some_and(|panel| disclosure.content.focus_was_in(panel)) =>
                {
                    machine::HostFocus::Inside
                }
                Some(token) => machine::HostFocus::Elsewhere(token),
                None => machine::HostFocus::Nowhere,
            },
        },
        move |cx, effect| match effect {
            machine::Effect::Misuse => raise(
                ErrorKind::Message,
                "PopupApi: one surface at a time; create one popup per simultaneous surface",
            ),
            machine::Effect::Dismiss(reason) => {
                cx.send(Event::Closing(reason));
                if disclosure.is_open().get_untracked() {
                    disclosure.open.set(false);
                }
            }
            machine::Effect::Accept {
                reason,
                opener,
                policy,
            } => {
                if !disclosure.is_open().get_untracked() {
                    let target = return_target(
                        policy.0,
                        reason,
                        opener,
                        settings.get().anchor.unwrap_or(own_anchor),
                        disclosure.content.is_modal(),
                    );
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
        },
    );
    PopupApi {
        disclosure,
        own_anchor,
        title: node_ref(),
        description: node_ref(),
        settings,
        machine,
    }
}

impl PopupApi {
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

    /// Start the focus on the surface's first focusable node at each opening.
    /// Setup only.
    pub fn initial_focus_first(self) -> Self {
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

    /// Replace the policy of the opening in progress, for its close. Runs
    /// while the surface is bound.
    pub(crate) fn set_opening_policy(self, policy: FocusReturn) {
        self.machine.send(Event::Policy(policy));
    }

    /// The caller's policy, or the anchor on `DISMISS`, read at each opening.
    fn base_policy(self) -> FocusReturn {
        self.settings
            .get()
            .focus_return
            .unwrap_or_else(|| FocusReturn::anchor(self.anchor_ref()))
            .resolved()
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

    pub(crate) fn open_binding(self) -> Signal<bool> {
        self.disclosure.open
    }

    /// The reference the surface part binds, for crate behaviors that build
    /// their own trigger.
    pub(crate) fn content_ref(self) -> NodeRef {
        self.disclosure.content
    }

    /// Request closure with `reason`: the focus return follows the caller's
    /// `open` write. A missing target receives no focus request.
    #[track_caller]
    pub(crate) fn close(self, reason: PopupCloseReason) {
        if self.is_open().get_untracked() {
            self.note_close(reason);
        }
    }

    /// Note `reason` for this opening, then write `open` false. The follow of
    /// `open` (or the surface's unmount) accepts the close with that reason.
    fn note_close(self, reason: PopupCloseReason) {
        self.machine.send(Event::Closing(reason));
        if self.is_open().get_untracked() {
            self.disclosure.open.set(false);
        }
    }

    /// Show: write `true` to the open state unless it is shown.
    pub fn show(self) {
        if !self.is_open().get_untracked() {
            self.disclosure.open.set(true);
        }
    }

    /// Close as an explicit close of the application.
    pub fn hide(self) {
        self.close(PopupCloseReason::Programmatic);
    }

    /// Show a hidden popup, hide a shown one.
    pub fn toggle(self) {
        if self.is_open().get_untracked() {
            self.hide();
        } else {
            self.show();
        }
    }

    /// The focus return of an unmount: by the reason noted for this opening's
    /// close, else as an unmount.
    fn unmount_target(self) -> Option<kivact_core::custom::NodeToken> {
        let reason = match self.machine.state().get_untracked() {
            State::Open {
                closing: Some(reason),
                ..
            } => reason,
            _ => PopupCloseReason::Unmount,
        };
        self.return_target(reason)
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
            self.content_ref().is_modal(),
        )
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
        Part::new(part_state, move |host| {
            host.part(input)
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
                    // binding. The surface is still mounted during this handler
                    // chain, so the close effect still computes the reason's focus
                    // return.
                    self.note_close(match reason {
                        DismissReason::Escape | DismissReason::WindowClose => {
                            PopupCloseReason::Escape
                        }
                        DismissReason::OutsidePress => PopupCloseReason::OutsidePress,
                        DismissReason::Accessibility => PopupCloseReason::Accessibility,
                        DismissReason::Timeout => PopupCloseReason::Programmatic,
                    });
                })
                .labelled_by(self.title)
                .described_by(self.description)
                .on_attach(move |_| self.watch())
        })
    }

    /// The title: a level 2 heading the surface is `labelled_by`.
    #[track_caller]
    pub fn title(self) -> Part<PopupState> {
        let open = self.is_open();
        let input = part(move |input| PopupState {
            input,
            open: open.get(),
        });
        let part_state = input.state();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(self.title)
                .role(Role::Heading)
                .level(2_u32)
        })
    }

    /// A description the surface is `described_by`.
    #[track_caller]
    pub fn description(self) -> Part<PopupState> {
        let open = self.is_open();
        let input = part(move |input| PopupState {
            input,
            open: open.get(),
        });
        let part_state = input.state();
        Part::new(part_state, move |host| {
            host.part(input).node_ref(self.description)
        })
    }

    /// A button whose activation closes the popup as a selection.
    #[track_caller]
    pub fn close_trigger(self) -> Part<PopupState> {
        let open = self.is_open();
        let input = part(move |input| PopupState {
            input,
            open: open.get(),
        });
        let part_state = input.state();
        Part::new(part_state, move |host| {
            host.part(input)
                .role(Role::Button)
                .focusable(true)
                .on_press(move || self.close(PopupCloseReason::Selection))
        })
    }

    /// The floating surface with its content, mounted at the anchor while the
    /// popup is shown, in a fresh owner on every opening.
    #[track_caller]
    pub fn surface<V: IntoView>(
        self,
        placement: impl Into<SurfacePlacement>,
        content: impl Fn() -> V + 'static,
    ) -> View {
        let placement: SurfacePlacement = placement.into();
        show(self.is_open(), move || {
            with_placement(
                overlay(self.anchor_ref(), placement.side, content()),
                placement,
            )
            .with_node(|node| node.part(self.content()))
        })
        .into_view()
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
                target: self.unmount_target(),
            });
        });
        let within = self.content_ref().focus_within();
        let anchor = self.anchor_ref();
        let anchor_focused = anchor.focused();
        let open = self.is_open();
        let anchor_starts = self.settings.get().anchor_starts_focus_range;
        effect(move || {
            if !open.get() || surface.is_modal() {
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
        self.machine.feed(open, |shown| {
            if shown {
                Event::FocusInitial
            } else {
                Event::Hidden
            }
        });
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
        FocusReturnTarget::AnchorValue(value) => value.get_untracked().mounted_node(),
    }
}
