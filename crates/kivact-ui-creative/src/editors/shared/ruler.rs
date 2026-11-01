//! Role: Time ruler of the Timeline, dope-sheet and graph editors.
//!
//! The time ruler of the Timeline, dope-sheet and graph editors: major and
//! minor ticks, their labels, and the bounds that the zoom and a clip's
//! length keep.

pub(crate) mod data;
use crate::editors::shared::ruler::data::Ruler;
use crate::editors::shared::text::label;
use kivact::draw::{Draw, Point, Rect, Size, TextStyle};
use kivact::style::TextAlign;
use kivact::view::callback;

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
        let width = (d.measure_text(&text, &style).width + 8.0).max(14.0);
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
