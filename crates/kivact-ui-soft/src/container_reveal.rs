//! Role: Reveal in-place actions on hover and keyboard focus.
use crate::tokens;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevealForce {
    Active,
    Inactive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevealVisibility {
    Shown,
    Hidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RevealContent {
    pub inverted: bool,
    pub preserve_layout: bool,
    pub force: Option<RevealVisibility>,
    pub container_force: Option<RevealForce>,
    pub hover_delay_ms: f64,
}

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
        tokens().motion.fast
    };
    let in_flow = use_delayed(
        visible,
        Delays {
            open: Duration::ZERO,
            close: Duration::from_millis(fade_out),
        },
    );
    move || {
        let t = tokens();
        let base = style()
            .opacity(if visible.get() { 1.0 } else { 0.0 })
            .transition(
                TransitionProp::Paint,
                Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
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
