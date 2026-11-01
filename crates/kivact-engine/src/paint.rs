//! Role: Conversion of the model's tree into an Arvane display list in paint order.
//!
//! Convert the model's tree into an Arvane display list in paint
//! order: the root's tree, then each layer (the fixed boxes outside the
//! overlays, then each overlay in the order they were attached with the
//! fixed boxes inside it) at its window origin, as one flat paint list.
//! Which nodes are drawn, where, at which opacity and in which layers is
//! the retained, culled paint list's (`list`); each frame draws their
//! current content.
//!
//! Invariants:
//! - What a node looks like is the model's evaluation (state
//!   styles, `Model::effective_paint` and `text_style`), sanitized per
//!   node; it is drawn in its own space at its untransformed layout box
//!   (`visual`), through the transform the paint list pushes around it
//!   and its subtree (`model::transform`: what it is hit and exposed
//!   with too).
//! - A hidden node draws nothing below it, overlays attached below it
//!   included (`Model::shown_overlays`). A node's opacity fades its
//!   colors, or, when it draws more than one thing, composites it and its
//!   subtree as one group (`list`); an overlay starts afresh.
//! - A node draws, in CSS order: its outer shadows, its backdrop, its
//!   background layers, its inset shadows, its border and its focus ring
//!   (`border`, `fill`), all in its layers (filters, clip path, mask,
//!   group opacity, blending: `effects`), and every color through its
//!   color filter ([`Tint`]).
//! - `DrawOver` follows the descendants. A clipping node clips its scenes,
//!   text and descendants to its padding box, rounded by its inner radii
//!   (both axes); overlays escape. The clip is a layer clipped to the
//!   rounded padding box when it has corners, else a rect clip.
//! - The steps of a node: its transform pushed; with a clip path, its
//!   focus ring straddling the clip path's outline beneath its layers; its
//!   layers opened; its box (shadows .. ring, when its own paint shows);
//!   its clip; its draw scene (inside the border); its content (a text, a
//!   text input's lines, an image); its descendants; its draw-over scene;
//!   its scrollbars (track, then thumb); then its clip, layers and
//!   transform closed. An unbalanced pop in a scene is ignored and what a
//!   scene leaves open is closed after it, so one scene never unbalances
//!   another's.
//! - The focus ring is drawn whenever the paint has one (the core puts it
//!   in the focus state style).
//! - Content that cannot be shaped, or images still loading or failed, are
//!   left out rather than failing the frame; images carry their alpha and
//!   color filter to the renderer.
//! - An `Image` node draws the store's pixels of its current asset:
//!   placed in its box (inside the border)
//!   by its `ImageFit`, clipped to the box when it overflows, rasterized
//!   for the drawn rect × the frame's scale × how much the transforms
//!   around it enlarge it (SVGs; `raster_zoom`), in its text color when
//!   monochrome. A scene image fills its rect.
mod list;
pub(crate) mod state;
mod text;

use list::{NO_MATRIX, NO_TRANSFORM};

use crate::model::Model;
use crate::model::interact::{text_geometry, visual};
use arvane::asset::{PixelSize, fit_image};
use kivact_lower::effects::Tint;
use kivact_lower::shape::BoxShape;
use kivact_lower::{border, effects, scene, shape};
use kivact_types::DrawCommand as D;
use kivact_wire::{HostEvent, NodeId, NodeType, PaintProps, Point, Rect, Rgba, Size};

use state::{Entry, Step};

impl Model {
    /// Build the frame's display list from the paint list (`list`): every
    /// listed node with its current content. Every shown image is drawn in
    /// it: a new frame of the asset store's recency.
    pub(crate) fn scene(&mut self, engine: &mut arvane::Engine) -> kivact_types::Scene {
        self.paint_list();
        self.assets.borrow_mut().begin_frame();
        let mut out = kivact_types::Scene::default();
        self.paint.scene_cache.borrow_mut().begin_frame();
        // How much the transforms around a node enlarge it, outermost
        // first: its images are rasterized for that much more.
        let mut zoom: Vec<f32> = Vec::new();
        for entry in &self.paint.list.entries {
            let at = zoom.last().copied().unwrap_or(1.0);
            match entry {
                Entry::Open(step) => {
                    let at = match self.paint.list.transforms.get(step.transform as usize) {
                        Some(t) => {
                            let z = at * t.max_scale();
                            zoom.push(z);
                            z
                        }
                        None => at,
                    };
                    self.paint
                        .raster
                        .set(self.paint.scale * scene::raster_zoom(at));
                    self.paint_open(step, engine, &mut out);
                }
                Entry::Close(step) => {
                    self.paint
                        .raster
                        .set(self.paint.scale * scene::raster_zoom(at));
                    self.paint_close(step, engine, &mut out);
                    if step.transform != NO_TRANSFORM {
                        zoom.pop();
                    }
                }
            }
        }
        self.paint.raster.set(self.paint.scale);
        for (node, count) in self.paint.dropped_draws.take() {
            self.push(HostEvent::DrawDropped { node, count });
        }
        out
    }

