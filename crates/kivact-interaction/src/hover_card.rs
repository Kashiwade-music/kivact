//! Role: Project preview visibility and provenance from the shared hover surface.
use crate::{DelayedPhase, Delays, HoverCardTiming, HoverSurfaceApi};
use kivact_core::state::{Binding, IntoBinding, IntoValue, Value, effect, effect_on, memo};
use kivact_core::view::NodeRef;

/// The input that currently requests a preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverCardOpenedBy {
    Pointer,
    Focus,
    Binding,
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
    open: Binding<bool>,
    shown: Value<bool>,
}
/// Connect pointer travel and immediate focus or external opening to one surface.
pub fn use_hover_card(
    anchor: NodeRef,
    surface: NodeRef,
    enabled: impl IntoValue<bool>,
    focused: impl IntoValue<bool>,
    requested: impl IntoBinding<bool>,
    timing: HoverCardTiming,
) -> HoverCardApi {
    let requested = requested.into_binding();
    let focused = focused.into_value();
    let enabled = enabled.into_value();
    let immediate = focused;
    let surface = crate::hover_corridor::hover_surface(
        anchor,
        surface,
        enabled,
        immediate,
        Delays {
            open: timing.open_delay,
            close: timing.close_delay,
        },
    );
    effect(move || {
        if immediate.get() && surface.phase().get() == DelayedPhase::Opening {
            surface.pulse();
        }
    });
    effect_on(surface.phase(), move |phase| match phase {
        DelayedPhase::Shown => requested.set(true),
        DelayedPhase::Hidden | DelayedPhase::Dismissed => requested.set(false),
        DelayedPhase::Opening | DelayedPhase::Closing => (),
    });
    effect_on(requested, move |open| {
        if !open {
            surface.dismiss();
        }
    });
    let shown = memo(move || enabled.get() && requested.get());
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
                    by: HoverCardOpenedBy::Binding,
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
        open: requested,
        shown,
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
    /// Hide until the current engagement ends.
    pub fn dismiss(self) {
        self.open.set(false);
        self.surface.dismiss();
    }
}
