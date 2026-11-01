//! Role: The `Draw` painter: shapes, paths, text, images, layers and canvas regions.
//! Coordinates are node-local logical px after layout.
//! A layer's transform maps every primitive, including text, images and
//! clips, through the renderer. Regions retain their shapes and transforms for hits.
//! A style's `paint(..)` callback is kept here as a `LocalPaint` (see `CompiledStyle::paint_retained`).
//!
//! `canvas`, `draw` and `draw_over` build the wire `Scene`. A group opens
//! only with `d.layer(Layer, closure)` and closes at the end of the
//! closure; there is no public pop. A [`Layer`] holds a transform, a clip
//! (a `Shape`, a [`Path`] or a rectangle) and opacity, blend, filter, mask
//! and shadow, pushed from the outside in: transform, rectangle clip, then
//! `PushLayer`. The transform goes to the renderer as `PushTransform`
//! (style does not flatten it) and applies to text, images, clips, fills,
//! strokes and shadows alike. `line` and `stroke_rect` take a `Stroke` and
//! a `Paint` (a number becomes a solid stroke of that width): a plain solid
//! line keeps its direct command, while caps, joins, dashes and gradients
//! go the way of `stroke_path`. Fills and strokes of paths and shapes share
//! `Paint`, `Shape` and `Stroke` (cap, join, miter, dash, offset); color
//! filters (tint) and layer filters and blending share `paint::effects`
//! with the element style, so a canvas adds no meaning of its own. Rare
//! command values are boxed so an ordinary rectangle scene stays small.
//!
//! A text's `TextStyle.align` is the shared `TextAlign` (Start, Center,
//! End) and places the text horizontally against its anchor (not the
//! style's child alignment); with [`TextBaseline`] (Top, Middle,
//! Alphabetic, Bottom) it places the text by the extent and baseline of the
//! same shaping as `measure_text`. The default Top/Start measures nothing
//! extra.
//!
//! Regions:
//! - A canvas reads the size signal of its node, so its first drawing is
//!   at size 0 and it draws again after the first layout report.
//! - `region`, `region_shape` and `region_path` make an accessibility leaf
//!   and a hit area directly under the node, for a region the node's setup
//!   declared with `Node::region(id, part)` (or `Node::regions`): its role,
//!   name, states (`checked`, `selected`, `expanded`, `indeterminate`, ...)
//!   and input come from that part. A switch track, a selectable item or a
//!   disclosure on a canvas is thus its own native accessibility leaf, and a
//!   state change is published by the next drawing; `kivact::test`'s
//!   `Element` state queries read the region's published state. This is
//!   meaning only: the pointer and the state updates are the component's
//!   handlers.
//! - A region keeps the transform at registration and its original shape:
//!   a hit maps the point back and tests the same outline the renderer
//!   draws. Its accessibility bounds are the bounding box of the
//!   transformed rectangle. Draw and layer clips do not cut a region's hit
//!   area; the node's own clip and transform apply.
//! - `PointerEvent::region` and `A11yEvent::region` carry the author's
//!   `RegionId`; the node itself is `None`.
//! - An undeclared id, or geometry repeated in one layer, is reported and
//!   dropped. Pixels whose size disagrees with their buffer and a canvas
//!   shadow's spread or inset are `Unsupported`, reported once per layer,
//!   located at the node; the engine's scene culling includes the reach of
//!   transforms, strokes and filters.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use kivact_wire as p;
use kivact_wire::{A11yAction, Fill, Role, Scene, SceneCmd};

use crate::asset::Asset;
use crate::color::Color;
use crate::effect::{Shape, basic, filter_ops};
use crate::paint::{Paint, fill_layer};
use crate::text::{FontFeature, FontKerning, LineHeight, TextDecorationLine, TextLength};
use crate::theme::value::Theme;
mod image;
mod layer;
mod paragraph;
pub use super::geometry::{IntrinsicSize, Point, Rect, Size};
pub use super::path::{FillRule, Path, PathContour};
#[cfg(feature = "gpu-interop")]
pub use image::GpuFrame;
pub use image::{ImageOptions, Pixels};
pub use layer::Layer;
pub use paragraph::TextParagraph;

