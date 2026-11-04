//! Role: Measures text for client drawing callbacks.
//!
//! The host keeps Arvane's text engine in an `Rc<RefCell<..>>` it shares
//! with kivact's reader: a draw callback's `Draw::measure_text` borrows it
//! for one call. The layout pass borrows it in `measure::leaf::Measure`
//! and lends it to kivact's measuring closures as a `LeafTextMeasurer`
//! instead. A [`TextMeasurer`] finding it borrowed violates the draw
//! callback contract: debug builds stop, while release builds answer 0.

use std::cell::RefCell;
use std::rc::Rc;

use std::sync::Arc;

use kivact_types::{
    ComputedTextStyle, InlineBox, ShapedInlineText, TextMetrics, TextRun, TextShaper,
};

/// Text measurement for one drawing callback, independent of any node: the
/// shared text engine and the text style the drawn runs inherit. A run
/// without a family or a line height takes the inherited one, exactly as
/// the scene lowering that draws it does, so the metrics agree with the
/// drawing. The inherited style is fixed when the measurer is made; fonts
/// registered later are seen through the shared engine.
#[derive(Clone)]
pub struct TextMeasurer {
    engine: Rc<RefCell<arvane::Engine>>,
    inherited: ComputedTextStyle,
}

impl TextMeasurer {
    pub(crate) fn new(engine: Rc<RefCell<arvane::Engine>>, inherited: ComputedTextStyle) -> Self {
        Self { engine, inherited }
    }

    /// The extent `run` is drawn with (`DrawCmd::Text`, its `origin` and
    /// `color` aside). Text that cannot be shaped has zero extent. Borrowing
    /// the text engine here violates the drawing contract and asserts in
    /// debug builds; release builds return zero instead.
    pub fn measure(&self, run: &TextRun) -> TextMetrics {
        let engine = self.engine.try_borrow_mut();
        debug_assert!(
            engine.is_ok(),
            "text engine is already borrowed during draw measurement"
        );
        engine.map_or_else(
            |_| TextMetrics::default(),
            |mut engine| crate::lower::glyphs::measure_run(&mut engine, run, Some(&self.inherited)),
        )
    }

    /// `run` shaped as a paragraph with its carets, wrapped at `max_width`:
    /// the shaping `DrawCmd::Paragraph` draws. The borrow of the text
    /// engine ends before this returns (the result owns its data). Under
    /// the same contract as [`measure`](Self::measure): a borrow found
    /// taken asserts in debug builds and answers `None` in release builds.
    pub fn shape(
        &self,
        run: &TextRun,
        max_width: Option<f64>,
    ) -> Option<Arc<kivact_types::render::ShapedText>> {
        let engine = self.engine.try_borrow_mut();
        debug_assert!(
            engine.is_ok(),
            "text engine is already borrowed during draw shaping"
        );
        let mut engine = engine.ok()?;
        crate::lower::glyphs::shape_run(&mut engine, run, Some(&self.inherited), max_width)
    }
}

impl TextShaper for TextMeasurer {
    fn measure(&self, run: &TextRun) -> TextMetrics {
        TextMeasurer::measure(self, run)
    }

    fn shape(
        &self,
        run: &TextRun,
        max_width: Option<f64>,
    ) -> Option<Arc<kivact_types::render::ShapedText>> {
        TextMeasurer::shape(self, run, max_width)
    }

    fn shape_inline(
        &self,
        run: &TextRun,
        max_width: Option<f64>,
        boxes: &[InlineBox],
    ) -> Option<ShapedInlineText> {
        let engine = self.engine.try_borrow_mut();
        debug_assert!(
            engine.is_ok(),
            "text engine is already borrowed during draw shaping"
        );
        let mut engine = engine.ok()?;
        crate::lower::glyphs::shape_inline_run(
            &mut engine,
            run,
            Some(&self.inherited),
            max_width,
            boxes,
        )
    }
}
