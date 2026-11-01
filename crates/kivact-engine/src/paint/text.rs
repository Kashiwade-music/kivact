//! Role: Model-aware fill, scene and text painting orchestration.

use crate::model::Model;
use kivact_lower::context::Context;
use kivact_lower::effects::Tint;
use kivact_lower::glyphs::{Colors, push_text};
use kivact_lower::shape::BoxShape;
use kivact_wire::{Fill, FillLayer, NodeId, Point, Rect, Scene};

impl Model {
    /// The window's painting context: its assets and raster scale. A
    /// window drops the failures the context keeps.
    fn painting(&self) -> Context<'_> {
        Context::new(&self.assets, &self.paint.raster, &self.paint.scene_cache)
    }

    pub(crate) fn fill(
        &self,
        out: &mut kivact_types::Scene,
        shape: &BoxShape,
        fill: &Fill,
        tint: &Tint,
        origin: Point,
    ) {
        self.painting().fill(out, shape, fill, tint, origin);
    }

    pub(crate) fn layer_paint(
        &self,
        layer: &FillLayer,
        area: Rect,
        origin: Point,
    ) -> Option<kivact_types::Paint> {
        self.painting().layer_paint(layer, area, origin)
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
        scene: &std::sync::Arc<Scene>,
        origin: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut kivact_types::Scene,
    ) {
        let inherited = scene
            .cmds
            .iter()
            .any(|cmd| matches!(cmd, kivact_wire::SceneCmd::Text(run) if kivact_lower::glyphs::inherits(run)))
            .then(|| self.text_props(id));
        let context = self.painting();
        context.paint_scene(scene, inherited.as_deref(), origin, tint, engine, out);
        let dropped = context.dropped();
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
    /// The text node `id`'s paragraph in its content box `area`, wrapped
    /// at its width.
    pub(crate) fn paint_text(
        &self,
        id: NodeId,
        area: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut kivact_types::Scene,
    ) {
        let Some(n) = self.node(id) else {
            return;
        };
        let props = self.text_props(id);
        let color = tint.color(self.text_color(id));
        let mut style = kivact_lower::assets::text_style(engine, &props);
        let content = n.content().unwrap_or("");
        let spans = n.spans();
        // One pixel of slack over the laid-out width keeps a pixel-rounded
        // box from wrapping its measured lines.
        let width = Some(area.width.max(0.0) + 1.0);
        let (shaped, colors) = if spans.is_empty() {
            // Shaped without a color: a color change (a hover) reuses it.
            style.color = kivact_types::TextStyle::default().color;
            let deco = match props.more().decoration.color {
                Some(own) => tint.color(own),
                None => color,
            };
            style.decoration.color = None;
            let colors = Colors {
                glyphs: Some(color),
                decorations: Some(deco),
            };
            (engine.shape_glyphs(content, &style, width), colors)
        } else {
            style.color = color;
            style.decoration.color = props.more().decoration.color.map(|c| tint.color(c));
            let rich = kivact_lower::conversion::spans(
                spans,
                style.font_size,
                |c| tint.color(c),
                |f| kivact_lower::assets::family(engine, Some(f)),
            );
            let text = kivact_types::RichText {
                text: content,
                spans: &rich,
            };
            let colors = Colors {
                glyphs: None,
                decorations: None,
            };
            (engine.shape_glyphs(text, &style, width), colors)
        };
        // A missing font leaves the text undrawn.
        let Ok(shaped) = shaped else {
            return;
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
                    &BoxShape::Rect(rect),
                    fill,
                    tint,
                    Point {
                        x: rect.x,
                        y: rect.y,
                    },
                );
            }
        }
        push_text(
            out,
            &shaped,
            origin,
            &colors,
            &props.more().shadows,
            tint,
            self.paint.raster.get(),
        );
    }
}