pub use kivact_wire::RegionId;

/// Ends of open contours and dashes.
#[doc(inline)]
pub use kivact_wire::LineCap;
/// How adjacent segments meet.
#[doc(inline)]
pub use kivact_wire::LineJoin;
/// A line's width, caps, joins and repeating dash pattern, in logical px.
#[doc(inline)]
pub use kivact_wire::Stroke;

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
    /// Typed semantic properties (checked, selected, expanded, ...), if any.
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

/// The matrix a painter maps its coordinates by, `(x, y) → (a·x + c·y +
/// e, b·x + d·y + f)`: the layer transforms, resolved against the canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Affine {
    m: [f32; 6],
}

impl Affine {
    const IDENTITY: Affine = Affine {
        m: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// `self` followed by `next` (a point mapped by `self`, then by `next`).
    fn then(self, next: Affine) -> Affine {
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

    fn apply(&self, pt: Point) -> Point {
        let [a, b, c, d, e, f] = self.m;
        Point::new(a * pt.x + c * pt.y + e, b * pt.x + d * pt.y + f)
    }
}

/// Text style for `Draw::text` and `Draw::measure_text`: the text
/// vocabulary of element styles, so canvas text matches element text.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::draw::TextStyle;
/// use kivact::style::{TextDecorationLine, em};
/// let label = TextStyle {
///     size: 11.0,
///     weight: 600,
///     letter_spacing: em(0.08),
///     tabular_nums: true,
///     decoration: TextDecorationLine::UNDERLINE,
///     ..TextStyle::default()
/// };
/// # let _ = label;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    /// Horizontal placement relative to the anchor. `Justify` places like
    /// `Start`: a line drawn alone is the last of its paragraph.
    pub align: crate::TextAlign,
    /// Vertical placement relative to the anchor.
    pub baseline: TextBaseline,
    pub size: f32,
    pub weight: u16,
    pub color: Color,
    /// A family name or a generic family (`"system-ui"`, `"sans-serif"`,
    /// `"serif"`, `"monospace"`), or an ordered comma-separated CSS list;
    /// `None` (the default) uses the family the
    /// node inherits (`style().font_family(..)`, else the theme's).
    pub family: Option<String>,
    /// Use the italic face of the family when it has one (default `false`).
    pub italic: bool,
    /// The distance between lines (CSS `line-height`; a number is a
    /// multiple of `size`, `LineHeight::px(20)` is px). `None` (the
    /// default) uses the line height the node inherits
    /// (`style().line_height(..)`, else `Theme.text.line_height`), as
    /// text elements do.
    pub line_height: Option<LineHeight>,
    /// Extra space after every character (CSS `letter-spacing`; px or
    /// `em(..)` of `size`).
    pub letter_spacing: TextLength,
    /// Extra space after every space (CSS `word-spacing`; px or `em(..)`
    /// of `size`).
    pub word_spacing: TextLength,
    /// Pair kerning (CSS `font-kerning`).
    pub font_kerning: FontKerning,
    /// OpenType features (CSS `font-feature-settings`):
    /// `vec![("ss01", 1).into()]`.
    pub features: Vec<FontFeature>,
    /// Digits of one width (CSS `font-variant-numeric: tabular-nums`).
    pub tabular_nums: bool,
    /// Lines drawn with the text (CSS `text-decoration-line`).
    pub decoration: TextDecorationLine,
    /// Their color (default: `color`).
    pub decoration_color: Option<Color>,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            align: crate::TextAlign::Start,
            baseline: TextBaseline::Top,
            size: 14.0,
            weight: 400,
            color: Color::token(crate::ColorToken::Text),
            family: None,
            italic: false,
            line_height: None,
            letter_spacing: TextLength::default(),
            word_spacing: TextLength::default(),
            font_kerning: FontKerning::Auto,
            features: Vec::new(),
            tabular_nums: false,
            decoration: TextDecorationLine::NONE,
            decoration_color: None,
        }
    }
}