    /// The color matrix at `index` of the paint list.
    fn matrix(&self, index: u32) -> Option<kivact_types::ColorMatrix> {
        if index == NO_MATRIX {
            return None;
        }
        self.paint.list.matrices.get(index as usize).copied()
    }

    /// How `step`'s colors are drawn.
    fn tint(&self, step: &Step) -> Tint {
        Tint {
            alpha: step.alpha,
            matrix: self.matrix(step.matrix),
        }
    }

    /// Where `id` is painted when its layout box starts at `o`: its
    /// paint, its box and its canvas (inside the border).
    fn placed(&self, id: NodeId, o: Point) -> (PaintProps, Rect, Rect) {
        let layout = self.computed_layout(id).unwrap_or_default();
        let p = visual::paint(self, id);
        let bounds = Rect {
            x: o.x,
            y: o.y,
            width: layout.size.width,
            height: layout.size.height,
        };
        // Canvas scenes start inside the border.
        let canvas = visual::padding_box(bounds, &p);
        (p, bounds, canvas)
    }

    /// The layers `step` is composited in, outermost first: the clip path
    /// and mask over its painted box `bounds`, its group opacity, its
    /// blend mode and its layer filters.
    fn paint_layers(&self, step: &Step, p: &PaintProps, bounds: Rect) -> Vec<kivact_types::Layer> {
        let e = p.effects.as_deref();
        let outer = kivact_types::Layer {
            clip: e
                .and_then(|e| e.clip_path.as_ref())
                .map(|s| shape::clip_path(s, bounds)),
            mask: e.and_then(|e| e.mask.as_ref()).and_then(|m| {
                let area = bounds;
                // A mask still loading hides nothing yet.
                let origin = Point {
                    x: area.x,
                    y: area.y,
                };
                self.layer_paint(m, area, origin)
                    .map(|paint| kivact_types::Mask { rect: area, paint })
            }),
            ..kivact_types::Layer::default()
        };
        let opacity = if step.group { p.opacity } else { 1.0 };
        let parent = self.matrix(step.parent_matrix);
        let mut layers = effects::layers(p, outer, opacity, parent.as_ref());
        // As many as the list counted (it closes as many).
        layers.resize_with(usize::from(step.layers), kivact_types::Layer::default);
        layers
    }

    /// Paint `step`'s node up to its children: push its transform, open
    /// its layers, draw its box (when its own paint shows), then (inside
    /// its clip when it clips) its canvas scene and its content.
    fn paint_open(&self, step: &Step, engine: &mut arvane::Engine, out: &mut kivact_types::Scene) {
        let host = &self;
        let id = step.id;
        if let Some(t) = self.paint.list.transforms.get(step.transform as usize) {
            out.commands.push(D::PushTransform(*t));
        }
        let Some(n) = host.node(id) else {
            return;
        };
        let (p, bounds, canvas) = self.placed(id, step.origin);
        let tint = self.tint(step);
        let outline = BoxShape::of(bounds, &p.radius);
        let clip_path = p
            .effects
            .as_ref()
            .and_then(|e| e.clip_path.as_ref())
            .map(|s| shape::clip_path(s, bounds));
        if step.own
            && let (Some(ring), Some(clip)) = (&p.ring, &clip_path)
        {
            // Around the clip path, beneath the node's paint.
            border::focus_ring(out, &outline, Some(clip), ring, &tint);
        }
        if step.layers > 0 {
            for layer in self.paint_layers(step, &p, bounds) {
                out.commands.push(D::PushLayer(layer));
            }
        }
        if step.own {
            self.paint_box(step, &p, &outline, clip_path.is_some(), &tint, out);
        }
        if step.clips {
            // Closed as the list says (a rect clip or a rounded layer).
            let padding =
                BoxShape::of(bounds, &p.radius).inset(shape::border_widths(p.border.as_ref()));
            if step.rounded {
                out.commands.push(D::PushLayer(kivact_types::Layer {
                    clip: Some(padding.shape()),
                    ..kivact_types::Layer::default()
                }));
            } else {
                out.commands.push(D::PushClip(padding.rect()));
            }
        }
        if !step.own {
            return;
        }
        if let Some(scene) = &n.scenes()[0] {
            self.paint_scene(id, 0, scene, canvas, &tint, engine, out);
        }
        match n.kind() {
            _ if host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_some() =>
            {
                self.paint_input(id, bounds, step.clip, &tint, engine, out)
            }
            NodeType::Text => {
                let layout = host.computed_layout(id).unwrap_or_default();
                let area = visual::content_at(bounds, layout.content_box);
                self.paint_text(id, area, &tint, engine, out);
            }
            _ => {}
        }
        if host.editors[n.index.0]
            .as_deref()
            .map(|editor| editor.editor.state())
            .is_none()
            && let Some(asset) = n.image()
        {
            self.paint_image(id, n, asset, canvas, &tint, out);
        }
    }

