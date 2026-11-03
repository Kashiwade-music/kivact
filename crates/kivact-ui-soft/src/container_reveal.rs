//! Role: Reveal in-place actions on hover and keyboard focus.
use crate::tokens;
use kivact::motion::Transition;
use kivact::style::{Position, Style, TransitionProp, style};
use kivact::view::NodeRef;

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
    pub hover_delay_ms: f32,
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
    move || {
        let t = tokens();
        let touch = kivact::platform::capabilities().touch;
        let hover = match options.container_force {
            Some(RevealForce::Active) => true,
            Some(RevealForce::Inactive) => false,
            None => hovered.get(),
        };
        let focused = group_focused.get();
        let visible = match options.force {
            Some(RevealVisibility::Shown) => true,
            Some(RevealVisibility::Hidden) => content_focused.get(),
            None if options.inverted => focused || touch || !hover,
            None => focused || touch || hover,
        };
        let delay = if hover && !focused && !touch && options.force.is_none() {
            options.hover_delay_ms.max(0.0)
        } else {
            0.0
        };
        let transition =
            Transition::tween(t.motion.fast as f64, t.motion.ease_standard).delay(delay);
        let base = style()
            .opacity(if visible { 1.0 } else { 0.0 })
            .transition(TransitionProp::Paint, transition);
        if !visible && !options.inverted && !options.preserve_layout {
            base.position(Position::Absolute)
        } else {
            base
        }
    }
}