/// The extent of measured text: [`Draw::measure_text`] and
/// `Constraints::measure_text` of `kivact::primitive`,
/// what `Draw::text` draws (one line, unwrapped) in logical px.
/// `width` is the line advance, `height` is its height, and `baseline`
/// is measured from its top (the `pos` passed to `Draw::text`).
#[doc(inline)]
pub use kivact_wire::TextMetrics;

/// The text run of a [`TextStyle`] in the engine's form: what `Draw::text`
/// draws and `Draw::measure_text` measures. It owns the compiled text, its
/// spacing and its font features; shaping is the engine's.
#[derive(Clone, Debug)]
pub struct TextRun(p::TextRun);

impl TextRun {
    /// The run of `text` at `origin` in `style` scaled by `scale` (layer
    /// transforms), colored `color`. Its decoration color resolves against
    /// `theme`; measuring needs no color and passes `None`. A run without a
    /// family or a line height keeps taking the inherited one.
    pub fn from_style(
        text: &str,
        origin: p::Point,
        style: &TextStyle,
        scale: f32,
        color: p::Rgba,
        theme: Option<&Theme>,
    ) -> Self {
        let size = style.size * scale;
        let (features, _) = crate::text::features(Some(&style.features), style.tabular_nums);
        Self(p::TextRun {
            text: text.to_string(),
            origin,
            font_size: size,
            weight: style.weight,
            color,
            family: style.family.clone(),
            italic: style.italic,
            line_height: style.line_height,
            letter_spacing: style.letter_spacing.to_wire().px(style.size) * scale,
            word_spacing: style.word_spacing.to_wire().px(style.size) * scale,
            kerning: style.font_kerning,
            features,
            decoration: p::TextDecoration {
                lines: style.decoration,
                // Measuring needs no color (and no theme).
                color: theme.and_then(|t| style.decoration_color.map(|c| c.resolve(t))),
                ..p::TextDecoration::default()
            },
        })
    }

    /// The run in the engine's form, borrowed (no copy of the text or the
    /// features).
    pub fn as_wire(&self) -> &p::TextRun {
        &self.0
    }

    /// The run in the engine's form, consumed (for a scene command).
    pub fn into_wire(self) -> p::TextRun {
        self.0
    }
}

/// Scene builder passed to draw closures.
pub struct Draw {
    theme: Theme,
    cmds: Vec<SceneCmd>,
    regions: Vec<p::Region>,
    declarations: HashMap<RegionId, RegionSemantics>,
    drawn_regions: HashSet<RegionId>,
    region_errors: Vec<String>,
    /// The canvas size (percentages in layer transforms are of it).
    size: Size,
    transform: Affine,
    unsupported: u32,
    /// Measures text (`None` before the engine is there).
    measurer: Option<Rc<dyn p::TextShaper>>,
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

