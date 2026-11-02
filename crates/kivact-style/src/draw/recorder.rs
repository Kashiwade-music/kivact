//! Role: The `Draw` recorder: its state, transform and the `Recording` it yields.

use crate::color::Color;
use crate::geometry::{Point, Size};
use crate::paint::{Paint, fill_layer};
use crate::theme::value::Theme;
use kivact_wire as p;
use kivact_wire::{A11yAction, Fill, RegionId, Role, Scene, SceneCmd};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// The matrix a painter maps its coordinates by, `(x, y) → (a·x + c·y +
/// e, b·x + d·y + f)`: the layer transforms, resolved against the canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Affine {
    pub(super) m: [f32; 6],
}

impl Affine {
    pub(super) const IDENTITY: Affine = Affine {
        m: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// `self` followed by `next` (a point mapped by `self`, then by `next`).
    pub(super) fn then(self, next: Affine) -> Affine {
        let [a1, b1, c1, d1, e1, f1] = self.m;
        let [a2, b2, c2, d2, e2, f2] = next.m;
        Affine {
            m: [
                a2 * a1 + c2 * b1,
                b2 * a1 + d2 * b1,
                a2 * c1 + c2 * d1,
                b2 * c1 + d2 * d1,
                a2 * e1 + c2 * f1 + e2,
                b2 * e1 + d2 * f1 + f2,
            ],
        }
    }

    pub(super) fn apply(&self, pt: Point) -> Point {
        let [a, b, c, d, e, f] = self.m;
        Point::new(a * pt.x + c * pt.y + e, b * pt.x + d * pt.y + f)
    }
}

/// Scene builder passed to draw closures.
pub struct Draw {
    pub(super) theme: Theme,
    pub(super) cmds: Vec<SceneCmd>,
    pub(super) regions: Vec<p::Region>,
    pub(super) declarations: HashMap<RegionId, RegionSemantics>,
    pub(super) drawn_regions: HashSet<RegionId>,
    pub(super) region_errors: Vec<String>,
    /// The canvas size (percentages in layer transforms are of it).
    pub(super) size: Size,
    pub(super) transform: Affine,
    pub(super) unsupported: u32,
    /// Measures text (`None` before the engine is there).
    pub(super) measurer: Option<Rc<dyn p::TextShaper>>,
}

/// What one [`Draw::record`] made.
pub struct Recording {
    /// The commands and the regions, in drawing order.
    pub scene: Scene,
    /// Commands the painter dropped as unsupported (pixels whose size
    /// disagrees with their buffer, a canvas shadow's spread or inset).
    pub unsupported: u32,
    /// Messages about undeclared regions and geometry repeated in one
    /// layer, which were dropped.
    pub region_errors: Vec<String>,
}

impl Draw {
    /// Record one draw layer of a node of `size` under `theme`: run `f` on a
    /// fresh painter and return what it drew. Inside `f`, first install
    /// [`set_text_measurer`](Draw::set_text_measurer) and
    /// [`declare_regions`](Draw::declare_regions), then run the
    /// application's drawing, then the retained style paint
    /// ([`CompiledStyle::paint_retained`](super::CompiledStyle::paint_retained)).
    /// A panic in `f` propagates.
    pub fn record(size: Size, theme: &Theme, f: impl FnOnce(&mut Draw)) -> Recording {
        let mut draw = Draw::new(theme.clone(), size);
        f(&mut draw);
        Recording {
            scene: Scene {
                cmds: draw.cmds,
                regions: draw.regions,
            },
            unsupported: draw.unsupported,
            region_errors: draw.region_errors,
        }
    }

    /// Measure and shape text with `shaper` from now on (`measure_text`, the
    /// placement of aligned text, `layout_paragraph`). It must shape a run
    /// the way the engine draws it, with the text style the drawing
    /// inherits; without one, `measure_text` answers zero and paragraphs
    /// are empty. Install it before drawing.
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
            transform: Affine::IDENTITY,
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
    /// Role of the accessibility leaf.
    pub role: Role,
    /// Accessible name.
    pub name: String,
    /// Accessible description, if any.
    pub description: Option<String>,
    /// Accessible value text, if any.
    pub value: Option<String>,
    /// Actions the leaf accepts.
    pub actions: Vec<A11yAction>,
    /// Checked state of the region (`None`: not a toggle).
    pub checked: Option<p::CheckedState>,
    /// Selection state.
    pub selected: bool,
    /// Disclosure state (`None`: not expandable).
    pub expanded: Option<bool>,
    /// The region's value is invalid.
    pub invalid: bool,
    /// Typed extra semantic properties (value range, live region, relations,
    /// ..), if any.
    pub properties: Option<Box<p::SemanticsProperties>>,
    /// The region is disabled.
    pub disabled: bool,
    /// The region is hidden from accessibility.
    pub hidden: bool,
    /// The region can take the focus.
    pub focusable: bool,
    /// Position in the focus order among the node's regions.
    pub focus_order: u32,
    /// Where a press moves the focus, if not onto the region itself.
    pub focus_on_press: Option<p::FocusTarget>,
}
