//! Role: Shared logical-time ruler configuration.
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
