//! Role: Theme-wide interaction policy and its per-control options.
use crate::TouchSliderGesture;
use crate::menu_policy::{DisabledMenuFocus, MenuPress, SubmenuTiming};
use crate::tooltip_policy::TooltipTiming;
use kivact_core::geometry::Edge;
use kivact_core::motion::{Spring, Transition};
use kivact_core::style::{Theme, ThemeExtension};

/// Whether a theme keeps disabled buttons in keyboard focus order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisabledButtonFocus {
    #[default]
    Skip,
    Retain,
}

/// Optional touch behavior and thumb motion installed by a slider theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SliderTouchMotion {
    /// Keep standard mouse-style pointer tracking for all pointer kinds.
    #[default]
    Standard,
    /// Use touch-specific relative tracking and release motion.
    Direct {
        gesture: TouchSliderGesture,
        lifted_scale: f64,
        lift: Spring,
        drop: Spring,
    },
}

/// Whether switches in this theme use direct manipulation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SwitchDrag(pub bool);

/// Theme opt-in for touch and pen dismissal of standard toast cards.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ToastSwipe {
    #[default]
    Disabled,
    Enabled {
        edge: Edge,
        transition: Transition,
    },
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct InteractionPolicy {
    pub menu: MenuPolicy,
    pub tooltip: TooltipTiming,
    pub button: ButtonPolicy,
    pub slider: SliderPolicy,
    pub switch: SwitchPolicy,
    pub toast: ToastPolicy,
    pub select: SelectPolicy,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MenuPolicy {
    pub press: MenuPress,
    pub drift: PressDrift,
    pub submenu: SubmenuTiming,
    pub disabled_focus: DisabledMenuFocus,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ButtonPolicy {
    pub disabled_focus: DisabledButtonFocus,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SliderPolicy {
    pub touch: SliderTouchMotion,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SwitchPolicy {
    pub drag: SwitchDrag,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PressDrift {
    pub context: f64,
    pub other: f64,
}
impl Default for PressDrift {
    fn default() -> Self {
        Self {
            context: 10.0,
            other: 13.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastPolicy {
    pub swipe: ToastSwipe,
    pub swipe_slop: f64,
}
impl Default for ToastPolicy {
    fn default() -> Self {
        Self {
            swipe: ToastSwipe::default(),
            swipe_slop: 8.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectPolicy {
    pub sheet_slop: f64,
}
impl Default for SelectPolicy {
    fn default() -> Self {
        Self { sheet_slop: 10.0 }
    }
}

impl ThemeExtension for InteractionPolicy {
    fn fallback(_: &Theme) -> Self {
        Self::default()
    }
}
