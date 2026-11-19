//! Role: The progress recipe state, tokens, recipe table and default looks.
use crate::look_common::metric_height;

use crate::control_size::ControlSize;
use crate::progress::ProgressVariant;
use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::draw::{LineCap, Path, Stroke};
use kivact_core::geometry::{Point, Rect};
use kivact_core::motion::Easing;
use kivact_core::style::{Align, ColorToken, Justify, Position, Style, Theme, pct, style};
pub(crate) fn ring(s: &Look<ProgressState>) -> bool {
    s.variant::<ProgressVariant>() == Some(&ProgressVariant::Ring)
}

pub(crate) fn gauge(s: &Look<ProgressState>) -> bool {
    s.variant::<ProgressVariant>() == Some(&ProgressVariant::Gauge)
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
                    ColorToken::Border,
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
                        ColorToken::Border,
                    );
                }
            })
    } else {
        style()
            .width(pct(100))
            .height(metric_height(s))
            .min_width(0)
            .radius(metric_height(s) / 2.0)
            .background(ColorToken::Border)
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
                        ColorToken::Accent,
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
                            ColorToken::Accent,
                        );
                    }
                }
            } else {
                d.rounded_rect(
                    Rect::new(start * size.width, 0.0, length * size.width, size.height),
                    size.height / 2.0,
                    ColorToken::Accent,
                );
            }
        })
}

pub(crate) fn progress_label(t: &Theme, _: &Look<ProgressState>) -> Style {
    style()
        .font_size(t.text().size_small)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
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

/// Timing of standard progress bars. Configure it through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
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

impl crate::Variant for ProgressMotion {}

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

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ProgressTokens {
    pub motion: ProgressMotion,
}

recipe_table! {
    /// The recipes of a [`progress`](super::progress) bar.
    ProgressRecipes {
        /// The bar itself: the track the fill moves in, which clips it.
        track(track_recipe): ProgressState = progress_track,
        /// The fill, the track's one child: the whole track, painted or
        /// sized to the value.
        fill(fill_recipe): ProgressState = progress_fill,
        /// Percentage text in a ring.
        label(label_recipe): ProgressState = progress_label,
    }
}
