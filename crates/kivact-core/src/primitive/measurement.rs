//! Role: The measurement contract of a primitive: constraints, sizes and baselines.

use kivact_style::draw::{Size, TextMetrics, TextParagraph, TextRun, TextStyle};
use kivact_wire::{self as wire};

/// Space available to a primitive along one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvailableSpace {
    /// A definite number of logical pixels.
    Definite(f32),
    /// As little as the content allows (the primitive's minimum size).
    MinContent,
    /// As much as the content wants (the primitive's preferred size).
    MaxContent,
}

/// What a primitive is measured under, in logical pixels. A dimension the
/// layout already fixed (`known_*`, from the style or a stretch) wins
/// over the measured one. It lends the layout's text measurement
/// ([`Constraints::measure_text`]) for the call.
#[derive(Clone, Copy)]
pub struct Constraints<'a> {
    /// The width the layout already fixed.
    pub known_width: Option<f32>,
    /// The height the layout already fixed.
    pub known_height: Option<f32>,
    /// The width available.
    pub available_width: AvailableSpace,
    /// The height available.
    pub available_height: AvailableSpace,
    text: wire::LeafTextMeasurer<'a>,
}

impl Constraints<'_> {
    /// The widest the primitive can be: the known width, else a definite
    /// available width (`None`: unbounded, `Some(0.0)`: as narrow as it
    /// can).
    pub fn max_width(&self) -> Option<f32> {
        match (self.known_width, self.available_width) {
            (Some(w), _) | (None, AvailableSpace::Definite(w)) => Some(w),
            (None, AvailableSpace::MinContent) => Some(0.0),
            (None, AvailableSpace::MaxContent) => None,
        }
    }

    /// The extent `Draw::text` draws `text` with in `style` in this
    /// primitive (one line; a style without a family takes the
    /// primitive's inherited one): size a chip by its label. It shapes
    /// with the layout's own text engine, so it agrees with drawing. A
    /// new font registered later measures the primitive again.
    ///
    /// ```no_run
    /// # use kivact_core as kivact;
    /// use kivact::prelude::*;
    /// use kivact::draw::{Size};
    /// use kivact::draw::TextStyle;
    /// use kivact::primitive::{Constraints, Measurement, primitive};
    ///
    /// fn chip(label: Signal<String>) -> impl IntoView {
    ///     primitive().measure(label, |label: &String, c: Constraints| {
    ///         let m = c.measure_text(label, &TextStyle::default());
    ///         // The label sits 4 px below the top.
    ///         Measurement::new(Size::new(m.width + 16.0, m.height + 8.0)).baseline(m.baseline + 4.0)
    ///     })
    /// }
    /// # let _ = chip;
    /// ```
    pub fn measure_text(&self, text: &str, style: &TextStyle) -> TextMetrics {
        let origin = wire::Point { x: 0.0, y: 0.0 };
        let run = TextRun::from_style(text, origin, style, 1.0, wire::Rgba::TRANSPARENT, None);
        self.text.measure(run.as_wire())
    }

    /// `text` shaped in `style` and wrapped at `max_width` (`None`: at
    /// hard line breaks only) as a [`TextParagraph`]: its size, lines and
    /// carets for a caret, a hit test or a selection, in the primitive's
    /// own pixels. It is the paragraph [`Draw::layout_paragraph`](crate::draw::Draw::layout_paragraph) makes of
    /// the same inputs in this primitive's draw closure, shaped once by the
    /// layout's own text engine (a style without a family takes the
    /// primitive's inherited one). A new font registered later measures the
    /// primitive again.
    ///
    /// ```no_run
    /// # use kivact_core as kivact;
    /// use kivact::prelude::*;
    /// use kivact::draw::{Size, TextStyle};
    /// use kivact::primitive::{Constraints, Measurement, primitive};
    ///
    /// fn line(text: Signal<String>) -> impl IntoView {
    ///     primitive().measure(text, |text: &String, c: Constraints| {
    ///         let paragraph = c.measure_paragraph(text, &TextStyle::default(), c.max_width());
    ///         Measurement::new(paragraph.size()).baseline(paragraph.baseline())
    ///     })
    /// }
    /// # let _ = line;
    /// ```
    pub fn measure_paragraph(
        &self,
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> TextParagraph {
        let origin = wire::Point { x: 0.0, y: 0.0 };
        let run = TextRun::from_style(text, origin, style, 1.0, wire::Rgba::TRANSPARENT, None);
        let shaped = self
            .text
            .shape(run.as_wire(), max_width)
            .unwrap_or_default();
        TextParagraph::from_shaped(text, shaped, max_width)
    }
}

impl PartialEq for Constraints<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.known_width == other.known_width
            && self.known_height == other.known_height
            && self.available_width == other.available_width
            && self.available_height == other.available_height
    }
}

impl std::fmt::Debug for Constraints<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Constraints")
            .field("known_width", &self.known_width)
            .field("known_height", &self.known_height)
            .field("available_width", &self.available_width)
            .field("available_height", &self.available_height)
            .finish_non_exhaustive()
    }
}

pub(super) fn space(s: wire::AvailableSpace) -> AvailableSpace {
    match s {
        wire::AvailableSpace::Definite(v) => AvailableSpace::Definite(v),
        wire::AvailableSpace::MinContent => AvailableSpace::MinContent,
        wire::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
    }
}

pub(super) fn constraints(
    r: wire::MeasureRequest,
    text: wire::LeafTextMeasurer<'_>,
) -> Constraints<'_> {
    Constraints {
        known_width: r.known_width,
        known_height: r.known_height,
        available_width: space(r.available_width),
        available_height: space(r.available_height),
        text,
    }
}

/// **Experimental.** What [`Primitive::measure`](super::Primitive::measure)'s function returns: the
/// primitive's size and, optionally, where its first baseline lies. A
/// plain [`Size`] converts to a measurement without a baseline.
///
/// A flex container that aligns the primitive by baseline
/// (`align(Align::Baseline)`) lines this baseline up with its siblings'
/// (a text's, a text field's). Without one, the primitive's baseline is
/// the bottom of its box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    /// The primitive's size.
    pub size: Size,
    /// The first baseline, in logical pixels below the top of the
    /// primitive's content box (inside its padding and border); `None`:
    /// the bottom of the box.
    pub baseline: Option<f32>,
}

impl Measurement {
    /// A measurement of `size` with no baseline.
    pub fn new(size: Size) -> Self {
        Measurement {
            size,
            baseline: None,
        }
    }

    /// This measurement with its first baseline `y` below the top of the
    /// content box (for example a label's [`TextMetrics::baseline`] plus
    /// the padding above it).
    #[must_use]
    pub fn baseline(mut self, y: f32) -> Self {
        self.baseline = Some(y);
        self
    }
}

impl From<Size> for Measurement {
    fn from(size: Size) -> Self {
        Measurement::new(size)
    }
}

/// A size the layout can use: finite and not negative; a baseline it can
/// use: finite.
pub(super) fn sanitize(m: Measurement) -> wire::LeafSize {
    let clean = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
    wire::LeafSize {
        size: wire::Size {
            width: clean(m.size.width),
            height: clean(m.size.height),
        },
        baseline: m
            .baseline
            .filter(|y| y.is_finite())
            .map(wire::Baseline::FromTop),
    }
}
