//! Role: The `Draw` recorder: its state, transform and the `Recording` it yields.

use crate::color_value::Color;
use crate::geometry::Size;
use crate::length::Resolve;
use crate::paint::{Paint, fill_layer};
use crate::theme::value::Theme;
use kivact_types as p;
use kivact_types::{DrawCmd, DrawList, DrawableId, Fill, RegionId};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// DrawList builder passed to draw closures.
///
/// A drawing's lengths resolve in the theme in effect: `em` is the theme's
/// base size (a drawing has no style), and an invalid sum is counted as
/// unsupported. Text spacing resolves in the theme the text is drawn in,
/// so a measure of the same text agrees. Two places drop an invalid length
/// without counting it: measuring text (`Draw::measure_text`,
/// `Draw::layout_paragraph` and the `Constraints` measures), because the
/// drawing of the same format counts it; and an editor's highlighter
/// decoration, which runs inside layout, where nothing is reported.
pub struct Draw {
    pub(super) theme: Theme,
    pub(super) cmds: Vec<DrawCmd>,
    pub(super) regions: Vec<p::Region>,
    pub(super) declarations: HashMap<RegionId, RegionSemantics>,
    pub(super) drawn_regions: HashSet<RegionId>,
    pub(super) drawn_drawables: HashSet<DrawableId>,
    pub(super) region_errors: Vec<String>,
    /// The canvas size (percentages in layer transforms are of it).
    pub(super) size: Size,
    pub(super) transform: p::Affine,
    pub(super) unsupported: u32,
    /// Measures and shapes text.
    pub(super) measurer: Rc<dyn p::TextShaper>,
}

/// What a recording needs from its host, given at once: the theme its
/// colors resolve in, the text engine and the regions the node's setup
/// declared (empty for an offscreen drawing).
pub struct RecordContext {
    /// The theme in effect.
    pub theme: Theme,
    /// Measures and shapes text.
    pub shaper: Rc<dyn p::TextShaper>,
    /// The declared regions, each drawable once per layer.
    pub regions: Vec<(RegionId, RegionSemantics)>,
}

/// What one [`crate::internal::record`] made.
pub struct Recording {
    /// The commands and the regions, in drawing order.
    pub scene: DrawList,
    /// Commands the painter dropped as unsupported (pixels whose size
    /// disagrees with their buffer, a canvas shadow's spread or inset).
    pub unsupported: u32,
    /// Messages about undeclared regions and geometry repeated in one
    /// layer, which were dropped.
    pub region_errors: Vec<String>,
}

impl Draw {
    pub(super) fn new(cx: RecordContext, size: Size) -> Self {
        Draw {
            measurer: cx.shaper,
            theme: cx.theme,
            cmds: Vec::new(),
            regions: Vec::new(),
            declarations: cx.regions.into_iter().collect(),
            drawn_regions: HashSet::new(),
            drawn_drawables: HashSet::new(),
            region_errors: Vec::new(),
            size,
            transform: p::Affine::IDENTITY,
            unsupported: 0,
        }
    }

    /// The theme in effect (for token lookups in custom code).
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// The size of the area drawn in, in logical px: the node's size in a
    /// `canvas` or `draw` callback (the same value its `Size` argument
    /// has), the image's in `Offscreen::render`. Percentages in layer
    /// transforms are of it.
    pub fn size(&self) -> Size {
        self.size
    }

    pub(super) fn color(&self, c: Color) -> p::Rgba {
        c.resolve(&self.theme)
    }

    /// Run `f` with the lengths of this drawing resolved: the theme's
    /// variables, and `em` of the theme's base size (a drawing has no
    /// style). An invalid sum counts as unsupported.
    pub(super) fn resolving<T>(&mut self, f: impl FnOnce(&mut Resolve<'_>) -> T) -> T {
        let mut misplaced = 0;
        let out = f(&mut Resolve::of_theme(&self.theme, &mut misplaced));
        self.unsupported += misplaced;
        out
    }

    /// Paint tokens are resolved in the same place as element styles.
    pub(super) fn fill_of(&mut self, paint: &Paint) -> Fill {
        if let Some(c) = paint.solid() {
            return Fill::Solid(self.color(c));
        }
        Fill::Layers(vec![self.resolving(|cx| fill_layer(paint, cx))])
    }
}

/// What a node's setup declared about a canvas region (`Node::region`),
/// resolved by node mounting and handed to [`record`] in its
/// [`RecordContext`], so that `region`, `region_shape` and `region_path`
/// can publish the region's accessibility leaf.
#[derive(Clone, Debug, PartialEq)]
pub struct RegionSemantics {
    /// Canonical accessibility payload for this region.
    pub semantics: p::Semantics<p::FocusTarget>,
    /// Position in the node's region focus order.
    pub focus_order: u32,
    /// Optional press focus destination.
    pub focus_on_press: Option<p::FocusTarget>,
}

/// Record one draw layer of a node of `size` with what `cx` gives: run `f`
/// on a fresh painter and return what it drew. `f` runs the application's
/// drawing, then the retained style paint (core calls `LocalPaint::draw`
/// from its draw effect). A panic in `f` propagates.
pub fn record(size: Size, cx: RecordContext, f: impl FnOnce(&mut Draw)) -> Recording {
    let mut draw = Draw::new(cx, size);
    f(&mut draw);
    Recording {
        scene: DrawList {
            cmds: draw.cmds,
            regions: draw.regions,
        },
        unsupported: draw.unsupported,
        region_errors: draw.region_errors,
    }
}
