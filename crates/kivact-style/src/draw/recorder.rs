//! Role: The `Draw` recorder: its state, transform and the `Recording` it yields.

use crate::color::Color;
use crate::geometry::Size;
use crate::paint::{Paint, fill_layer};
use crate::theme::value::Theme;
use kivact_types as p;
use kivact_types::{DrawCmd, DrawList, Fill, RegionId};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// DrawList builder passed to draw closures.
pub struct Draw {
    pub(super) theme: Theme,
    pub(super) cmds: Vec<DrawCmd>,
    pub(super) regions: Vec<p::Region>,
    pub(super) declarations: HashMap<RegionId, RegionSemantics>,
    pub(super) drawn_regions: HashSet<RegionId>,
    pub(super) region_errors: Vec<String>,
    /// The canvas size (percentages in layer transforms are of it).
    pub(super) size: Size,
    pub(super) transform: p::Transform,
    pub(super) unsupported: u32,
    /// Measures text (`None` before the engine is there).
    pub(super) measurer: Option<Rc<dyn p::TextShaper>>,
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
    /// Install the framework text engine before recording draw commands.
    /// Core mounting and offscreen recording supply the inherited text style
    /// automatically; application draw closures receive an initialized painter.
    // xtask-allow(internal-contract): Core mounting and offscreen recording automatically install the backend text shaper across the style boundary.
    #[doc(hidden)]
    pub fn set_text_measurer(&mut self, shaper: impl p::TextShaper + 'static) {
        self.measurer = Some(Rc::new(shaper));
    }

    pub(super) fn new(theme: Theme, size: Size) -> Self {
        Draw {
            measurer: None,
            theme,
            cmds: Vec::new(),
            regions: Vec::new(),
            declarations: HashMap::new(),
            drawn_regions: HashSet::new(),
            region_errors: Vec::new(),
            size,
            transform: p::Transform::IDENTITY,
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

    /// Paint tokens are resolved in the same place as element styles.
    pub(super) fn fill_of(&self, paint: &Paint) -> Fill {
        if let Some(c) = paint.solid() {
            return Fill::Solid(self.color(c));
        }
        Fill::Layers(vec![fill_layer(paint, &self.theme)])
    }
}

/// What a node's setup declared about a canvas region (`Node::region`),
/// resolved by node mounting and handed to the painter with
/// [`Draw::declare_regions`] before the drawing callbacks run, so that
/// `region`, `region_shape` and `region_path` can publish the region's
/// accessibility leaf.
#[derive(Clone, Debug, PartialEq)]
pub struct RegionSemantics {
    /// Canonical accessibility payload for this region.
    pub semantics: p::Semantics<p::FocusTarget>,
    /// Position in the node's region focus order.
    pub focus_order: u32,
    /// Optional press focus destination.
    pub focus_on_press: Option<p::FocusTarget>,
}

/// Record one draw layer of a node of `size` under `theme`: run `f` on a
/// fresh painter and return what it drew. Inside `f`, first install
/// [`set_text_measurer`](Draw::set_text_measurer) and
/// [`declare_regions`](Draw::declare_regions), then run the
/// application's drawing, then the retained style paint
/// ([`CompiledStyle::paint_retained`](crate::CompiledStyle::paint_retained)).
/// A panic in `f` propagates.
pub fn record(size: Size, theme: &Theme, f: impl FnOnce(&mut Draw)) -> Recording {
    let mut draw = Draw::new(theme.clone(), size);
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
