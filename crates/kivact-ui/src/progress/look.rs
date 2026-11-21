//! Role: The progress recipe state, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::Look;
use crate::recipe::recipe_table;
use kivact_core::draw::{LineCap, Path, Stroke};
use kivact_core::geometry::{Point, Rect};
use kivact_core::motion::Easing;
use kivact_core::style::{Align, Justify, Position, Style, Theme, pct, style};
use kivact_core::view::Variant;
use kivact_macros::Prop;

/// Timing of standard progress bars. Configure it through [`ProgressTheme`](crate::theme::ProgressTheme).
#[derive(Clone, Debug, PartialEq, Prop)]
pub struct ProgressMotion {
    /// Whether the first value animates from empty.
    pub animate_initial: bool,
    /// The milliseconds a determinate change takes.
    pub determinate_ms: u64,
    /// The easing of a determinate change.
    pub determinate_easing: Easing,
    /// The milliseconds per revolution of an indeterminate bar.
    pub period_ms: u64,
    /// The indeterminate period under reduced motion; `None` stops the bar.
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

pub(crate) fn ring(s: &Look<ProgressState>) -> bool {
    s.variant::<ProgressVariant>() == Some(ProgressVariant::Ring)
}

pub(crate) fn gauge(s: &Look<ProgressState>) -> bool {
    s.variant::<ProgressVariant>() == Some(ProgressVariant::Gauge)
}

pub(crate) fn progress_track(_: &Theme, s: &Look<ProgressState>) -> Style {
    if ring(s) {
        style()
            .size(metric_height(s) * 12.0, metric_height(s) * 12.0)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .paint(|d, size| {
                let radius = size.width.min(size.height) * 0.4;
                d.stroke_path(
                    &Path::new().arc(size.to_rect().center(), radius, 0.0, 359.99, false),
                    &Stroke::new(radius / 5.0),
                    kivact_core::style::color::BORDER,
                );
            })
    } else if gauge(s) {
        style()
            .width(pct(100))
            .height(metric_height(s) * 3.0)
            .min_width(0)
            .paint(|d, size| {
                for i in 0..20 {
                    d.fill_rect(
                        Rect::new(
                            i as f64 * size.width / 20.0,
                            0.0,
                            size.width / 20.0 * 0.75,
                            size.height,
                        ),
                        kivact_core::style::color::BORDER,
                    );
                }
            })
    } else {
        style()
            .width(pct(100))
            .height(metric_height(s))
            .min_width(0)
            .border_radius(metric_height(s) / 2.0)
            .background(kivact_core::style::color::BORDER)
            .clip(true)
    }
}

pub(crate) fn progress_fill(_: &Theme, s: &Look<ProgressState>) -> Style {
    let state = **s;
    let is_ring = ring(s);
    let is_gauge = gauge(s);
    style()
        .position(Position::Absolute)
        .inset(0)
        .paint(move |d, size| {
            let (start, length) = if state.value.is_some() {
                (0.0, state.shown.get().clamp(0.0, 1.0))
            } else {
                state.sweep()
            };
            if is_ring {
                let radius = size.width.min(size.height) * 0.4;
                let center = Point::new(size.width / 2.0, size.height / 2.0);
                if length > 0.0 {
                    d.stroke_path(
                        &Path::new().arc(
                            center,
                            radius,
                            -90.0 + start * 360.0,
                            -90.0 + (start + length.min(0.9999)) * 360.0,
                            false,
                        ),
                        &Stroke::new(radius / 5.0).cap(LineCap::Round),
                        kivact_core::style::color::ACCENT,
                    );
                }
            } else if is_gauge {
                for i in 0..20 {
                    let fraction = (i as f64 + 0.5) / 20.0;
                    if fraction >= start && fraction <= start + length {
                        d.fill_rect(
                            Rect::new(
                                i as f64 * size.width / 20.0,
                                0.0,
                                size.width / 20.0 * 0.75,
                                size.height,
                            ),
                            kivact_core::style::color::ACCENT,
                        );
                    }
                }
            } else {
                d.rounded_rect(
                    Rect::new(start * size.width, 0.0, length * size.width, size.height),
                    size.height / 2.0,
                    kivact_core::style::color::ACCENT,
                );
            }
        })
}

pub(crate) fn progress_label(t: &Theme, _: &Look<ProgressState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .font_weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
        .color(kivact_core::style::color::TEXT)
}

/// The state of a [`progress`](super::progress) bar for the recipes of its
/// track and fill.
#[derive(Clone, Copy, Debug)]
pub struct ProgressState {
    /// The completed fraction from 0 to 1 (a value outside it is clamped),
    /// or `None` while the bar is indeterminate.
    pub value: Option<f64>,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    /// The fraction the bar is shown at, from 0 to 1: it moves toward
    /// `value` over a short time (at once under reduced motion), starting
    /// from 0 by default when the bar appears; 0 while indeterminate. A recipe that
    /// paints the fill reads it in the paint, which follows it. The handle
    /// is made at setup and stays the same for the mounted bar; it is not
    /// part of the state's equality.
    pub shown: kivact_core::state::Value<f64>,
    /// A clock from 0 to 1 that repeats with the theme's
    /// [`ProgressMotion`](super::ProgressMotion) period while the bar is
    /// indeterminate or [`Progress::moving`](super::Progress::moving) is true.
    /// The clock rests at 0 under reduced motion by default. A recipe reads it
    /// in a paint that moves. The handle is made at setup and stays the same
    /// for the mounted bar; it is not part of the state's equality.
    pub phase: kivact_core::state::Value<f64>,
}

impl ProgressState {
    /// Where an indeterminate bar's moving segment is, as `(start, length)`
    /// in fractions of the track: it enters from the left, crosses and
    /// leaves at the right (the start is below 0 or above 1 while it is partly
    /// or wholly outside). It follows [`phase`](Self::phase), so a paint that
    /// calls it moves; under reduced motion it rests at the track's start.
    pub fn sweep(&self) -> (f64, f64) {
        const LENGTH: f64 = 0.3;
        if kivact_core::platform::is_motion_reduced().get() {
            return (0.0, LENGTH);
        }
        let eased = (1.0 - (self.phase.get() * std::f64::consts::PI).cos()) / 2.0;
        (LENGTH * (-1.1 + 4.7 * eased), LENGTH)
    }
}

impl PartialEq for ProgressState {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.size == other.size
    }
}

