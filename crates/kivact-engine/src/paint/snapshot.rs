//! Role: Extract and CPU rasterize a mounted subtree using the window paint pipeline.

use crate::model::Model;
use crate::offscreen::{OffscreenError, OffscreenImage, OffscreenRenderer};
use kivact_types::{NodeId, Rect};

impl Model {
    pub(crate) fn snapshot(
        &mut self,
        id: NodeId,
        engine: &mut arvane::Engine,
    ) -> Option<Result<OffscreenImage, OffscreenError>> {
        let size = self.computed_layout(id)?.size;
        let width = size.width.ceil() as u32;
        let height = size.height.ceil() as u32;
        let list = self.subtree_paint_list(
            id,
            Rect {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            },
        );
        // The retained window list and raster scale remain the window's.
        let previous = std::mem::replace(&mut self.paint.list, list);
        let scale = self.paint.scale;
        self.paint.scale = 1.0;
        let scene = self.paint_scene_list(engine, true);
        self.paint.scale = scale;
        self.paint.raster.set(scale);
        self.paint.list = previous;
        Some(OffscreenRenderer::new().render_scene(&scene, (width, height)))
    }
}
