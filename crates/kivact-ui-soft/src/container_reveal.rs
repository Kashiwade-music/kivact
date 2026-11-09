//! Role: Reveal in-place actions on hover and keyboard focus.

use kivact::interaction::Delays;
use kivact::interaction::use_delayed;
use kivact::motion::Transition;
use kivact::state::memo;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::TransitionProp;
use kivact::style::style;
use kivact::view::NodeRef;
use std::time::Duration;

/// Override the container hover input used by a reveal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevealForce {
    /// Treat the container as hovered.
    Active,
    /// Treat the container as not hovered.
    Inactive,
}

/// Override visibility while preserving focus in the revealed content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevealVisibility {
    /// Keep content visible.
    Shown,
    /// Hide content unless it contains keyboard focus.
    Hidden,
}

/// Fixed reveal behavior sampled when the reveal is bound.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
// Immutable reveal construction policy: independent inversion, layout retention, caller overrides and delay; these fields describe no current hover or visibility phase.
pub struct RevealContent {
    // xtask-allow(state-fields): independent: immutable hover-inversion policy, not a current hover flag.
    /// Reveal away from hover instead of on hover; focus and touch still reveal.
    pub inverted: bool,
    // xtask-allow(state-fields): independent: immutable layout-retention policy, not current visibility.
    /// Keep hidden content in normal layout instead of removing its space.
    pub preserve_layout: bool,
    // xtask-allow(state-fields): option: caller visibility policy override, not a stored lifecycle.
    /// Optional visibility override; focused content remains visible.
    pub force: Option<RevealVisibility>,
    // xtask-allow(state-fields): option: independent container hover-input override, not current hover state.
    /// Optional override of the container hover input.
    pub container_force: Option<RevealForce>,
    /// Delay before hover reveals content, in milliseconds; defaults to zero.
    pub hover_delay_ms: f64,
}

/// Mark a base style as a reveal group.
pub fn reveal_container(base: Style) -> Style {
    base.group()
}

/// Resolve a reveal against measured input state. Bind the refs to the
/// enclosing container and to the individual revealed node.
pub fn reveal_content(
    container: NodeRef,
    content: NodeRef,
    options: RevealContent,
) -> impl Fn() -> Style {
    let hovered = container.hovered();
    let group_focused = container.focus_within();
    let content_focused = content.focus_within();
    let touch = kivact::platform::capabilities().get().touch;
    let hover_delay = if options.hover_delay_ms.is_finite() {
        Duration::from_millis(options.hover_delay_ms.max(0.0) as u64)
    } else {
        Duration::ZERO
    };
    let hover = use_delayed(
        hovered,
        Delays {
            open: hover_delay,
            close: Duration::ZERO,
        },
    );
    let visible = memo(move || {
        let hover = match options.container_force {
            Some(RevealForce::Active) => true,
            Some(RevealForce::Inactive) => false,
            None => hover.is_open().get(),
        };
        let focused = group_focused.get();
        match options.force {
            Some(RevealVisibility::Shown) => true,
            Some(RevealVisibility::Hidden) => content_focused.get(),
            None if options.inverted => focused || touch || !hover,
            None => focused || touch || hover,
        }
    });
    let fade_out = if kivact::platform::is_motion_reduced().get() {
        0
    } else {
        kivact::style::use_tokens::<crate::Tokens>().motion.fast
    };
    let in_flow = use_delayed(
        visible,
        Delays {
            open: Duration::ZERO,
            close: Duration::from_millis(fade_out),
        },
    );
    move || {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        let base = style()
            .opacity(if visible.get() { 1.0 } else { 0.0 })
            .transition(
                TransitionProp::All,
                Transition::tween(t.motion.fast as f64, t.motion.ease_standard.clone()),
            );
        if !visible.get()
            && !in_flow.is_open().get()
            && !options.inverted
            && !options.preserve_layout
        {
            base.position(Position::Absolute)
        } else {
            base
        }
    }
}
