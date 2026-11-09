//! Role: The measurement contract of a primitive: constraints, sizes and baselines.

use kivact_style::draw::{Size, TextFormat, TextMetrics, TextParagraph};
use kivact_types::{self as values};

/// Space available to a primitive along one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvailableSpace {
    /// A definite number of logical pixels.
    Definite(f64),
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
    // xtask-allow(state-fields): borrowed input snapshot - The width the layout already fixed
    /// The width the layout already fixed.
    pub known_width: Option<f64>,
    // xtask-allow(state-fields): borrowed input snapshot - The height the layout already fixed
    /// The height the layout already fixed.
    pub known_height: Option<f64>,
    /// The width available.
    pub available_width: AvailableSpace,
    /// The height available.
    pub available_height: AvailableSpace,
    text: kivact_engine::contract::LeafTextMeasurer<'a>,
}

impl Constraints<'_> {
    /// The widest the primitive can be: the known width, else a definite
    /// available width (`None`: unbounded, `Some(0.0)`: as narrow as it
    /// can).
    pub fn max_width(&self) -> Option<f64> {
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
    /// use kivact::state::{Signal};
    /// use kivact::view::{IntoView, NodeBuilder};
    /// use kivact::geometry::{Size};
    /// use kivact::draw::TextFormat;
    /// use kivact::custom::{Constraints, Measurement, primitive};
    ///
    /// fn chip(label: Signal<String>) -> impl IntoView {
    ///     primitive().measure(label, |label: &String, c: Constraints| {
    ///         let m = c.measure_text(label, &TextFormat::default());
    ///         // The label sits 4 px below the top.
    ///         Measurement::new(Size::new(m.size.width + 16.0, m.size.height + 8.0)).baseline(m.baseline + 4.0)
    ///     })
    /// }
    /// # let _ = chip;
    /// ```
    pub fn measure_text(&self, text: &str, style: &TextFormat) -> TextMetrics {
        let origin = values::Point { x: 0.0, y: 0.0 };
        let run =
            kivact_style::draw::text_run(text, origin, style, 1.0, values::Rgba::TRANSPARENT, None);
        self.text.measure(&run)
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
    /// use kivact::state::{Signal};
    /// use kivact::view::{IntoView, NodeBuilder, text};
    /// use kivact::draw::{TextFormat};
    /// use kivact::geometry::{Size};
    /// use kivact::custom::{Constraints, Measurement, primitive};
    ///
    /// fn line(text: Signal<String>) -> impl IntoView {
    ///     primitive().measure(text, |text: &String, c: Constraints| {
    ///         let paragraph = c.measure_paragraph(text, &TextFormat::default(), c.max_width());
    ///         Measurement::new(paragraph.size()).baseline(paragraph.baseline())
    ///     })
    /// }
    /// # let _ = line;
    /// ```
    pub fn measure_paragraph(
        &self,
        text: &str,
        style: &TextFormat,
        max_width: Option<f64>,
    ) -> TextParagraph {
        let origin = values::Point { x: 0.0, y: 0.0 };
        let run =
            kivact_style::draw::text_run(text, origin, style, 1.0, values::Rgba::TRANSPARENT, None);
        let shaped = self.text.shape(&run, max_width).unwrap_or_default();
        TextParagraph::from_shaped(text, shaped, max_width)
    }

    /// Measure a paragraph around atomic inline objects with the same typography as drawing.
    pub fn measure_inline_paragraph(
        &self,
        text: &str,
        style: &TextFormat,
        max_width: Option<f64>,
        boxes: &[values::InlineBox],
    ) -> Option<kivact_style::draw::InlineParagraph> {
        let run = kivact_style::draw::text_run(
            text,
            values::Point::default(),
            style,
            1.0,
            values::Rgba::TRANSPARENT,
            None,
        );
        let shaped = self.text.shape_inline(&run, max_width, boxes)?;
        Some(kivact_style::draw::InlineParagraph::from_shaped(
            text, shaped, max_width,
        ))
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

pub(super) fn space(s: values::AvailableSpace) -> AvailableSpace {
    match s {
        values::AvailableSpace::Definite(v) => AvailableSpace::Definite(v),
        values::AvailableSpace::MinContent => AvailableSpace::MinContent,
        values::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
    }
}

pub(super) fn constraints(
    r: values::MeasureRequest,
    text: kivact_engine::contract::LeafTextMeasurer<'_>,
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
    pub baseline: Option<f64>,
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
    pub fn baseline(mut self, y: impl Into<f64>) -> Self {
        self.baseline = Some(y.into());
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
pub(super) fn sanitize(m: Measurement) -> values::LeafSize {
    let clean = |v: f64| if v.is_finite() { v.max(0.0) } else { 0.0 };
    values::LeafSize {
        size: values::Size {
            width: clean(m.size.width),
            height: clean(m.size.height),
        },
        baseline: m
            .baseline
            .filter(|y| y.is_finite())
            .map(values::Baseline::FromTop),
    }
}
