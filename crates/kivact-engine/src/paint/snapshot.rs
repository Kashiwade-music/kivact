//! Role: Extract and CPU rasterize a mounted subtree using the window paint pipeline.

use super::state::PaintPass;
use crate::contract::SnapshotResult;
use crate::model::Model;
use crate::offscreen::{OffscreenError, OffscreenImage, OffscreenRenderer};
use kivact_types::{NodeId, Rect};

impl Model {
    /// Answer queued captures only from the host's settled layout boundary.
    pub(crate) fn snapshot_requests(&mut self, engine: &mut arvane::Engine) {
        for (request, node) in std::mem::take(&mut self.snapshots) {
            let result = match self.snapshot(node, engine) {
                None => SnapshotResult::Unmounted,
                Some(Ok(image)) => SnapshotResult::Pixels(kivact_types::Pixels {
                    width: image.width,
                    height: image.height,
                    rgba: image.rgba.into(),
                }),
                Some(Err(OffscreenError::Dropped(_))) => SnapshotResult::Unsupported,
                Some(Err(OffscreenError::Failed(message))) => {
                    SnapshotResult::Failed(message.to_string())
                }
            };
            self.push(crate::contract::HostEvent::SnapshotResult { request, result });
        }
    }

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
        let pass = PaintPass { scale: 1.0, size };
        let composition = self.compose_list(engine, &list, pass);
        self.report_dropped_draws();
        Some(OffscreenRenderer::new().render_composition(&composition, (width, height)))
    }
}