    /// The node's own box: shadows, backdrop, background, border and ring
    /// (see the module invariants).
    fn paint_box(
        &self,
        step: &Step,
        p: &PaintProps,
        outline: &BoxShape,
        clip_path: bool,
        tint: &Tint,
        out: &mut kivact_types::Scene,
    ) {
        let knockout = !border::opaque(p.background.as_ref(), tint);
        border::shadows(out, outline, &p.shadows, false, knockout, tint);
        let parent = self.matrix(step.parent_matrix);
        if let Some(b) = effects::backdrop(p, outline.shape(), parent.as_ref()) {
            out.commands.push(D::Backdrop(b));
        }
        if let Some(fill) = &p.background {
            let area = outline.rect();
            let origin = Point {
                x: area.x,
                y: area.y,
            };
            self.fill(out, outline, fill, tint, origin);
        }
        border::shadows(out, outline, &p.shadows, true, false, tint);
        if let Some(b) = &p.border {
            border::border(out, outline, b, tint);
        }
        if let Some(ring) = &p.ring
            && !clip_path
        {
            border::focus_ring(out, outline, None, ring, tint);
        }
    }

    /// Paint `step`'s node after its children: its `DrawOver` scene, then
    /// pop its clip, its layers and its transform.
    fn paint_close(&self, step: &Step, engine: &mut arvane::Engine, out: &mut kivact_types::Scene) {
        let id = step.id;
        if step.own
            && let Some(n) = self.node(id)
            && let Some(scene) = &n.scenes()[1]
        {
            let (_, _, canvas) = self.placed(id, step.origin);
            let tint = self.tint(step);
            self.paint_scene(id, 1, scene, canvas, &tint, engine, out);
        }
        let bars = if step.own {
            self.scrollbar_bars(id)
        } else {
            [None, None]
        };
        if bars.iter().any(Option::is_some) {
            let props = self.scrollbar_style(id);
            let tint = self.tint(step);
            for bar in bars.into_iter().flatten() {
                for (rect, color) in [(bar.track, props.track), (bar.thumb, props.thumb)] {
                    let rect = Rect {
                        x: step.origin.x + rect.x,
                        y: step.origin.y + rect.y,
                        width: rect.width,
                        height: rect.height,
                    };
                    out.commands.push(D::FillShape {
                        shape: kivact_types::Shape::Rect(rect),
                        paint: kivact_types::Paint::Solid(tint.color(color)),
                    });
                }
            }
        }
        if step.clips {
            out.commands.push(if step.rounded {
                D::PopLayer
            } else {
                D::PopClip
            });
        }
        out.commands
            .extend(std::iter::repeat_n(D::PopLayer, usize::from(step.layers)));
        if step.transform != NO_TRANSFORM {
            out.commands.push(D::PopTransform);
        }
    }

    /// The image of `Image` node `id` showing `asset` in its box `canvas`
    /// (see the module invariants); nothing while it loads or after it
    /// failed.
    fn paint_image(
        &self,
        id: NodeId,
        n: &crate::tree::node::HostNode,
        source: &kivact_types::Asset,
        canvas: Rect,
        tint: &Tint,
        out: &mut kivact_types::Scene,
    ) {
        let mut assets = self.assets.borrow_mut();
        let Some(natural) = assets.natural(source) else {
            return;
        };
        let fit = n.image_data().and_then(|d| d.fit).unwrap_or_default();

        let monochrome = n.image_data().and_then(|d| d.monochrome).unwrap_or(false);
        // The node's text color, inherited, with its and its ancestors' states.
        let tint_color = monochrome.then(|| self.text_color(id));
        let placed = fit_image(fit, natural, canvas);
        let device = PixelSize::of(
            Size {
                width: placed.rect.width,
                height: placed.rect.height,
            },
            self.paint.raster.get(),
        );
        // An overflowing SVG is rasterized for the part the box shows.
        let Some((image, region)) =
            assets.pixels(source, device, placed.visible(canvas), tint_color)
        else {
            return;
        };
        drop(assets);
        if placed.clip {
            out.commands.push(D::PushClip(canvas));
        }
        out.commands.push(D::Image {
            image,
            rect: region.place(placed.rect),
            opacity: tint.alpha,
            color_matrix: tint.matrix,
        });
        if placed.clip {
            out.commands.push(D::PopClip);
        }
    }

