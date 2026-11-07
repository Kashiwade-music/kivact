//! Role: Delivers the engine's drawable events to the drawable's export link.
//!
//! Each event runs on the link of its drawable when that drawable is alive
//! on this thread; events for other ids are stale and dropped.

use kivact_engine::contract::DrawableTexturesHandle;
use kivact_types::render::ExportedFrame;
use kivact_types::{DrawableId, Failure};

use super::registry::{all, with_link};
use crate::view::drawable_share::ExportLink;

/// A frame the compositor exported for a drawable.
pub(crate) fn frame(frame: &ExportedFrame) {
    with_link(frame.sink, |inbox, changed| {
        inbox.update(|link| {
            if let ExportLink::Open(open) = link {
                open.on_frame(frame.clone());
            }
        });
        changed.notify();
    });
}

/// The engine answered the textures request of `generation` for `id`.
pub(crate) fn textures(
    id: DrawableId,
    generation: u64,
    result: &Result<DrawableTexturesHandle, Failure>,
) {
    #[cfg(feature = "gpu-interop")]
    {
        let result = match result {
            Ok(handle) => match handle.take() {
                Some(textures) => Ok(textures),
                None => return,
            },
            Err(failure) => Err(failure.clone()),
        };
        with_link(id, |inbox, changed| {
            inbox.update(|link| {
                if let ExportLink::Open(open) = link {
                    open.on_textures(generation, result);
                }
            });
            changed.notify();
        });
    }
    #[cfg(not(feature = "gpu-interop"))]
    let _ = (id, generation, result);
}

/// The drawable `id` was resized: a GPU export asks for a new pool.
pub(crate) fn resized(id: DrawableId) {
    with_link(id, |inbox, changed| {
        inbox.update(|link| {
            if let ExportLink::Open(open) = link {
                let current = open.current();
                #[cfg(feature = "gpu-interop")]
                open.on_gpu_renew(current);
                #[cfg(not(feature = "gpu-interop"))]
                let _ = current;
            }
        });
        changed.notify();
    });
}

/// The GPU pool of drawable `id` of `generation` was lost with the device.
pub(crate) fn lost(id: DrawableId, generation: u64) {
    with_link(id, |inbox, changed| {
        inbox.update(|link| {
            if let ExportLink::Open(open) = link {
                #[cfg(feature = "gpu-interop")]
                open.on_gpu_renew(generation);
                #[cfg(not(feature = "gpu-interop"))]
                let _ = (open, generation);
            }
        });
        changed.notify();
    });
}

/// The window can export to GPU again: every waiting drawable asks anew.
pub(crate) fn gpu_available() {
    for registered in all() {
        registered.inbox.update(|link| {
            if let ExportLink::Open(open) = link {
                #[cfg(feature = "gpu-interop")]
                open.on_gpu_available();
                #[cfg(not(feature = "gpu-interop"))]
                let _ = open;
            }
        });
        registered.changed.notify();
    }
}
