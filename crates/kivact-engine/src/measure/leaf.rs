//! Role: Measurement of layout leaves with Arvane's text shaping and image sizes.
//!
//! [`Measure`]: the leaves of the layout tree measured with Arvane's
//! shaping (`Engine::measure_text`, cached across frames by the facade,
//! which shapes a paragraph once for every width it is measured and
//! painted at), images by the natural size Arvane's asset layer found.
//!
//! Arvane has no layout: the model's geometry (rects, hit testing,
//! scrolling, overlay placement, the reports) reads layouts on its retained
//! nodes. The `arvane` crates stay free of Kivact.
//!
//! `Measure` is one concrete type that calls each kind of leaf's measuring
//! directly (text, text input, image, a measured box's closure). A borrowed
//! `EngineMeasure` adapter implements layout's independent `LeafMeasure`.
//! What it keeps between passes it keeps in the nodes (`HostNode::extents`
//! for measured text only, in a box, and the image's natural size), which
//! the model's semantics never read.
//!
//! Invariants:
//! - A text leaf wraps at a definite available width, at every break
//!   opportunity for min-content and not at all for max-content; text
//!   whose one unwrapped line already fits is not shaped again. Only
//!   wrapping and an ellipsis (`narrows`) make a width narrower than the
//!   line measure otherwise; a text with an ellipsis is measured at that
//!   width (at 0 for min-content), so it shrinks in a row as a CSS item
//!   with `overflow: hidden` does.
//! - A text's extents (`HostNode::extents`) are kept per content and
//!   style, its colors and decorations aside (a change of those alone
//!   measures nothing again): unwrapped once, and once for each of the
//!   last `WRAPPED_PER_NODE` (4) narrower widths. They are dropped when
//!   the node is measured again (`Measure::measures_same`).
//! - A leaf's first baseline (`LeafSize::baseline`, which the layout
//!   places from the border box's top as taffy's first baseline): a text's
//!   first line's; a single-line input's from its content box's centre
//!   (the line box is centered there, `kivact_text::line_offset`); a
//!   multiline input's from its top; an input with no text a space's. An
//!   image, and a measured leaf that gives none, have none (the bottom of
//!   the box). A new baseline at the same size lays out again.
//! - A rich text's spans have their families resolved and are converted to
//!   Arvane spans each time it is measured, their colors not measured; a
//!   plain text allocates none.
//! - A measured width is the shaped width rounded up to whole pixels plus
//!   half a pixel of slack: the layout rounds boxes to
//!   pixels, and a box narrower than the shaped line would make the paint
//!   (which shapes at the box) wrap it.
//! - A text input never wraps: one line (a single-line input shows its
//!   line feeds as spaces), or the value's lines in a multiline one; it is
//!   at least 120 wide and as high as its lines (at least three in a
//!   multiline input). It is measured from its lines (`kivact-text`): its
//!   width is the widest of the lines starting in its first 16 KiB (a
//!   longer document's later lines scroll), so neither an edit nor
//!   opening a long document shapes every line to measure it.
//! - Text that cannot be shaped (no font) measures as its minimum; an
//!   image still loading or unreadable has no natural size, so the image
//!   rule uses its `Intrinsic` preferred size. Neither fails the
//!   layout.
//! - A text's family is resolved as it is painted (`assets::text_style`);
//!   a missing family (a rich text span's too) is reported here, once per
//!   family, at the first text measured with it
//!   (`ArvaneAssets::check_family`).
//! - A measured leaf (an extension primitive: a box with a `Measure`
//!   prop) is sized by its closure alone, which wins over its `Intrinsic`
//!   size; the node's layout record keeps
//!   its sizes until a new `Measure` prop (new content) arrives, which is
//!   measured again only under the recorded requests first
//!   ([`Measure::measures_same`]). The closure (`wire::LeafMeasure`, equal
//!   when it is the same `Rc`) runs synchronously in the layout pass with
//!   the model borrowed; the text measurer it is lent borrows the Arvane
//!   engine of the pass, never the tree or a reader again, and its runs
//!   take the leaf's inherited family and line height. A `Measure` prop
//!   is content: it restacks no paint list. A registered font measures
//!   every measured leaf again (it may measure text).
//! - Content that changed is checked by [`Measure::measures_same`] alone:
//!   when its record is complete (no request was dropped since the last
//!   edit) and the new content gives the same `LeafSize`, baseline
//!   included, under every recorded request, the last computation holds
//!   and taffy does not run; otherwise the leaf's record is cleared and it
//!   is laid out again. No other skip guesses from widths or heights. An
//!   image whose natural size changed is always laid out again.
//! - The natural size an image was measured with is recorded in its node
//!   (`HostNode::natural`), so `Model::poll_assets` measures again only
//!   the images whose natural size changed.