impl kivact_core::view::Variant for ProgressMotion {}

recipe_table! {
    /// The recipes of a [`progress`](super::progress) bar.
    ProgressTheme {
        /// The bar itself: the track the fill moves in, which clips it.
        track(track_recipe): ProgressState = progress_track,
        /// The fill, the track's one child: the whole track, painted or
        /// sized to the value.
        fill(fill_recipe): ProgressState = progress_fill,
        /// Percentage text in a ring.
        label(label_recipe): ProgressState = progress_label,
    }
    settings {
        /// The motion of the bar; see [`ProgressMotion`].
        motion(motion_value): ProgressMotion = ProgressMotion::default(),
    }
}

/// Standard progress appearances, read through the existing variant contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum ProgressVariant {
    /// A horizontal bar (the default).
    #[default]
    Bar,
    /// A circular track with visible percentage text.
    Ring,
    /// A segmented horizontal gauge.
    Gauge,
}

impl Variant for ProgressVariant {}

/// The thickness of a bar at a size.
fn metric_height(s: &ProgressState) -> f64 {
    match s.size {
        crate::ControlSize::ExtraSmall => 3.0,
        crate::ControlSize::Small => 4.0,
        crate::ControlSize::Medium => 6.0,
        crate::ControlSize::Large => 10.0,
    }
}
