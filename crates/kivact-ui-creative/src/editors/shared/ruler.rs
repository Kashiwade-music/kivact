//! Role: Time ruler of the Timeline, dope-sheet and graph editors.
//!
//! The time ruler of the Timeline, dope-sheet and graph editors: major and
//! minor ticks, their labels, and the bounds that the zoom and a clip's
//! length keep.

use crate::editors::shared::text::label;
use kivact::draw::{Draw, Point, Rect, Size, TextStyle};
use kivact::style::TextAlign;
use kivact::view::callback;
use std::rc::Rc;

/// How the editor's time ruler spaces its ticks ([`Timeline::ruler_ticks`](crate::Timeline::ruler_ticks)).
///
/// The intervals are in logical editor units, the unit of [`Clip`](crate::Clip)
/// positions and of the playhead. At each zoom the ruler uses the smallest
/// `major` interval whose ticks are at least [`Self::min_spacing`] pixels
/// apart. If even the largest candidate is too dense it is multiplied by 10,
/// 100, and so on until it fits. Each major tick gets a grid line and a
/// label; `minor` short marks sit between two major ticks.
#[derive(Clone, Debug, PartialEq)]
pub struct RulerTicks {
    pub(crate) major: Vec<f64>,
    pub(crate) minor: u32,
    pub(crate) min_spacing: f32,
}

impl RulerTicks {
    /// Candidate major intervals, in any order, with `minor` marks between
    /// two major ticks. Intervals that are not finite and positive are
    /// ignored; with none left the ruler uses the sequence 1, 2, 5, 10, 20,
    /// 50, 100 and so on.
    ///
    /// ```
    /// use kivact_ui_creative as ui;
    /// // Frames at 30 per second: a label every 1, 2, 5, 10, 30 or 60 seconds.
    /// let ticks = ui::RulerTicks::new([30.0, 60.0, 150.0, 300.0, 900.0, 1800.0], 4);
    /// # let _ = ticks;
    /// ```
    pub fn new(major: impl IntoIterator<Item = f64>, minor: u32) -> Self {
        let mut major: Vec<f64> = major
            .into_iter()
            .filter(|interval| interval.is_finite() && *interval > 0.0)
            .collect();
        major.sort_by(f64::total_cmp);
        major.dedup();
        Self {
            major,
            minor,
            min_spacing: 56.0,
        }
    }

    /// The smallest distance between two major ticks in logical pixels
    /// (default 56, at least 8). A value that is not finite keeps the
    /// current setting.
    pub fn min_spacing(mut self, px: f32) -> Self {
        if px.is_finite() {
            self.min_spacing = px.max(8.0);
        }
        self
    }

    /// The major interval in logical units when one unit spans `scale` pixels.
    pub(crate) fn interval(&self, scale: f64) -> f64 {
        let fits = |interval: f64| interval * scale >= f64::from(self.min_spacing);
        let Some(&last) = self.major.last() else {
            let mut decade = 1.0;
            loop {
                if let Some(interval) = [1.0, 2.0, 5.0]
                    .into_iter()
                    .map(|m| m * decade)
                    .find(|interval| fits(*interval))
                {
                    return interval;
                }
                decade *= 10.0;
            }
        };
        if let Some(interval) = self.major.iter().copied().find(|i| fits(*i)) {
            return interval;
        }
        let mut interval = last;
        while !fits(interval) {
            interval *= 10.0;
        }
        interval
    }
}

impl Default for RulerTicks {
    fn default() -> Self {
        Self::new([1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0], 4)
    }
}

/// The ruler settings the builder collects.
pub(crate) struct Ruler {
    pub(crate) format: Rc<dyn Fn(f64) -> String>,
    pub(crate) ticks: RulerTicks,
    pub(crate) zoom: (f64, f64),
    pub(crate) min_duration: f64,
    /// Arguments the builder refused, reported when the editor is mounted.
    pub(crate) usage: Vec<String>,
}

impl Default for Ruler {
    fn default() -> Self {
        Self {
            format: Rc::new(|time| format!("{}", time.trunc() as i64)),
            ticks: RulerTicks::default(),
            zoom: (0.25, 4.0),
            min_duration: 1.0,
            usage: Vec::new(),
        }
    }
}

impl Ruler {
    /// Report the arguments the builder refused. Setup only.
    pub(crate) fn report(&self) {
        let report = callback(|error| -> kivact::Result<()> { Err(error) });
        for message in &self.usage {
            report.call(kivact::UiError::msg(message.clone()));
        }
    }

    /// The text for a logical time: the consumer's `time_format`.
    pub(crate) fn label(&self, time: f64) -> String {
        (self.format)(time)
    }

    /// The playhead's badge centered on `x`, wide enough for its label.
    pub(crate) fn badge(&self, d: &mut Draw, x: f32, time: f64) {
        let p = d.theme().get::<crate::tokens::Tokens>();
        let text = self.label(time);
        let style = TextStyle {
            size: 9.0,
            color: p.canvas,
            align: TextAlign::Center,
            ..TextStyle::default()
        };
        let width = (d.measure_text(&text, &style).size.width + 8.0).max(14.0);
        d.rounded_rect(
            Rect::new(x - width / 2.0, 0.0, width, 17.0),
            6.0,
            p.text_high,
        );
        d.text(&text, Point::new(x, 3.0), style);
    }

    /// Grid lines, ticks and labels of the area right of `left`, below the
    /// ruler band `0..top`.
    pub(crate) fn draw(&self, d: &mut Draw, size: Size, left: f32, z: f32, pan: Point, top: f32) {
        let p = d.theme().get::<crate::tokens::Tokens>();
        let interval = self.ticks.interval(f64::from(z));
        let step = (interval * f64::from(z)) as f32;
        if !step.is_finite() || step <= 0.0 {
            return;
        }
        let origin = left + pan.x;
        let first = ((left - origin) / step).ceil() as i64;
        let last = ((size.width - origin) / step).ceil() as i64;
        let minors = self.ticks.minor;
        let minor_step = step / (minors + 1) as f32;
        for i in first..=last {
            let x = origin + (i as f64 * interval) as f32 * z;
            d.line(Point::new(x, top), Point::new(x, size.height), 1.0, p.hair);
            label(
                d,
                &self.label(i as f64 * interval),
                x + 3.0,
                5.0,
                9.0,
                p.dim,
            );
            if minor_step >= 3.0 {
                for k in 1..=i64::from(minors) {
                    let mx = x + k as f32 * minor_step;
                    if mx < left || mx > size.width {
                        continue;
                    }
                    d.line(
                        Point::new(mx, top - 5.0),
                        Point::new(mx, top),
                        1.0,
                        p.hair_strong,
                    );
                }
            }
        }
    }
}