use crate::tree::NodeTree;
use kivact_wire::{AvailableSpace, MeasureRequest};
use kivact_wire::{Baseline, LeafSize, NodeId, NodeType, Size};

use crate::measure::editor::EditorNode;
use crate::measure::state::{MeasureColumns, TextExtents};
use kivact_lower::assets::ArvaneAssets;
use kivact_lower::assets::text_style;
use kivact_text::{line_height, line_offset};

/// Measures leaves with an Arvane engine and the host's assets.
pub(crate) struct Measure<'a> {
    engine: &'a mut arvane::Engine,
    assets: &'a mut ArvaneAssets,
    cache: &'a MeasureColumns,
    editors: &'a [Option<Box<EditorNode>>],
}

/// One pass borrows content/resource inputs without mutating authoritative topology.
pub(crate) struct EngineMeasure<'a, 'm> {
    pub(crate) tree: &'a NodeTree,
    pub(crate) measure: &'a mut Measure<'m>,
}

impl kivact_layout::LeafMeasure for EngineMeasure<'_, '_> {
    fn measure(&mut self, id: NodeId, request: MeasureRequest) -> LeafSize {
        self.measure.measure(self.tree, id, request)
    }
}

impl<'a> Measure<'a> {
    pub(crate) fn new(
        engine: &'a mut arvane::Engine,
        assets: &'a mut ArvaneAssets,
        cache: &'a MeasureColumns,
        editors: &'a [Option<Box<EditorNode>>],
    ) -> Self {
        Self {
            engine,
            assets,
            cache,
            editors,
        }
    }

    /// The extent of the text node `id` (content `text`, `style`) at
    /// `max_width`, from the node's own extents when they were shaped
    /// with this style: a text is shaped unwrapped once per content, and
    /// wrapped once per width narrower than its line.
    fn node_text(
        &mut self,
        node: &crate::tree::node::HostNode,
        text: kivact_types::protocol::RichText<'_>,
        mut style: kivact_types::protocol::TextStyle,
        max_width: Option<f32>,
    ) -> Option<LeafSize> {
        // Colors and decorations never change an extent.
        style.color = kivact_types::protocol::TextStyle::default().color;
        style.decoration = kivact_types::protocol::TextDecoration::default();
        let mut extents = self.cache.extents[node.index.0].borrow_mut();
        if extents.as_ref().is_none_or(|e| e.style != style) {
            let single = self.engine.text_metrics(text, &style, None).ok();
            *extents = Some(Box::new(TextExtents {
                style,
                single,
                wrapped: Vec::new(),
            }));
        }
        let extents = extents.as_mut().expect("invariant: set above");
        let single = extents.single?;
        let Some(width) = max_width.filter(|w| narrows(&extents.style, *w, single.size)) else {
            return Some(text_leaf(single));
        };
        if let Some((_, size)) = extents.wrapped.iter().find(|(w, _)| *w == width) {
            return size.map(text_leaf);
        }
        let size = self
            .engine
            .text_metrics(text, &extents.style, Some(width))
            .ok();
        if extents.wrapped.len() == WRAPPED_PER_NODE {
            extents.wrapped.remove(0);
        }
        extents.wrapped.push((width, size));
        size.map(text_leaf)
    }
}

/// Whether text in `style` whose one unwrapped line is `single` measures
/// otherwise at `width`: it wraps, or is cut with an ellipsis.
fn narrows(style: &kivact_types::protocol::TextStyle, width: f32, single: Size) -> bool {
    (style.wrap == kivact_types::protocol::TextWrap::Wrap
        || style.overflow == kivact_types::TextOverflow::Ellipsis)
        && width < single.width
}

