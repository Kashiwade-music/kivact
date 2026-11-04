//! Role: Popup adds overlay focus, dismissal and restoration to a disclosure.
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

use std::panic::Location;

use kivact_core::diagnostics::ErrorKind;
use kivact_core::primitive::{
    NodeRefExt, NodeToken, Owner, Part, PartState, focused_node, part, raise,
};
use kivact_core::view::{
    A11yAction, A11yRequest, DismissReason, Handled, Key, KeyEvent, NodeRef, node_ref,
};
use kivact_core::{Binding, Value, Var, effect, on_cleanup, on_mount, var};

use super::disclosure::{Disclosure, use_disclosure};
use super::focus_return::{FocusReturn, PopupCloseReason};

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
/// use kivact_core::{signal, control::{NodeBuilder, node, text}, view::{Placement, Role, overlay, show}};
/// use kivact_interaction::use_popup;
/// use kivact_core::primitive::NodeExt;
/// let open = signal(false);
/// let menu = use_popup(open);
/// let trigger = menu.trigger();
/// let anchor = trigger.node_ref();
/// let view = (
///     node().part(trigger).name("Options").child(text("Options")),
///     show(menu.shown(), move || {
///         overlay(anchor, Placement::Below, text("Contents"))
///             .part(menu.content())
///             .role(Role::Dialog)
///             .name("Options")
///     }),
/// );
/// ```
#[derive(Clone, Copy)]
pub struct Popup {
    disclosure: Disclosure,
    /// The reference the trigger part binds.
    own_anchor: NodeRef,
    settings: Var<PopupSettings>,
    opening: Var<Option<Opening>>,
}

/// What the setters of a [`Popup`] store; read when a surface binds.
#[derive(Clone, Copy, Default)]
struct PopupSettings {
    anchor: Option<NodeRef>,
    initial_focus: Option<NodeRef>,
    focus_return: Option<FocusReturn>,
    anchor_starts_focus_range: bool,
}

/// What one opening of the surface remembers, from its bind to its removal.
#[derive(Clone, Copy)]
struct Opening {
    /// The node that had the focus when this opening's surface bound.
    opener: Option<NodeToken>,
    /// This opening's policy: the caller's (or the default) at bind, replaced
    /// by `set_opening_policy`.
    policy: FocusReturn,
    /// The accepted close reason of this opening, once there is one.
    closed_by: Option<PopupCloseReason>,
    /// Whether this opening's restoration was already requested.
    returned: bool,
}

/// Create a popup whose open state is `open`, the sole writer.
///
/// No implicit open signal is created and creation does not write it. Setup
/// only.
#[track_caller]
pub fn use_popup(open: impl Into<Binding<bool>>) -> Popup {
    Popup {
        disclosure: use_disclosure(open),
        own_anchor: node_ref(),
        settings: var(PopupSettings::default()),
        opening: var(None),
    }
}

