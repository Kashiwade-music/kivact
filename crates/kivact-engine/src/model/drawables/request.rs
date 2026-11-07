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
        } => m.drawables.ask(
            id,
            ExportAsk {
                generation,
                mode,
                scale,
            },
        ),
        HostRequest::SetDrawableGeometry {
            id,
            generation,
            geometry,
        } => m
            .drawables
            .set_geometry(id, generation, geometry.map(|g| g.flatten())),
        _ => debug_assert!(false, "only drawable requests reach the drawables"),
    }
}