    /// A text input: the lines of its value (the placeholder in its color,
    /// else at half opacity, while empty) that reach `clip` within its
    /// content box, scrolled to its caret, and while focused its selection,
    /// caret and composition underline on them. Its lines are shaped one at
    /// a time and shared with the caret, the pointer and the IME
    /// (`kivact-text`); its line colors only change the color of its glyph
    /// runs (a password and a placeholder have none), and are taken only
    /// as far as the lines drawn.
    fn paint_input(
        &self,
        id: NodeId,
        bounds: Rect,
        clip: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut kivact_types::Scene,
    ) {
        let host = &self;
        let Some(n) = host.node(id) else {
            return;
        };
        let props = host.text_style(id);
        let color = props.color;
        let layout = host.computed_layout(id).unwrap_or_default();
        let area = visual::content_at(bounds, layout.content_box);
        let line = text_geometry::line_box(host, id, engine, area);
        let left = area.x - text_geometry::input_scroll(host, id);
        let Some((mut lines, shown)) = host.edit_lines(id, engine) else {
            return;
        };
        let view = clip.intersect(&area);
        let rows = lines.rows(view.y - line.y, view.bottom() - line.y);
        lines.drawn(rows.clone());
        let place = |r: Rect| Rect {
            x: left + r.x,
            y: line.y + r.y,
            ..r
        };
        out.commands.push(D::PushClip(area));
        if host.focused()
            == Some(kivact_wire::FocusTarget {
                node: id,
                region: None,
            })
            && let Some(edit) = host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
        {
            let value = edit.value();
            // The shared selection bands of `range` on the drawn lines.
            let mut spans = |range: std::ops::Range<usize>, engine: &mut arvane::Engine| {
                let mut out = Vec::new();
                for i in rows.clone() {
                    let reached = lines
                        .content(i)
                        .is_some_and(|l| range.start <= l.end && range.end >= l.start);
                    if reached {
                        let bands =
                            lines.spans(value, i, (range.start, range.end), area.width, engine);
                        out.extend(bands.into_iter().map(place));
                    }
                }
                out
            };
            let fill = kivact_types::Paint::Solid(match props.more().selection_color {
                Some(own) => tint.color(own),
                None => tint.arvane(Rgba {
                    r: 0.2,
                    g: 0.45,
                    b: 0.85,
                    a: 0.3,
                }),
            });
            for rect in spans(edit.selection(), engine) {
                out.commands.push(D::Rect {
                    rect,
                    fill: fill.clone(),
                });
            }
            let marked = edit.marked().map(|m| spans(m, engine));
            out.commands.push(D::Rect {
                rect: place(lines.caret(value, edit.caret(), area.width, engine)),
                fill: kivact_types::Paint::Solid(
                    tint.color(props.more().caret_color.unwrap_or(color)),
                ),
            });
            if let Some(marked) = marked {
                for band in marked {
                    out.commands.push(D::Line {
                        from: Point {
                            x: band.x,
                            y: band.bottom(),
                        },
                        to: Point {
                            x: band.right(),
                            y: band.bottom(),
                        },
                        width: 1.0,
                        color: tint.color(color),
                    });
                }
            }
        }
        // Placeholders are drawn in their color, else at half opacity.
        let color = if lines.shows_value() {
            color
        } else {
            props
                .more()
                .placeholder_color
                .unwrap_or_else(|| color.with_alpha_mul(0.5))
        };
        let colors = n
            .input_data()
            .and_then(|data| data.colors.clone())
            .filter(|_| lines.shows_value() && !n.password());
        match &colors {
            Some(colors) => lines.color(shown, colors, rows.end),
            None => lines.uncolor(),
        }
        let base = kivact_lower::sanitize::color(color);
        for i in rows {
            let (x, y) = (
                left + lines.offset(shown, i, area.width, engine),
                line.y + i as f32 * lines.height(),
            );
            match colors
                .as_ref()
                .and_then(|_| lines.painted(shown, i, base, engine))
            {
                // Each run keeps its span's color, drawn through the tint.
                Some(painted) => push_runs(out, Some(&painted), x, y, |run| tint.arvane(run)),
                None => {
                    let color = tint.color(color);
                    let shaped = lines.shaped(shown, i, engine);
                    push_runs(out, shaped.as_deref(), x, y, |_| color);
                }
            }
        }
        out.commands.push(D::PopClip);
    }
}

/// Glyph runs of `shaped` at `(x, y)`, colored `color` of their shaped
/// color.
fn push_runs(
    out: &mut kivact_types::Scene,
    shaped: Option<&kivact_types::ShapedText>,
    x: f32,
    y: f32,
    color: impl Fn(kivact_types::Rgba) -> kivact_types::Rgba,
) {
    for run in shaped.iter().flat_map(|shaped| &shaped.runs) {
        out.commands.push(D::Glyphs {
            run: std::sync::Arc::clone(run),
            origin: Point { x, y },
            color: Some(color(run.color)),
        });
    }
}
