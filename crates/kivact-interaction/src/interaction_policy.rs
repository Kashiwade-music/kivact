//! Role: Theme-wide interaction policy and its per-control options.
use crate::TouchSliderGesture;
use crate::menu_policy::{DisabledMenuFocus, MenuPress, SubmenuTiming};
use crate::tooltip_policy::TooltipTiming;
use kivact_core::geometry::Edge;
use kivact_core::motion::{Spring, Transition};
use kivact_core::style::{Theme, ThemeExtension};
use kivact_core::view::Modifiers;

/// A modifier key a behavior reads from [`Modifiers`](kivact_core::view::Modifiers).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModifierKey {
    Shift,
    Ctrl,
    Alt,
    /// The platform command key (`Modifiers::meta`).
    Meta,
}

impl ModifierKey {
    /// Whether this key is held in `modifiers`.
    pub fn is_held(self, modifiers: Modifiers) -> bool {
        match self {
            Self::Shift => modifiers.shift,
            Self::Ctrl => modifiers.ctrl,
            Self::Alt => modifiers.alt,
            Self::Meta => modifiers.meta,
        }
    }
}

/// How modifier keys scale a value control's scrub, arrow keys and wheel, and
/// which click resets it. Read by the Creative kit's numeric controls.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrubPolicy {
    // xtask-allow(state-fields): option: absent means the policy has no fine modifier
    /// Held for a finer scrub, arrow key or wheel notch (default Ctrl).
    pub fine: Option<ModifierKey>,
    /// What the fine key multiplies the change by (default 0.1).
    pub fine_factor: f64,
    // xtask-allow(state-fields): option: absent means the policy has no coarse modifier
    /// Held for a coarser scrub or arrow key (default Shift). The wheel ignores
    /// it: Shift turns a wheel sideways on some platforms.
    pub coarse: Option<ModifierKey>,
    /// What the coarse key multiplies the change by (default 10.0).
    pub coarse_factor: f64,
    // xtask-allow(state-fields): option: absent means no click resets the value
    /// Held with a primary click to reset the value, as a double click does
    /// (default none).
    pub reset_click: Option<ModifierKey>,
    /// Scroll distance in logical pixels that makes one wheel step (default 60:
    /// one notch at three lines per notch). Smaller distances, such as a
    /// touchpad's, add up.
    pub wheel_notch: f64,
}

impl Default for ScrubPolicy {
    fn default() -> Self {
        Self {
            fine: Some(ModifierKey::Ctrl),
            fine_factor: 0.1,
            coarse: Some(ModifierKey::Shift),
            coarse_factor: 10.0,
            reset_click: None,
            wheel_notch: 60.0,
        }
    }
}

impl ScrubPolicy {
    /// The factor `modifiers` select: the fine factor when the fine key is
    /// held, else the coarse one when that is held, else 1. A factor that is
    /// not positive and finite acts as 1.
    pub fn factor(&self, modifiers: Modifiers) -> f64 {
        let selected = if self.fine.is_some_and(|key| key.is_held(modifiers)) {
            Some(self.fine_factor)
        } else if self.coarse.is_some_and(|key| key.is_held(modifiers)) {
            Some(self.coarse_factor)
        } else {
            None
        };
        match selected {
            Some(factor) if factor.is_finite() && factor > 0.0 => factor,
            _ => 1.0,
        }
    }
}

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
    pub hover_card: crate::HoverCardTiming,
    pub drawer: crate::DrawerPolicy,
    pub button: ButtonPolicy,
    pub slider: SliderPolicy,
    pub switch: SwitchPolicy,
    pub toast: ToastPolicy,
    pub select: SelectPolicy,
    /// Modifier keys of value scrubbing (Creative's number, value_slider, knob
    /// and fader).
    pub scrub: ScrubPolicy,
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
