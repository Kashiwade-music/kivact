//! Role: The interaction policy of Astryx Neutral: timings, gestures and motion of the standard interactions (crate-private).
use std::time::Duration;

use crate::motion::{EASE_STANDARD, FAST};
use kivact::geometry::Edge;
use kivact::interaction::{
    ButtonPolicy, Delays, DisabledButtonFocus, DisabledMenuFocus, InteractionPolicy, MenuPolicy,
    SliderPolicy, SliderTouchMotion, SubmenuTiming, SwitchPolicy, ToastPolicy, ToastSwipe,
    TouchSliderGesture,
};
use kivact::motion::{Spring, Transition};

/// The interaction policy of the kit.
pub(crate) fn policy() -> InteractionPolicy {
    InteractionPolicy {
        hover_card: Delays {
            open: Duration::from_millis(300),
            close: Duration::from_millis(200),
        },
        menu: MenuPolicy {
            press: true,
            disabled_focus: DisabledMenuFocus::Retain,
            submenu: SubmenuTiming {
                open: Duration::from_millis(150),
                close: Duration::from_millis(200),
            },
            ..Default::default()
        },
        button: ButtonPolicy {
            disabled_focus: DisabledButtonFocus::Retain,
        },
        slider: SliderPolicy {
            touch: SliderTouchMotion::Direct {
                gesture: TouchSliderGesture {
                    slop_px: 10.0,
                    projection: Duration::from_micros(99_600),
                    response: Duration::from_millis(400),
                },
                lifted_scale: 1.55,
                lift: Spring {
                    stiffness: 854.0,
                    damping: 35.07,
                    mass: 1.0,
                    rest: 1e-3,
                },
                drop: Spring {
                    stiffness: 157.9,
                    damping: 17.59,
                    mass: 1.0,
                    rest: 1e-3,
                },
            },
        },
        switch: SwitchPolicy { drag: true },
        toast: ToastPolicy {
            swipe: ToastSwipe::Enabled {
                edge: Edge::Bottom,
                transition: Transition::tween(FAST as f64, EASE_STANDARD),
            },
            ..Default::default()
        },
        ..Default::default()
    }
}