    fn new(theme: Theme, size: Size) -> Self {
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

    /// Supply the node's declared regions before the drawing callbacks run;
    /// a region drawn without a declaration is reported and dropped.
    pub fn declare_regions(&mut self, declarations: Vec<(RegionId, RegionSemantics)>) {
        self.declarations = declarations.into_iter().collect();
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

    fn color(&self, c: Color) -> p::Rgba {
        c.resolve(&self.theme)
    }

    /// Paint tokens are resolved in the same place as element styles.
    fn fill_of(&self, paint: &Paint) -> Fill {
        if let Some(c) = paint.solid() {
            return Fill::Solid(self.color(c));
        }
        Fill::Layers(vec![fill_layer(paint, &self.theme)])
    }
    /// Fill a rectangle with the element styles' paint vocabulary.
    pub fn fill_rect(&mut self, rect: Rect, fill: impl Into<Paint>) {
        let fill = self.fill_of(&fill.into());
        self.cmds.push(SceneCmd::Rect { rect, fill });
    }

    /// Stroke a rectangle with the same stroke and paint vocabulary as paths.
    pub fn stroke_rect(&mut self, rect: Rect, stroke: impl Into<Stroke>, paint: impl Into<Paint>) {
        let stroke = stroke.into();
        let paint = paint.into();
        if let Some(color) = paint.solid().filter(|_| plain_stroke(&stroke)) {
            self.cmds.push(SceneCmd::StrokeRect {
                rect,
                width: stroke.width,
                color: self.color(color),
                radius: 0.0,
            });
        } else {
            self.stroke_path(&Path::new().rect(rect), &stroke, paint);
        }
    }

    pub fn rounded_rect(&mut self, rect: Rect, radius: f32, fill: impl Into<Paint>) {
        let fill = self.fill_of(&fill.into());
        self.cmds.push(SceneCmd::RoundedRect { rect, radius, fill });
    }

    /// Stroke a line with the same stroke and paint vocabulary as paths.
    pub fn line(
        &mut self,
        from: Point,
        to: Point,
        stroke: impl Into<Stroke>,
        paint: impl Into<Paint>,
    ) {
        let stroke = stroke.into();
        let paint = paint.into();
        if let Some(color) = paint.solid().filter(|_| plain_stroke(&stroke)) {
            self.cmds.push(SceneCmd::Line {
                from,
                to,
                width: stroke.width,
                color: self.color(color),
            });
        } else {
            self.stroke_path(&Path::new().move_to(from).line_to(to), &stroke, paint);
        }
    }

    pub fn text(&mut self, text: &str, pos: Point, style: TextStyle) {
        let mut pos = pos;
        if style.align != crate::TextAlign::Start || style.baseline != TextBaseline::Top {
            let m = self.measure_text(text, &style);
            pos.x -= match style.align {
                crate::TextAlign::Center => m.width / 2.0,
                crate::TextAlign::End => m.width,
                // A line drawn alone is the last of its paragraph.
                crate::TextAlign::Start | crate::TextAlign::Justify => 0.0,
            };
            pos.y -= match style.baseline {
                TextBaseline::Top => 0.0,
                TextBaseline::Middle => m.height / 2.0,
                TextBaseline::Alphabetic => m.baseline,
                TextBaseline::Bottom => m.height,
            };
        }
        let run = TextRun::from_style(
            text,
            pos,
            &style,
            1.0,
            self.color(style.color),
            Some(&self.theme),
        );
        self.cmds.push(SceneCmd::Text(run.into_wire()));
    }
    /// The extent `text(text, pos, style)` draws `text` with, before the
    /// layer transforms (in this painter's coordinates): size a label's
    /// pill, center a label (`(size.height - m.height) / 2`), place a
    /// baseline. The same shaping as the drawing, so they agree; zero
    /// where there is no engine to measure with.
    ///
    /// ```
    /// use kivact::prelude::*;
    /// use kivact::draw::{Draw, Point, Rect};
    /// use kivact::draw::TextStyle;
    /// // A pill as wide as its label plus 12 px.
    /// fn pill(d: &mut Draw, label: &str, at: Point) {
    ///     let style = TextStyle { size: 11.0, ..TextStyle::default() };
    ///     let m = d.measure_text(label, &style);
    ///     d.rounded_rect(Rect::new(at.x, at.y, m.width + 12.0, m.height + 4.0), 4.0, ColorToken::Accent);
    ///     d.text(label, Point::new(at.x + 6.0, at.y + 2.0), style);
    /// }
    /// # let _ = pill;
    /// ```
    pub fn measure_text(&self, text: &str, style: &TextStyle) -> TextMetrics {
        let Some(m) = &self.measurer else {
            return TextMetrics::default();
        };
        let origin = p::Point { x: 0.0, y: 0.0 };
        let run = TextRun::from_style(text, origin, style, 1.0, p::Rgba::TRANSPARENT, None);
        m.measure(run.as_wire())
    }

    /// Shape `text` in `style`, wrapped at `max_width` (`None`: at hard line
    /// breaks only), with the window's text engine: the paragraph
    /// [`Draw::paragraph`] draws, and the one `Constraints::measure_paragraph`
    /// makes of the same inputs. Without an engine to shape with it is empty.
    pub fn layout_paragraph(
        &self,
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> TextParagraph {
        let origin = p::Point { x: 0.0, y: 0.0 };
        let run = TextRun::from_style(text, origin, style, 1.0, p::Rgba::TRANSPARENT, None);
        let shaped = self
            .measurer
            .as_ref()
            .and_then(|shaper| shaper.shape(run.as_wire(), max_width))
            .unwrap_or_default();
        TextParagraph::from_shaped(text, shaped, max_width)
    }

    /// Draw `paragraph` with its top left at `origin`, in `color`: the glyphs
    /// and decoration lines it was shaped with, not shaped again.
    pub fn paragraph(
        &mut self,
        paragraph: &TextParagraph,
        origin: Point,
        color: impl crate::theme::value::ThemeColor,
    ) {
        let color = self.color(color.resolve(&self.theme));
        self.cmds.push(SceneCmd::Paragraph {
            shaped: paragraph.shaped(),
            origin,
            color,
        });
    }

    /// Draw `source` stretched to `rect` (`ImageFit::Fill`). The image is
    /// cached by the asset's identity, so create the `Asset` once outside
    /// the draw closure and borrow it here (`Asset::from(bytes.clone())`
    /// inside the closure is a new asset, decoded again on every redraw).
    /// It draws nothing until loaded, and a failure is never reported
    /// (only `image(..)` nodes report `ErrorKind::Asset`).
    pub fn image(&mut self, source: &Asset, rect: Rect) {
        self.cmds.push(SceneCmd::Image {
            source: source.clone(),
            rect,
        });
    }

    /// Paint the hit geometry of a region declared by the canvas node's setup.
    ///
    /// Semantics and input come from the attached part. Regions follow the
    /// current transform and remain independent of draw clips. An undeclared
    /// identifier or repeated geometry in one drawing layer is reported and
    /// ignored. A declaration without geometry is not a native focus target.
    pub fn region(&mut self, id: RegionId, rect: Rect) {
        self.push_region(id, rect);
    }

    fn push_region(&mut self, id: RegionId, rect: Rect) -> bool {
        let Some(semantics) = self.declarations.get(&id) else {
            self.region_errors
                .push(format!("canvas region {} has no setup declaration", id.0));
            return false;
        };
        if !self.drawn_regions.insert(id) {
            self.region_errors.push(format!(
                "canvas region {} has duplicate geometry in one layer",
                id.0
            ));
            return false;
        }
        if ![rect.x, rect.y, rect.width, rect.height]
            .iter()
            .all(|v| v.is_finite())
        {
            self.region_errors
                .push(format!("canvas region {} has invalid geometry", id.0));
            return false;
        }
        // Initial layout may supply a zero-sized canvas; it has no hit geometry.
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return false;
        }
        let semantics = semantics.clone();
        let pts = [
            self.transform.apply(Point::new(rect.x, rect.y)),
            self.transform
                .apply(Point::new(rect.x + rect.width, rect.y + rect.height)),
            self.transform
                .apply(Point::new(rect.x + rect.width, rect.y)),
            self.transform
                .apply(Point::new(rect.x, rect.y + rect.height)),
        ];
        if !pts
            .iter()
            .all(|point| point.x.is_finite() && point.y.is_finite())
        {
            self.region_errors
                .push(format!("canvas region {} has an invalid transform", id.0));
            return false;
        }
        let x0 = pts.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let y0 = pts.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let x1 = pts.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let y1 = pts.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        self.regions.push(p::Region {
            shape: None,
            transform: self.transform.m,
            local_rect: rect,
            id,
            rect: p::Rect {
                x: x0,
                y: y0,
                width: x1 - x0,
                height: y1 - y0,
            },
            properties: semantics.properties,
            description: semantics.description,
            disabled: semantics.disabled,
            hidden: semantics.hidden,
            focusable: semantics.focusable,
            focus_order: semantics.focus_order,
            focus_on_press: semantics.focus_on_press,
            role: semantics.role,
            name: semantics.name,
            value: semantics.value,
            actions: semantics.actions,
        });
        true
    }
}

/// A canvas text anchor's vertical alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextBaseline {
    #[default]
    Top,
    Middle,
    Alphabetic,
    Bottom,
}

impl Draw {
    /// A shaped region resolved against `rect`, following the current transform.
    /// Draw clips do not restrict the region's hit area.
    pub fn region_shape(&mut self, id: RegionId, rect: Rect, shape: &Shape) {
        if self.push_region(id, rect)
            && let Some(region) = self.regions.last_mut()
        {
            region.shape = Some(basic(rect, shape));
        }
    }
    /// A path region; `rect` supplies its accessibility bounds.
    /// Draw clips do not restrict the region's hit area.
    pub fn region_path(&mut self, id: RegionId, rect: Rect, path: &Path) {
        if self.push_region(id, rect)
            && let Some(region) = self.regions.last_mut()
        {
            region.shape = Some(path.wire());
        }
    }
}

fn plain_stroke(stroke: &Stroke) -> bool {
    stroke.cap == p::LineCap::Butt
        && stroke.join == p::LineJoin::Miter
        && stroke.miter_limit == 10.0
        && stroke.dash.is_empty()
        && stroke.dash_offset == 0.0
}

impl Draw {
    fn painted_shape(&mut self, shape: p::DrawingShape, paint: Paint, stroke: Option<&Stroke>) {
        self.cmds.push(p::SceneCmd::Shape(Box::new(p::ShapePaint {
            shape,
            paint: self.fill_of(&paint),
            stroke: stroke.cloned(),
        })));
    }
    pub fn fill_path(&mut self, path: &Path, paint: impl Into<Paint>) {
        self.painted_shape(path.wire(), paint.into(), None);
    }
    pub fn stroke_path(&mut self, path: &Path, stroke: &Stroke, paint: impl Into<Paint>) {
        self.painted_shape(path.wire(), paint.into(), Some(stroke));
    }
    /// Percentages in the shape and paint resolve against `rect`.
    pub fn fill_shape(&mut self, rect: Rect, shape: &Shape, paint: impl Into<Paint>) {
        self.painted_shape(basic(rect, shape), paint.into(), None);
    }
    pub fn circle(&mut self, center: Point, radius: impl Into<f64>, paint: impl Into<Paint>) {
        let r = radius.into() as f32;
        self.fill_shape(
            Rect::new(center.x - r, center.y - r, r * 2.0, r * 2.0),
            &Shape::circle(r),
            paint,
        );
    }
    pub fn polyline(
        &mut self,
        points: impl IntoIterator<Item = Point>,
        stroke: &Stroke,
        paint: impl Into<Paint>,
    ) {
        let mut points = points.into_iter();
        let Some(first) = points.next() else { return };
        let path = points.fold(Path::new().move_to(first), |path, p| path.line_to(p));
        self.stroke_path(&path, stroke, paint);
    }
}

impl Draw {
    /// A source rectangle uses natural image pixel coordinates (Canvas drawImage's nine arguments).
    pub fn image_with(&mut self, source: &Asset, rect: Rect, options: ImageOptions) {
        self.cmds.push(p::SceneCmd::ImagePart {
            source: source.clone(),
            rect,
            crop: options.source,
            opacity: options.opacity,
            pixelated: options.pixelated,
        });
    }
    pub fn pixels(&mut self, source: &Pixels, rect: Rect, options: ImageOptions) {
        let length = u64::from(source.width)
            .checked_mul(u64::from(source.height))
            .and_then(|pixels| pixels.checked_mul(4));
        if source.width == 0 || source.height == 0 || length != Some(source.rgba.len() as u64) {
            self.unsupported += 1;
            return;
        }
        self.cmds.push(p::SceneCmd::Pixels {
            image: p::Pixels {
                width: source.width,
                height: source.height,
                rgba: source.rgba.clone(),
            },
            rect,
            source: options.source,
            opacity: options.opacity,
            pixelated: options.pixelated,
        });
    }
    /// Draw `frame` of an image an external render engine shares
    /// (`kivact::gpu_interop`) into `rect`, as [`Draw::pixels`] draws its
    /// pixels: `options.source` is the part of the frame in its own
    /// pixels, `opacity` and `pixelated` apply alike. The frame is drawn
    /// by the GPU of the window the image was imported into, once the
    /// engine has signaled it ready; a frame not ready within the frame's
    /// wait, of an image not imported (or dropped, or lost), or in a
    /// window that draws on the CPU draws nothing, and no error is
    /// reported. A layer's color filter does not apply to it.
    ///
    /// The frame's RGBA8 values are composited as sRGB-encoded values
    /// (premultiplied alpha), as `Pixels` and CSS colors are: an engine
    /// that computes in linear light encodes to sRGB before it hands the
    /// frame over.
    #[cfg(feature = "gpu-interop")]
    pub fn gpu_frame(&mut self, frame: &GpuFrame, rect: Rect, options: ImageOptions) {
        self.cmds.push(p::SceneCmd::External {
            frame: frame.frame,
            rect,
            source: options.source,
            opacity: options.opacity,
            pixelated: options.pixelated,
        });
    }
}

impl Draw {
    /// Draw what `f` draws through `layer` (its transform, clip and group
    /// effects), which ends with `f`.
    pub fn layer(&mut self, layer: Layer, f: impl FnOnce(&mut Draw)) {
        let saved = self.transform;
        if let Some(t) = layer.transform {
            let m = Affine {
                m: t.matrix_for(self.size),
            };
            self.transform = m.then(self.transform);
            self.cmds.push(p::SceneCmd::PushTransform(m.m));
        }
        if let Some(rect) = layer.clip_rect {
            self.cmds.push(p::SceneCmd::PushClip(rect));
        }
        let group = layer.groups();
        if group {
            self.push_group(&layer);
        }
        f(self);
        if group {
            self.cmds.push(p::SceneCmd::PopLayer);
        }
        if layer.clip_rect.is_some() {
            self.cmds.push(p::SceneCmd::PopClip);
        }
        if layer.transform.is_some() {
            self.transform = saved;
            self.cmds.push(p::SceneCmd::PopTransform);
        }
    }

