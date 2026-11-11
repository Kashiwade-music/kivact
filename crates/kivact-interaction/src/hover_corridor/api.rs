//! Role: Preserve a hover-open surface while the mouse travels towards it.

use std::time::Duration;

use kivact_core::geometry::Rect;
use kivact_core::state::IntoValue;
use kivact_core::state::{Notifier, Value, effect, machine, memo, notifier};
use kivact_core::view::{NodeRef, pointer_position};

use super::machine::{Corridor, Effect, Event, Machine, PointerSampleCache, State};
use crate::{DelayedApi, Delays};

/// Hold a submenu while the mouse moves from an anchor toward the surface.
///
/// Geometry is measured in window coordinates. The hold starts on departure
/// from the anchor, renews only with forward movement inside the triangle,
/// and expires after `timeout` without progress. It does not open anything.
#[track_caller]
pub fn use_hover_corridor(
    anchor: impl IntoValue<Option<Rect>>,
    surface: impl IntoValue<Option<Rect>>,
    enabled: impl IntoValue<bool>,
    timeout: Duration,
) -> HoverCorridorApi {
    let holding = hover_corridor(
        anchor.into_value(),
        surface.into_value(),
        enabled.into_value(),
        timeout,
    )
    .0;
    HoverCorridorApi { holding }
}

/// Delay opening and closing a hover surface. A diagonal path from the
/// anchor toward the open surface restarts the existing close delay.
#[track_caller]
pub fn use_hover_surface(
    anchor: NodeRef,
    surface: NodeRef,
    enabled: impl IntoValue<bool>,
    delays: Delays,
) -> HoverSurfaceApi {
    hover_surface(
        anchor,
        surface,
        enabled.into_value(),
        (|| false).into_value(),
        delays,
        |_| (),
    )
}

pub(crate) fn hover_surface(
    anchor: NodeRef,
    surface: NodeRef,
    enabled: Value<bool>,
    immediate: Value<bool>,
    delays: Delays,
    on_visible: impl Fn(bool) + 'static,
) -> HoverSurfaceApi {
    let over_anchor = anchor.hovered();
    let over_surface = surface.hovered();
    let wanted =
        (move || enabled.get() && (immediate.get() || over_anchor.get() || over_surface.get()))
            .into_value();
    let delayed = crate::delayed::delayed(
        wanted,
        move || {
            if immediate.get_untracked() {
                Duration::ZERO
            } else {
                delays.open
            }
        },
        delays.close,
        on_visible,
    );
    let shown = delayed.is_open();
    let (_, progress) = hover_corridor(
        anchor.rect(),
        surface.rect(),
        (move || enabled.get() && shown.get()).into_value(),
        delays.close,
    );
    effect(move || {
        progress.track();
        if shown.get_untracked() && !over_anchor.get_untracked() && !over_surface.get_untracked() {
            delayed.pulse();
        }
    });
    HoverSurfaceApi { delayed }
}

fn hover_corridor(
    anchor: Value<Option<Rect>>,
    surface: Value<Option<Rect>>,
    enabled: Value<bool>,
    timeout: Duration,
) -> (Value<bool>, Notifier) {
    let mouse = pointer_position();
    let progress = notifier();
    let state = machine(
        State {
            corridor: Corridor::Idle,
            sample_cache: PointerSampleCache::default(),
        },
        move || Machine { timeout },
        move |_, effect| match effect {
            Effect::Progress => progress.notify(),
        },
    );
    effect(move || {
        state.send(Event::Sample {
            geometry: anchor.get().zip(surface.get()),
            at: mouse.get(),
            available: enabled.get() && !timeout.is_zero(),
        })
    });
    (
        memo(move || matches!(state.get().corridor, Corridor::Holding { .. })),
        progress,
    )
}

/// Tracks whether travel towards the surface currently holds it open.
#[derive(Clone, Copy)]
pub struct HoverCorridorApi {
    holding: Value<bool>,
}
impl HoverCorridorApi {
    /// Whether the pointer is advancing through the corridor.
    pub fn is_holding(self) -> Value<bool> {
        self.holding
    }
}
/// Delayed visibility of a hover surface, including its travel corridor.
#[derive(Clone, Copy)]
pub struct HoverSurfaceApi {
    delayed: DelayedApi,
}
impl HoverSurfaceApi {
    /// Whether the surface is open.
    pub fn is_open(self) -> Value<bool> {
        self.delayed.is_open()
    }
    /// Observe the visibility transition.
    pub fn phase(self) -> Value<crate::DelayedPhase> {
        self.delayed.phase()
    }
    /// Hide until the pointer leaves and enters again.
    pub fn hide(self) {
        self.delayed.hide();
    }
    /// Renew the close delay.
    pub fn pulse(self) {
        self.delayed.pulse();
    }
    pub(crate) fn dismiss_on_release(self, requested: impl IntoValue<bool>) {
        self.delayed.dismiss_on_release(requested);
    }
    pub(crate) fn hurry_on(self, immediate: impl IntoValue<bool>) {
        self.delayed.hurry_on(immediate);
    }
}
