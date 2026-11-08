//! Role: Applying the app's drawable requests (export asks and geometry) to the model.
//!
//! The requests are kept by drawable id (see `state`), so a request made
//! before the drawable is mounted still applies once it is.

use crate::contract::HostRequest;
use crate::model::Model;
use crate::model::drawables::state::ExportAsk;

/// Apply one drawable request of the app (`HostRequest::DrawableExport` or
/// `SetDrawableGeometry`); any other request is a caller's mistake.
pub(crate) fn apply_request(m: &mut Model, request: HostRequest) {
    match request {
        HostRequest::DrawableExport {
            id,
            generation,
            mode,
            scale,
        } => {
            m.drawables.ask(
                id,
                ExportAsk {
                    generation,
                    mode,
                    scale,
                },
            );
            // The export is composed by a frame; nothing else may be
            // changing, so the frame that starts it is asked for here.
            m.paint.request_redraw();
        }
        HostRequest::SetDrawableGeometry {
            id,
            generation,
            geometry,
        } => {
            m.drawables
                .set_geometry(id, generation, geometry.map(|g| g.flatten()));
            // The root's drawn map is its window map, so its bounds and its
            // accessibility transform follow the geometry.
            if let Some(root) = m.drawables.get(id).map(|entry| entry.root) {
                m.a11y.subtree(root);
                m.changes.moved();
            }
        }
        _ => debug_assert!(false, "only drawable requests reach the drawables"),
    }
}