/// Wrap widths kept per text node (a layout pass asks a few).
const WRAPPED_PER_NODE: usize = 4;

/// Whole pixels plus half a pixel of slack, so the
/// box the layout rounds to is never narrower than the shaped line and the
/// paint, which shapes at that box, never wraps a line the layout measured
/// as one.
fn slack(size: Size) -> Size {
    Size {
        width: size.width.ceil() + 0.5,
        height: size.height,
    }
}

/// A text's size with slack and its first baseline from the top of its
/// box.
fn text_leaf(metrics: kivact_types::TextMetrics) -> LeafSize {
    LeafSize {
        size: slack(metrics.size),
        baseline: Some(Baseline::FromTop(metrics.baseline)),
    }
}

impl Measure<'_> {
    /// The size of the leaf `id` under `request` (the layout overrides the
    /// dimensions it already knows): a text or text input as shaped, a
    /// measured leaf by its `Measure` closure, any other leaf by its
    /// `Intrinsic` preferred (else minimum) size. Image
    /// nodes are sized by the image rule
    /// (`kivact_layout::image_content_size`) from [`Measure::natural_size`].
    pub(crate) fn measure(
        &mut self,
        tree: &NodeTree,
        id: NodeId,
        request: MeasureRequest,
    ) -> LeafSize {
        let Some(n) = tree.get(id) else {
            return LeafSize::default();
        };
        match n.kind() {
            _ if self.editors[n.index.0].is_some() => {
                self.assets.check_family(
                    self.engine,
                    id,
                    crate::measure::computed_text(tree, self.cache, id)
                        .family
                        .as_deref(),
                );
                let style = crate::measure::input_style(tree, self.cache, id, self.engine);
                let line = line_height(self.engine, &style);
                let (count, width, first) = match crate::measure::input::edit_lines(
                    tree,
                    self.cache,
                    self.editors,
                    id,
                    self.engine,
                ) {
                    Some((mut lines, text)) => (
                        lines.len(),
                        lines.measured_width(text, self.engine),
                        lines.baseline(text, 0, self.engine),
                    ),
                    None => (1, 0.0, 0.0),
                };
                let lines = if n.multiline() { count.max(3) } else { 1 };
                let min = Size {
                    width: 120.0,
                    height: lines as f32 * line,
                };
                let size = slack(Size {
                    width,
                    height: count as f32 * line,
                });
                // A single-line input's line sits `line_offset` into its
                // content box (as `text_geometry::line_box` places it), so
                // its baseline is measured from the box's center: the
                // offset in a box of no height is the line's top above it.
                // A multiline input's starts at its top.
                let baseline = if n.multiline() {
                    Baseline::FromTop(first)
                } else {
                    Baseline::FromCenter(line_offset(0.0, line) + first)
                };
                LeafSize {
                    size: Size {
                        width: size.width.max(min.width),
                        height: size.height.max(min.height),
                    },
                    baseline: Some(baseline),
                }
            }
            NodeType::Image => {
                let (width, height) = kivact_layout::style_known(request, n.layout());
                let natural = self.natural_size(tree, id);
                kivact_layout::image_content_size(width, height, n.intrinsic(), natural).into()
            }
            NodeType::Text => {
                let props = crate::measure::computed_text(tree, self.cache, id);
                self.assets
                    .check_family(self.engine, id, props.family.as_deref());
                let style = text_style(self.engine, &props);
                let max_width = match request.available_width {
                    AvailableSpace::Definite(w) => Some(w.max(0.0)),
                    AvailableSpace::MinContent => Some(0.0),
                    AvailableSpace::MaxContent => None,
                };
                for span in n.spans() {
                    self.assets
                        .check_family(self.engine, id, span.style.family.as_deref());
                }
                let spans = rich_spans(self.engine, n.spans(), &style);
                let text = kivact_types::protocol::RichText {
                    text: n.content().unwrap_or(""),
                    spans: &spans,
                };
                self.node_text(n, text, style, max_width)
                    .unwrap_or_default()
            }

            _ => {
                // A measured leaf may measure text: like a scene's run in it
                // (its family and line height are the leaf's inherited ones).
                let text = LeafText {
                    engine: std::cell::RefCell::new(&mut *self.engine),
                    inherited: std::cell::OnceCell::new(),
                    tree,
                    columns: self.cache,
                    id,
                };
                leaf_size(n, request, kivact_wire::LeafTextMeasurer::new(&text))
            }
        }
    }

    /// Whether the new content measures exactly as before under every
    /// recorded request, so the previous layout and measurements hold.
    /// Clear old text extents before comparing the complete LeafSize.
    /// Images always invalidate layout when their natural size changes.
    pub(crate) fn measures_same(
        &mut self,
        tree: &NodeTree,
        columns: &kivact_layout::NodeLayoutColumns,
        id: NodeId,
    ) -> bool {
        let Some(n) = tree.get(id) else {
            return true;
        };
        self.cache.extents[n.index.0].replace(None);
        if n.kind() == NodeType::Image {
            return false;
        }
        let mut leaves = EngineMeasure {
            tree,
            measure: self,
        };
        columns.remeasures_same(n.index.0, id, &mut leaves)
    }

    /// The natural size of the image of the `Image` node `id` in logical
    /// px, once it is loaded (`None` while loading, after a failure),
    /// recorded in its node (an image that finishes loading later is
    /// measured again through [`crate::model::Model::remeasure`]).
    pub(crate) fn natural_size(&mut self, tree: &NodeTree, id: NodeId) -> Option<Size> {
        let n = tree.get(id)?;
        let natural = self.assets.natural_size(n.image());
        self.cache.natural[n.index.0].set(natural);
        natural
    }
}