    fn push_group(&mut self, layer: &Layer) {
        let area = layer.area.unwrap_or(self.size.to_rect());
        let mut filter = layer.filter.clone();
        if let Some(shadow) = &layer.shadow {
            if shadow.spread != 0.0 || shadow.inset {
                self.unsupported += 1;
            }
            filter =
                filter.drop_shadow(shadow.offset.x, shadow.offset.y, shadow.blur, shadow.color);
        }
        self.cmds
            .push(p::SceneCmd::PushLayer(Box::new(p::DrawingLayer {
                area,
                clip: layer.clip.as_ref().map(|shape| basic(area, shape)),
                opacity: layer.opacity,
                blend: layer.blend,
                filter: filter_ops(&filter, &self.theme),
                mask: layer.mask.as_ref().map(|p| fill_layer(p, &self.theme)),
            })));
    }
}

type PaintFn = dyn Fn(&mut Draw, Size);

/// A style's `paint(..)` callback; clones share it and compare by identity.
#[derive(Clone)]
pub(crate) struct LocalPaint(Rc<PaintFn>);

impl LocalPaint {
    pub(super) fn new(f: impl Fn(&mut Draw, Size) + 'static) -> Self {
        Self(Rc::new(f))
    }

    pub(crate) fn draw(&self, draw: &mut Draw, size: Size) {
        (self.0)(draw, size);
    }
}

impl fmt::Debug for LocalPaint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LocalPaint(..)")
    }
}

impl PartialEq for LocalPaint {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
