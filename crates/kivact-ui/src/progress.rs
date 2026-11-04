//! Role: Build the themed progress bar from its value and the motion it shows.
//!
//! The bar is a `Role::ProgressBar` whose track is the root and whose fill is
//! its one child. Both recipes get one [`ProgressState`]: the value, the
//! fraction shown and a repeating clock for a paint that moves. The theme
//! sets their timing with [`ProgressMotion`]. The clock only runs in an active
//! window while the bar is indeterminate or [`Progress::moving`] is true.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::values::{ControlSize, ProgressState};
use super::variants::{Variant, Variants};
use kivact_core::custom::NodeExt;
use kivact_core::motion::{Animated, Easing, FrameRate, Transition, animated, frame_loop};
use kivact_core::platform::{reduced_motion, surface_active};
use kivact_core::state::IntoValue;
use kivact_core::state::{Value, effect, memo, signal};
use kivact_core::style::{Theme, ThemeExtension, current_theme};
use kivact_core::view::{IntoView, Role, View};
use kivact_core::view::{Node, NodeBuilder, node};

/// Timing of standard progress bars. Install it with [`Theme::with`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressMotion {
    /// Whether the first determinate fill animates from zero.
    pub animate_initial: bool,
    /// Milliseconds for changes to the determinate fill.
    pub determinate_ms: u64,
    /// Easing of determinate fill changes.
    pub determinate_easing: Easing,
    /// Milliseconds per full cycle of the moving-fill clock.
    pub period_ms: u64,
    /// Optional full clock period in reduced motion. None leaves it still.
    pub reduced_period_ms: Option<u64>,
}

impl Default for ProgressMotion {
    fn default() -> Self {
        Self {
            animate_initial: true,
            determinate_ms: 400,
            determinate_easing: Easing::EaseOut,
            period_ms: 1300,
            reduced_period_ms: None,
        }
    }
}

impl ThemeExtension for ProgressMotion {
    fn fallback(_: &Theme) -> Self {
        Self::default()
    }
}

/// A bar of the share done, created by [`progress`].
pub struct Progress {
    root: Node,
    value: Value<Option<f64>>,
    fixed_indeterminate: bool,
    size: ControlSize,
    moving: Value<bool>,
    variants: Variants,
}

/// A progress bar at `value`, a fraction from 0 to 1 (a number outside it is
/// clamped); `None` is an indeterminate bar, which reports no value to
/// assistive technology and shows motion.
///
/// The node is a `Role::ProgressBar` with the range 0 to 1, the current value
/// and the text "n%". It is not named, because only the caller knows what it
/// measures: name it with `.name(..)` and draw any visible label yourself.
/// The look is the theme's [`UiRecipes::progress`](super::UiRecipes), whose
/// track is the bar itself, 100 % wide, and whose fill is its child.
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
#[track_caller]
pub fn progress(value: impl IntoValue<Option<f64>>) -> Progress {
    let prop = value.into_prop();
    let fixed_indeterminate = matches!(prop.fixed(), Some(None));
    let value = IntoValue::into_value(prop);
    Progress {
        root: node().role(Role::ProgressBar),
        value,
        fixed_indeterminate,
        size: super::values::default_control_size(),
        moving: false.into_value(),
        variants: Variants::default(),
    }
}

impl Progress {
    /// The size (default [`ControlSize::Medium`]), given to the recipes in
    /// their state; the recipes size the bar by it.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Run the clock the recipes read as [`ProgressState::phase`] also while
    /// the bar is determinate (constant or reactive), for a look whose fill
    /// moves, such as stripes.
    #[track_caller]
    pub fn moving(mut self, moving: impl IntoValue<bool>) -> Self {
        self.moving = moving.into_value();
        self
    }

    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}

impl NodeBuilder for Progress {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
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

impl IntoView for Progress {
    fn into_view(self) -> View {
        let Progress {
            mut root,
            value,
            fixed_indeterminate,
            size,
            moving,
            variants,
        } = self;
        let target = memo(move || fraction(value.get()));
        let motion = current_theme().get::<ProgressMotion>();
        let transition = Transition::tween(motion.determinate_ms as f64, motion.determinate_easing);
        let fill = move || target.get().unwrap_or(0.0);
        let shown = if motion.animate_initial {
            Animated::from(0.0, fill, transition)
        } else {
            animated(fill, transition)
        }
        .into_value();
        let phase = signal(0.0f64);
        let period = memo(|| {
            let motion = current_theme().get::<ProgressMotion>();
            if reduced_motion() {
                motion.reduced_period_ms
            } else {
                Some(motion.period_ms)
            }
        });
        let clock = frame_loop(move |frame| {
            if let Some(period) = period.get() {
                let period = period.max(1) as f64;
                let elapsed = frame.now.as_secs_f64() * 1000.0;
                phase.set(elapsed.rem_euclid(period) / period);
            }
        });
        effect(move || {
            if (target.get().is_none() || moving.get())
                && period.get().is_some()
                && surface_active()
            {
                clock.start(FrameRate::Hz(30.0));
            } else {
                clock.stop();
                phase.set(0.0);
            }
        });
        let phase = phase.into_value();
        let state = memo(move || ProgressState {
            value: target.get(),
            size,
            shown,
            phase,
        });
        if !fixed_indeterminate {
            root = root
                .value_min(0.0)
                .value_max(1.0)
                .value_now(move || target.get().unwrap_or(0.0))
                .value_text(move || match target.get() {
                    Some(v) => format!("{:.0}%", v * 100.0),
                    None => String::new(),
                });
        }
        let fill = with_recipe_style(
            node(),
            |recipes| recipes.progress_recipes().fill_recipe(),
            &variants,
            move || state.get(),
        );
        with_base_recipe_style(
            root,
            |recipes| recipes.progress_recipes().track_recipe(),
            &variants,
            move || state.get(),
        )
        .child(fill)
        .into_view()
    }
}