/// The Arvane spans of a rich text in `style` (empty for plain text: no
/// allocation), their families resolved; colors are not measured.
pub(crate) fn rich_spans(
    engine: &mut arvane::Engine,
    spans: &[kivact_wire::TextSpan],
    style: &kivact_types::protocol::TextStyle,
) -> Vec<kivact_types::protocol::TextSpan> {
    if spans.is_empty() {
        return Vec::new();
    }
    kivact_lower::conversion::spans(spans, style.font_size, kivact_lower::sanitize::color, |f| {
        kivact_lower::assets::family(engine, Some(f))
    })
}

/// The size of a leaf that is neither text nor image: its `Measure`
/// closure's under `request` (an extension primitive's), else its
/// `Intrinsic` preferred (else minimum) size. One pass over the props, as
/// the `Intrinsic` lookup alone was.
fn leaf_size(
    n: &crate::tree::node::HostNode,
    request: MeasureRequest,
    text: kivact_wire::LeafTextMeasurer<'_>,
) -> LeafSize {
    if let Some(measure) = n.measure() {
        return measure.measure(request, text);
    }
    let intrinsic = n.intrinsic();
    intrinsic
        .and_then(|i| i.preferred.or(i.min))
        .unwrap_or_default()
        .into()
}

/// The text service lent to a measured leaf: shapes a run the way a scene
/// draws it in the leaf (its family and line height are the leaf's
/// inherited ones, computed when a run needs them).
struct LeafText<'a> {
    engine: std::cell::RefCell<&'a mut arvane::Engine>,
    inherited: std::cell::OnceCell<std::rc::Rc<kivact_wire::ComputedText>>,
    tree: &'a NodeTree,
    columns: &'a MeasureColumns,
    id: NodeId,
}

impl LeafText<'_> {
    fn inherited(&self, run: &kivact_wire::TextRun) -> Option<&kivact_wire::ComputedText> {
        kivact_lower::glyphs::inherits(run).then(|| {
            &**self
                .inherited
                .get_or_init(|| crate::measure::computed_text(self.tree, self.columns, self.id))
        })
    }
}

impl kivact_wire::TextShaper for LeafText<'_> {
    fn measure(&self, run: &kivact_wire::TextRun) -> kivact_wire::TextMetrics {
        let inherited = self.inherited(run);
        kivact_lower::glyphs::measure_run(&mut self.engine.borrow_mut(), run, inherited)
    }

    fn shape(
        &self,
        run: &kivact_wire::TextRun,
        max_width: Option<f32>,
    ) -> Option<std::sync::Arc<kivact_types::protocol::ShapedText>> {
        let inherited = self.inherited(run);
        kivact_lower::glyphs::shape_run(&mut self.engine.borrow_mut(), run, inherited, max_width)
    }
}
