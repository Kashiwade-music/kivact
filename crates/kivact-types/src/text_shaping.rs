//! Role: Backend-neutral text shaping and measurement queries.
use crate::{InlineBox, ShapedInlineText, ShapedText, TextMetrics, TextRun};
use std::sync::Arc;

/// What a host measures and shapes text with, for a callback that has no
/// engine of its own: a scene's text run as it would be drawn in the node
/// the callback belongs to (a run without a family takes the node's
/// inherited one).
pub trait TextShaper {
    /// The extent of `run` on one line (`SceneCmd::Text`, its `origin` and
    /// `color` aside); zero when it cannot be shaped.
    fn measure(&self, run: &TextRun) -> TextMetrics;

    /// `run`'s text shaped with its carets and lines, wrapped at
    /// `max_width` (`None`: one line per hard line). Equal inputs share one
    /// `Arc`. `None` when it cannot be shaped. The result owns its data, so
    /// the shaper is not borrowed afterwards.
    fn shape(&self, run: &TextRun, max_width: Option<f32>) -> Option<Arc<ShapedText>>;

    /// Shape a paragraph with measured inline objects when the host supports it.
    fn shape_inline(
        &self,
        _run: &TextRun,
        _max_width: Option<f32>,
        _boxes: &[InlineBox],
    ) -> Option<ShapedInlineText> {
        None
    }
}
