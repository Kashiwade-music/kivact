//! Role: Project preview visibility and provenance from the shared hover surface.
use crate::{DelayedPhase, Delays, HoverSurfaceApi, PopupState};
use kivact_core::custom::{Part, part};
use kivact_core::state::{IntoValue, Signal, Value, memo};
use kivact_core::view::{
    A11yAction, A11yRequest, DismissReason, Element, IntoView, NodeRef, Placement, View, node_ref,
    overlay, show,
};

/// The input that currently requests a preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverCardOpenedBy {
    Pointer,
    Focus,
    Signal,
}
/// Canonical preview phase, projected from the lower visibility authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverCardState {
    Closed,
    Opening,
    Open { by: HoverCardOpenedBy },
    Closing,
    Dismissed,
}
/// Shared preview visibility; the lower surface owns all delay and dismissal state.
#[derive(Clone, Copy)]
pub struct HoverCardApi {
    surface: HoverSurfaceApi,
    state: Value<HoverCardState>,
    open: Signal<bool>,
    shown: Value<bool>,
    /// The node the preview is placed at and whose hover and focus open it.
    anchor: NodeRef,
    /// The preview surface; its focus keeps the preview open.
    card: NodeRef,
}
/// Connect pointer travel and focus or external opening to one surface.
/// `open` is the caller's open state; the surface writes it as it shows and hides.
/// `focus_opens` gates opening on focus; focus within the anchor or the card counts.
#[track_caller]
pub fn use_hover_card(
    enabled: impl IntoValue<bool>,
    focus_opens: impl IntoValue<bool>,
    timing: Delays,
    open: Signal<bool>,
) -> HoverCardApi {
    let anchor = node_ref();
    let card = node_ref();
    let (anchor_focus, card_focus) = (anchor.focus_within(), card.focus_within());
    let focus_opens = focus_opens.into_value();
    let focused = memo(move || focus_opens.get() && (anchor_focus.get() || card_focus.get()));
    let enabled = enabled.into_value();
    let immediate = focused;
    // The surface's visibility changes are the only writer of `open`;
    // the caller's own writes reach the surface through `feed`.
    let surface = crate::hover_corridor::hover_surface(
        anchor,
        card,
        enabled,
        immediate,
        timing,
        move |visible| open.set(visible),
    );
    surface.hurry_on(immediate);
    surface.dismiss_on_release(open);
    let shown = memo(move || enabled.get() && open.get());
    let state = memo(move || {
        if shown.get() {
            match surface.phase().get() {
                DelayedPhase::Closing => HoverCardState::Closing,
                DelayedPhase::Shown => HoverCardState::Open {
                    by: if focused.get() {
                        HoverCardOpenedBy::Focus
                    } else {
                        HoverCardOpenedBy::Pointer
                    },
                },
                _ => HoverCardState::Open {
                    by: HoverCardOpenedBy::Signal,
                },
            }
        } else {
            match surface.phase().get() {
                DelayedPhase::Opening => HoverCardState::Opening,
                DelayedPhase::Dismissed => HoverCardState::Dismissed,
                _ => HoverCardState::Closed,
            }
        }
    });
    HoverCardApi {
        surface,
        state,
        open,
        shown,
        anchor,
        card,
    }
}

impl HoverCardApi {
    /// Current phase and active opening source, without duplicate stored phase.
    pub fn state(self) -> Value<HoverCardState> {
        self.state
    }
    /// Whether the content should be mounted, including the close delay.
    pub fn is_open(self) -> Value<bool> {
        self.shown
    }
    /// Show the preview unless it is shown.
    pub fn show(self) {
        if !self.shown.get_untracked() {
            self.open.set(true);
        }
    }
    /// Show a hidden preview, hide a shown one.
    pub fn toggle(self) {
        if self.shown.get_untracked() {
            self.hide();
        } else {
            self.show();
        }
    }
    /// Hide until the current engagement ends.
    pub fn hide(self) {
        self.open.set(false);
        self.surface.hide();
    }
    /// The node the preview is placed at and whose hover and focus open it.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        let shown = self.shown;
        let input = part(move |input| PopupState {
            input,
            open: shown.get(),
        });
        Part::new(input.state(), move |host| {
            host.part(input).node_ref(self.anchor)
        })
    }
    /// The preview surface: Escape and an outside press hide it, and
    /// Escape inside it moves the focus back to the trigger.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        let shown = self.shown;
        let card_focus = self.card.focus_within();
        let input = part(move |input| PopupState {
            input,
            open: shown.get(),
        });
        Part::new(input.state(), move |host| {
            host.part(input)
                .node_ref(self.card)
                .a11y_action(A11yAction::Dismiss, move |event| {
                    let A11yRequest::Dismiss(reason) = event.request else {
                        unreachable!("invariant: the Dismiss slot receives only Dismiss requests")
                    };
                    if reason == DismissReason::Escape && card_focus.get_untracked() {
                        self.anchor.focus();
                    }
                    self.hide();
                })
        })
    }
    /// The preview surface placed at the anchor while it is shown, in a fresh
    /// owner on every opening.
    #[track_caller]
    pub fn surface<V: IntoView>(
        self,
        placement: Placement,
        content: impl Fn() -> V + 'static,
    ) -> View {
        show(self.shown, move || {
            overlay(self.anchor, placement, content()).with_node(|node| node.part(self.content()))
        })
        .into_view()
    }
}
