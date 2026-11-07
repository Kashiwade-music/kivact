//! Role: Model-aware fill, scene and text painting orchestration.

use super::state::PaintPass;
use crate::lower::context::{Context, ShaderTime};
use crate::lower::effects::Tint;
use crate::lower::fill::Boxes;
use crate::lower::glyphs::{Colors, push_text};
use crate::lower::shape::BoxShape;
use crate::model::Model;
use kivact_types::{DrawList, Fill, NodeId, Point, Rect};

impl Model {
    /// The painting context of `pass`: its assets, raster scale and shader clock.
    pub(super) fn painting(&self, pass: PaintPass) -> Context<'_> {
        let time = ShaderTime {
            seconds: self.frame.now() as f64 / 1e9,
            reduced: self.frame.reduced_motion(),
        };
        Context::new(&self.assets, pass.scale, &self.paint.scene_cache).timed(time)
    }

    /// Fill `boxes` with `fill` at `origin`; a shader layer is an item.
    pub(crate) fn fill(
        &self,
        out: &mut crate::lower::SceneOut,
        boxes: &Boxes,
        fill: &Fill,
        tint: &Tint,
        origin: Point,
        pass: PaintPass,
    ) {
        let context = self.painting(pass);
        context.fill_items(out, boxes, fill, tint, origin);
        context.finish();
    }

    /// Paint `scene`, the draw `layer` (0: draw, 1: draw-over) of node `id`,
    /// at `origin`; a text run without a family or a line height takes the
    /// node's. Commands dropped for values that cannot be drawn are queued
    /// for one report until the layer paints cleanly.
    #[expect(clippy::too_many_arguments, reason = "one node paint step")]
    pub(crate) fn paint_scene(
        &self,
        id: NodeId,
        layer: u8,
        scene: &std::sync::Arc<DrawList>,
        origin: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut crate::lower::SceneOut,
        pass: PaintPass,
    ) {
        let inherited = scene
            .cmds
            .iter()
            .any(|cmd| matches!(cmd, kivact_types::DrawCmd::Text(run) if crate::lower::glyphs::inherits(run)))
            .then(|| self.text_props(id));
        let context = self.painting(pass);
        out.scope(id, layer);
        context.paint_scene(scene, inherited.as_deref(), origin, tint, engine, out);
        let dropped = context.finish().dropped;
        if dropped == 0 {
            self.paint
                .dropped_reported
                .borrow_mut()
                .remove(&(id, layer));
        } else if self.paint.dropped_reported.borrow_mut().insert((id, layer)) {
            self.paint.dropped_draws.borrow_mut().push((id, dropped));
        }
    }
}

impl Model {
    /// The shaped paragraph of text node `id` in its content box `area`,
    /// wrapped at its width. Box paint clips to the same shaping (§9.4): the
    /// arvane cache serves both.
    pub(super) fn shape_node_text(
        &self,
        id: NodeId,
        area: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
    ) -> Option<kivact_types::ShapedTextSnapshot> {
        let n = self.node(id)?;
        let props = self.text_props(id);
        let color = tint.color(self.text_color(id));
        let mut style = crate::lower::assets::text_style(engine, &props);
        let content = n.content().unwrap_or("");
        let spans = n.spans();
        // One pixel of slack over the laid-out width keeps a pixel-rounded
        // box from wrapping its measured lines.
        let width = Some(area.width.max(0.0) + 1.0);
        let shaped = if spans.is_empty() {
            // Shaped without a color: a color change (a hover) reuses it.
            style.color = kivact_types::ComputedTextStyle::default().color;
            style.decoration.color = None;
            engine.shape_glyphs(content, &style, width)
        } else {
            style.color = color;
            style.decoration.color = props.decoration.color.map(|c| tint.color(c));
            let rich = crate::lower::conversion::spans(
                spans,
                style.font_size,
                |c| tint.color(c),
                |f| crate::lower::assets::family(engine, Some(f)),
            );
            let text = kivact_types::render::RichText {
                text: content,
                spans: &rich,
            };
            engine.shape_glyphs(text, &style, width)
        };
        shaped.ok()
    }

    /// The text node `id`'s paragraph in its content box `area`, wrapped
    /// at its width.
    pub(crate) fn paint_text(
        &self,
        id: NodeId,
        area: Rect,
        tint: &Tint,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut crate::lower::SceneOut,
    ) {
        let Some(n) = self.node(id) else {
            return;
        };
        let spans = n.spans();
        let props = self.text_props(id);
        let color = tint.color(self.text_color(id));
        // A missing font leaves the text undrawn.
        let Some(shaped) = self.shape_node_text(id, area, tint, engine) else {
            return;
        };
        let colors = if n.spans().is_empty() {
            let deco = match props.decoration.color {
                Some(own) => tint.color(own),
                None => color,
            };
            Colors {
                glyphs: Some(kivact_types::render::Paint::Solid(color)),
                decorations: Some(deco),
                ..Default::default()
            }
            .outlined(&props, tint)
        } else {
            Colors::default().outlined(&props, tint)
        };
        let origin = Point {
            x: area.x,
            y: area.y,
        };
        for band in &shaped.highlights {
            if let Some(fill) = spans
                .get(band.span)
                .and_then(|s| s.style.background.as_ref())
            {
                let rect = Rect {
                    x: band.rect.x + origin.x,
                    y: band.rect.y + origin.y,
                    ..band.rect
                };
                self.fill(
                    out,
                    &Boxes::uniform(BoxShape::Rect(rect)),
                    fill,
                    tint,
                    Point {
                        x: rect.x,
                        y: rect.y,
                    },
                    pass,
                );
            }
        }
        push_text(
            out,
            &shaped,
            origin,
            &colors,
            &props.shadows,
            tint,
            pass.scale,
        );
    }
}
