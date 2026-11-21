//! Role: Build the themed progress bar from its value and the motion it shows.
//!
//! The bar is a `Role::ProgressBar` whose track is the root and whose fill is
//! its one child. Both recipes get one [`ProgressState`]: the value, the
//! fraction shown and a repeating clock for a paint that moves. The theme
//! sets their timing with [`ProgressMotion`]. The clock only runs in an active
//! window while the bar is indeterminate or its `moving` prop is true.

pub(crate) mod look;

pub use look::ProgressMotion;
pub use look::ProgressVariant;

use crate::control_size::ControlSize;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::Caller;
use kivact_core::motion::{Animated, FrameRate, Transition, animated, frame_loop};
use kivact_core::platform::{is_motion_reduced, is_surface_active};
use kivact_core::state::{IntoValue, Machine, Step, Value, machine, memo, signal};
use kivact_core::style::use_theme;
use kivact_core::view::{Role, node, text};
use kivact_macros::component;
use look::{ProgressState, ProgressTheme};

#[derive(Clone, Copy, PartialEq)]
enum ProgressMotionPhase {
    Stopped,
    Running,
}
struct ProgressMotionInput;
impl Machine for ProgressMotionInput {
    type State = ProgressMotionPhase;
    type Event = bool;
    type Effect = ProgressMotionPhase;
    fn transition(
        &self,
        state: &Self::State,
        running: bool,
    ) -> Step<Self::State, bool, Self::Effect> {
        let next = if running {
            ProgressMotionPhase::Running
        } else {
            ProgressMotionPhase::Stopped
        };
        if *state == next {
            Step::stay()
        } else {
            Step::to(next).with(next)
        }
    }
}

/// A bar of the share done: `value` is a fraction from 0 to 1 (a number
/// outside it is clamped); `None` is an indeterminate bar, which reports no
/// value to assistive technology and shows motion. The value may be reactive,
/// so a bar becomes indeterminate while its value is `None`.
///
/// The node is a `Role::ProgressBar` with the range 0 to 1, the current value
/// and the text "n%". It is not named, because only the caller knows what it
/// measures: name it with `.name(..)` and draw any visible label yourself.
/// The look is the theme's [`ProgressTheme`](crate::theme::ProgressTheme), whose
/// track is the bar itself, 100 % wide, and whose fill is its child.
///
/// `size` (default [`ControlSize::Medium`]) is given to the recipes in their
/// state; the recipes size the bar by it. `moving` runs the clock the recipes
/// read as [`ProgressState::phase`] also while the bar is determinate, for a
/// look whose fill moves, such as stripes.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::progress;
///
/// fn upload() -> impl IntoView {
///     let done = signal(0.66);
///     column((
///         progress(move || Some(done.get())).name("Upload"),
///         progress(None::<f64>).name("Preparing"),
///     ))
///     .style(style().gap(8).width(240))
/// }
/// ```
#[component]
pub fn progress(
    value: Value<Option<f64>>,
    #[prop(default = crate::control_size::default_control_size())] size: Value<ControlSize>,
    #[prop(default = false)] moving: Value<bool>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let mut root = node().role(Role::ProgressBar);
    let target = memo(move || fraction(value.get()));
    let motion_override = crate::recipe::Look::of((), variants.clone()).variant::<ProgressMotion>();
    let motion = motion_override.clone().unwrap_or_else(|| {
        use_theme()
            .get()
            .ext::<ProgressTheme>()
            .motion_value()
            .clone()
    });
    let transition = Transition::tween(
        motion.determinate_ms as f64,
        motion.determinate_easing.clone(),
    );
    let fill = move || target.get().unwrap_or(0.0);
    let shown = if motion.animate_initial {
        Animated::from(0.0, fill, transition)
    } else {
        animated(fill, transition)
    }
    .into_value();
    let phase = motion_clock(target.into_value(), moving, motion_override);
    let state = memo(move || ProgressState {
        value: target.get(),
        size: size.get(),
        shown,
        phase,
    });
    // A `None` value reports no range and no value: NaN is not a finite
    // semantic value, so the node states none.
    root = root
        .value_min(move || {
            if target.get().is_some() {
                0.0
            } else {
                f64::NAN
            }
        })
        .value_max(move || {
            if target.get().is_some() {
                1.0
            } else {
                f64::NAN
            }
        })
        .value_now(move || target.get().unwrap_or(f64::NAN))
        .value_text(move || match target.get() {
            Some(v) => format!("{:.0}%", v * 100.0),
            None => String::new(),
        });
    let fill = with_recipe_style(
        node(),
        |r: &ProgressTheme| r.fill_recipe(),
        &variants,
        move || state.get(),
    );
    let label = (crate::recipe::Look::of((), variants.clone()).variant::<ProgressVariant>()
        == Some(ProgressVariant::Ring))
    .then(|| {
        with_recipe_style(
            text(move || {
                target
                    .get()
                    .map_or_else(String::new, |v| format!("{:.0}%", v * 100.0))
            }),
            |r: &ProgressTheme| r.label_recipe(),
            &variants,
            move || state.get(),
        )
    });
    with_base_recipe_style(
        root,
        |r: &ProgressTheme| r.track_recipe(),
        &variants,
        move || state.get(),
    )
    .child((fill, label))
}

/// `value` as a fraction from 0 to 1.
fn fraction(value: Option<f64>) -> Option<f64> {
    value.map(|v| {
        if v.is_finite() {
            v.clamp(0.0, 1.0)
        } else {
            0.0
        }
    })
}

/// Drive indeterminate motion only while a visible surface needs it.
fn motion_clock(
    target: Value<Option<f64>>,
    moving: Value<bool>,
    motion_override: Option<ProgressMotion>,
) -> Value<f64> {
    let phase = signal(0.0f64);
    let period = memo(move || {
        let motion = motion_override.clone().unwrap_or_else(|| {
            use_theme()
                .get()
                .ext::<ProgressTheme>()
                .motion_value()
                .clone()
        });
        if is_motion_reduced().get() {
            motion.reduced_period_ms
        } else {
            Some(motion.period_ms)
        }
    });
    // Essential: under reduced motion the clock runs only with a reduced
    // period (`reduced_period_ms`); without one `period` is None and it stops.
    let clock = frame_loop(move |frame| {
        if let Some(period) = period.get() {
            let period = period.max(1) as f64;
            let elapsed = frame.now.as_secs_f64() * 1000.0;
            phase.set(elapsed.rem_euclid(period) / period);
        }
    })
    .essential();
    let motion_machine = machine(
        ProgressMotionPhase::Stopped,
        || ProgressMotionInput,
        move |_, phase_state| match phase_state {
            ProgressMotionPhase::Running => clock.start(FrameRate::Hz(30.0)),
            ProgressMotionPhase::Stopped => {
                clock.stop();
                phase.set(0.0);
            }
        },
    );
    let running = memo(move || {
        (target.get().is_none() || moving.get())
            && period.get().is_some()
            && is_surface_active().get()
    });
    motion_machine.send(running.get_untracked());
    motion_machine.feed(running, |running| running);
    phase.into_value()
}
