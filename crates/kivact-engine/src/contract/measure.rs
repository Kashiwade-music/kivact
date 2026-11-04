//! Role: Engine client measure contracts.
use kivact_types::{
    InlineBox, LeafSize, MeasureRequest, ShapedInlineText, ShapedText, TextMetrics, TextRun,
    TextShaper,
};
use std::rc::Rc;
use std::sync::Arc;

/// Text measurement lent to a [`LeafMeasure`] while it runs: the extent a
/// scene's text run (`DrawCmd::Text`, its `origin` and `color` aside)
/// would be drawn with in the measured leaf (a run without a family takes
/// the leaf's inherited one), and the run shaped as a paragraph.
#[derive(Clone, Copy)]
pub struct LeafTextMeasurer<'a>(&'a dyn TextShaper);

impl<'a> LeafTextMeasurer<'a> {
    /// Shape a paragraph with measured inline objects.
    pub fn shape_inline(
        &self,
        run: &TextRun,
        max_width: Option<f64>,
        boxes: &[InlineBox],
    ) -> Option<ShapedInlineText> {
        self.0.shape_inline(run, max_width, boxes)
    }

    /// Lend a text shaper for the duration of a leaf measure.
    pub fn new(shaper: &'a dyn TextShaper) -> Self {
        LeafTextMeasurer(shaper)
    }

    /// The extent of `run`.
    pub fn measure(&self, run: &TextRun) -> TextMetrics {
        self.0.measure(run)
    }

    /// `run` shaped as a paragraph wrapped at `max_width`.
    pub fn shape(&self, run: &TextRun, max_width: Option<f64>) -> Option<Arc<ShapedText>> {
        self.0.shape(run, max_width)
    }
}

impl TextShaper for LeafTextMeasurer<'_> {
    fn shape_inline(
        &self,
        run: &TextRun,
        max_width: Option<f64>,
        boxes: &[InlineBox],
    ) -> Option<ShapedInlineText> {
        self.0.shape_inline(run, max_width, boxes)
    }

    fn measure(&self, run: &TextRun) -> TextMetrics {
        self.0.measure(run)
    }

    fn shape(&self, run: &TextRun, max_width: Option<f64>) -> Option<Arc<ShapedText>> {
        self.0.shape(run, max_width)
    }
}

impl std::fmt::Debug for LeafTextMeasurer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LeafTextMeasurer(..)")
    }
}

/// The size of a measured leaf under a request, computed by kivact.
#[derive(Clone)]
pub struct LeafMeasure(Rc<dyn Fn(MeasureRequest, LeafTextMeasurer<'_>) -> LeafSize>);

impl LeafMeasure {
    /// A leaf measured by `f`, which may measure text with the measurer it
    /// is lent.
    pub fn new(f: impl Fn(MeasureRequest, LeafTextMeasurer<'_>) -> LeafSize + 'static) -> Self {
        LeafMeasure(Rc::new(f))
    }

    /// The leaf's size (and baseline) under `request`.
    pub fn measure(&self, request: MeasureRequest, text: LeafTextMeasurer<'_>) -> LeafSize {
        (self.0)(request, text)
    }
}

impl PartialEq for LeafMeasure {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for LeafMeasure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LeafMeasure(..)")
    }
}