impl Popup {
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
            .update(|settings| settings.initial_focus = Some(node));
        self
    }

    pub(crate) fn anchor_starts_focus_range(self, enabled: bool) -> Self {
        self.settings
            .update(|settings| settings.anchor_starts_focus_range = enabled);
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
        self.opening.update(|opening| {
            opening
                .as_mut()
                .expect("invariant: set_opening_policy runs while the surface is bound")
                .policy = policy;
        });
    }

    /// The caller's policy, or the anchor on `DISMISS`.
    fn base_policy(self) -> FocusReturn {
        self.settings
            .with(|settings| settings.focus_return)
            .unwrap_or_else(|| FocusReturn::anchor(self.anchor_ref()))
    }

    /// Submit the policy's restoration for this opening's `reason`, once.
    fn request_return(self, reason: PopupCloseReason) {
        let Some(opening) = self.opening.get() else {
            return;
        };
        if opening.returned || !opening.policy.restores(reason) {
            return;
        }
        self.opening.update(|opening| {
            if let Some(opening) = opening {
                opening.returned = true;
            }
        });
        opening.policy.request(
            reason,
            opening.opener.or_else(|| self.anchor_ref().mounted_node()),
        );
    }

    /// The application's authoritative open state.
    pub fn shown(self) -> Value<bool> {
        self.disclosure.shown()
    }

    /// The anchor: the node given to [`anchor`](Self::anchor), else the
    /// trigger part's.
    pub(crate) fn anchor_ref(self) -> NodeRef {
        self.settings
            .with(|settings| settings.anchor)
            .unwrap_or(self.own_anchor)
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
        if !self.shown().get_untracked() {
            return;
        }
        self.finish_close(reason);
    }

    fn finish_close(self, reason: PopupCloseReason) {
        let modal = self.content_ref().is_modal();
        let open = self.shown();
        if open.get_untracked() {
            self.disclosure.open.set(false);
        }
        if open.get_untracked() || modal {
            return;
        }
        self.opening.update(|opening| {
            if let Some(opening) = opening {
                opening.closed_by = Some(reason);
            }
        });
        // A dismissal restores now, as the surface is still mounted. The
        // other reasons wait for the surface's removal, which knows whether
        // the focus was inside it.
        if matches!(
            reason,
            PopupCloseReason::Escape
                | PopupCloseReason::Selection
                | PopupCloseReason::Accessibility
        ) {
            self.request_return(reason);
        }
    }

    fn toggle(self) {
        if self.shown().get() {
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
        self.disclosure
            .base_trigger(|input, open| PopupState { input, open })
            .reference(self.own_anchor)
            // The host consumes Tab to move focus, so only its release is
            // seen, by the element that has the focus now. Shift+Tab from the
            // surface back to the trigger leaves the surface, which stays open
            // as the anchor is part of its focus range.
            .on_key_up(move |event: KeyEvent| {
                if event.key() == Key::Tab && self.shown().get() {
                    self.close(PopupCloseReason::Tab);
                }
                Handled::No
            })
            .on_press(move || self.toggle())
            .on_a11y(A11yAction::Expand, move || self.disclosure.open.set(true))
            .on_a11y(A11yAction::Collapse, move || {
                self.close(PopupCloseReason::Accessibility)
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
        let at = Location::caller();
        let open = self.shown();
        part(move |input| PopupState {
            input,
            open: open.get(),
        })
        .reference(self.content_ref())
        .on_a11y_event(A11yAction::Dismiss, move |event| {
            let A11yRequest::Dismiss(reason) = event.request else {
                unreachable!("invariant: the Dismiss slot receives only Dismiss requests")
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
            });
        })
        .on_bind(move |_| self.watch(at))
    }

    /// Per-mount focus range, anchor removal and initial focus. Setup only,
    /// under the mounted surface's owner.
    fn watch(self, at: &'static Location<'static>) {
        if self.opening.with(Option::is_some) {
            raise(
                ErrorKind::Message,
                "Popup: one surface at a time; create one popup per simultaneous surface",
                at,
            );
        }
        let owner = Owner::current("Popup::watch");
        self.opening.set(Some(Opening {
            opener: focused_node(),
            policy: self.base_policy(),
            closed_by: None,
            returned: false,
        }));
        let surface = self.content_ref();
        let panel = surface.mounted_node();
        // A modal's restoration is the overlay's own; known once mounted.
        let modal = var(false);
        on_mount(move || modal.set(surface.is_modal()));
        on_cleanup(move || {
            if !modal.get() && panel.is_some_and(|node| surface.focus_was_in(node)) {
                let closed_by = self
                    .opening
                    .with(|opening| opening.and_then(|opening| opening.closed_by));
                self.request_return(closed_by.unwrap_or(PopupCloseReason::Unmount));
            }
            self.opening.set(None);
        });
        let within = self.content_ref().focus_within();
        let anchor = self.anchor_ref();
        let anchor_focused = anchor.focused();
        let open = self.shown();
        let anchor_starts = self
            .settings
            .with(|settings| settings.anchor_starts_focus_range);
        let entered = var(false);
        let anchor_seen = var(false);
        effect(move || {
            if !open.get() {
                return;
            }
            let focused = owner.focused_node();
            if anchor.tracked_node(at).is_some() {
                anchor_seen.set(true);
            } else if anchor_seen.get() {
                self.close(PopupCloseReason::AnchorRemoved);
                return;
            }
            if within.get() || (anchor_starts && anchor_focused.get()) {
                entered.set(true);
            } else if entered.get() && !anchor_focused.get() && focused.is_some() {
                self.close(PopupCloseReason::Tab);
            }
        });
        if let Some(initial) = self.settings.with(|settings| settings.initial_focus) {
            on_mount(move || {
                if open.get() && initial.mounted_node().is_some() {
                    initial.focus();
                }
            });
        }
    }
}
