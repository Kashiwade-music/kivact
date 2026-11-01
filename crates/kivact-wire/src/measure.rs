//! Role: Size measurement protocol.
//!
//! A leaf measures itself from available space and optional fixed dimensions.
//! Hosts may cache equal requests and may re-evaluate them for validation;
//! callbacks must produce the same result for the same request and content.
//! Closure identity defines content identity, so changed content requires a
//! new [`LeafMeasure`].
//!
//! The callback must not re-enter its host or run application effects. The
//! lent [`LeafTextMeasurer`] is the supported text-measurement service during
//! the call. Captured values must also be safe to drop while the host owns its
//! model; hosts may release callbacks during property replacement or removal.

use std::rc::Rc;
use std::sync::Arc;

use super::Size;
use super::scene::TextRun;
use kivact_types::ShapedText;

/// Space available to a node along one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvailableSpace {
    /// A definite number of pixels.
    Definite(f32),
    /// As little as the content allows.
    MinContent,
    /// As much as the content wants.
    MaxContent,
}

/// What a leaf is measured under.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasureRequest {
    /// Width the layout already fixed.
    pub known_width: Option<f32>,
    /// Height the layout already fixed.
    pub known_height: Option<f32>,
    /// Width available.
    pub available_width: AvailableSpace,
    /// Height available.
    pub available_height: AvailableSpace,
}

impl MeasureRequest {
    /// A request for `width` fixed and any height (a host checks whether
    /// new content keeps its size this way, outside a layout pass).
    pub fn at_width(width: f32) -> Self {
        MeasureRequest {
            known_width: Some(width),
            known_height: None,
            available_width: AvailableSpace::Definite(width),
            available_height: AvailableSpace::MaxContent,
        }
    }
}

/// The extent of measured text.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextMetrics {
    /// Logical width of the measured text.
    pub width: f32,
    /// Logical height of the measured text.
    pub height: f32,
    /// The first line's baseline, from the top.
    pub baseline: f32,
}

impl TextMetrics {
    /// The width and height.
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

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
}

/// Text measurement lent to a [`LeafMeasure`] while it runs: the extent a
/// scene's text run (`SceneCmd::Text`, its `origin` and `color` aside)
/// would be drawn with in the measured leaf (a run without a family takes
/// the leaf's inherited one), and the run shaped as a paragraph.
#[derive(Clone, Copy)]
pub struct LeafTextMeasurer<'a>(&'a dyn TextShaper);

impl<'a> LeafTextMeasurer<'a> {
    /// Lend a text shaper for the duration of a leaf measure.
    pub fn new(shaper: &'a dyn TextShaper) -> Self {
        LeafTextMeasurer(shaper)
    }

    /// The extent of `run`.
    pub fn measure(&self, run: &TextRun) -> TextMetrics {
        self.0.measure(run)
    }

    /// `run` shaped as a paragraph wrapped at `max_width`.
    pub fn shape(&self, run: &TextRun, max_width: Option<f32>) -> Option<Arc<ShapedText>> {
        self.0.shape(run, max_width)
    }
}

impl TextShaper for LeafTextMeasurer<'_> {
    fn measure(&self, run: &TextRun) -> TextMetrics {
        self.0.measure(run)
    }

    fn shape(&self, run: &TextRun, max_width: Option<f32>) -> Option<Arc<ShapedText>> {
        self.0.shape(run, max_width)
    }
}

impl std::fmt::Debug for LeafTextMeasurer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LeafTextMeasurer(..)")
    }
}

/// Where a measured leaf's first baseline lies, vertically in its content
/// box (inside border and padding).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Baseline {
    /// This far below the top of the content box.
    FromTop(f32),
    /// This far below the vertical center of the content box (negative:
    /// above it): a line centered in a box taller than itself.
    FromCenter(f32),
}

/// A measured leaf's content size and, when it has text on a line, its
/// first baseline.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LeafSize {
    pub size: Size,
    /// `None`: the leaf has no baseline, and a flex container aligning it
    /// by baseline takes the bottom of its box.
    pub baseline: Option<Baseline>,
}

impl From<Size> for LeafSize {
    fn from(size: Size) -> Self {
        LeafSize {
            size,
            baseline: None,
        }
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
