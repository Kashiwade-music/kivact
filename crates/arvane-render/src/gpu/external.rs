//! Role: The images external render engines share with a GPU context, frame by frame.
//!
//! A context's imports (feature `gpu-interop`): each image is a pool of
//! textures and two timeline fences imported into this context's device
//! (`kivact-gpu-share`, whose crate documentation has the hand-off and
//! `kivact_types::SharedTextures` the engine's contract). An image belongs to
//! the context (the window) it was imported into; a scene of another
//! window that names its id draws nothing.
//!
//! A frame of the context ([`GpuContext::with_external`]):
//! 1. For each frame the scene names, the ready fence is read on the CPU;
//!    a frame not ready yet is waited for, all of them together at most
//!    [`READY_WAIT`] from the frame's start. A frame still not ready, one
//!    the engine may already have overwritten (its value is at most the
//!    release value signaled), or one of a texture the pool lacks is not
//!    drawn and noted once ([`External::take_missed`]); a frame noted is
//!    not waited for again. So the queue never waits on the GPU for a
//!    frame that has not arrived.
//! 2. Before the frame's submission, each texture drawn is acquired (a
//!    GPU-side wait for a value already reached, and the hand-off).
//! 3. After it, each is released, and the release fence is signaled to
//!    the smallest value the frame drew of the image, less one, when that
//!    is above the last value signaled (values only grow).
//!
//! Dropping an image (or the context: the window closed, the device
//! failed) signals the release fence to the last value the engine
//! reached or the context drew, so the engine is never left waiting.
//!
//! Invariants:
//! - Used serially on the context's thread, around the context's own
//!   submissions (the raw submissions need it).
//! - Every texture acquired in a frame is released in the same frame
//!   (`with_external`), whether the frame was submitted, skipped or
//!   failed; frames do not nest.

use crate::error::RenderError;
use kivact_types::ExternalFrame;
use kivact_types::render::RenderScene;
use vello_hybrid::TextureId;

#[cfg(feature = "gpu-interop")]
use kivact_types::render::RenderCmd;
#[cfg(feature = "gpu-interop")]
use kivact_types::{ExternalImageId, GpuAdapter, SharedTextures};
#[cfg(feature = "gpu-interop")]
use std::collections::HashMap;
#[cfg(feature = "gpu-interop")]
use std::time::{Duration, Instant};

/// The longest a frame waits on the CPU for the ready fences of the
/// external frames it draws.
#[cfg(feature = "gpu-interop")]
pub(crate) const READY_WAIT: Duration = Duration::from_millis(100);

/// The texture ids of external frames: apart from the images' (counted
/// from 1).
#[cfg(feature = "gpu-interop")]
const FIRST_ID: u64 = 1 << 63;

/// An external frame ready for the frame being drawn.
pub(crate) struct Bound {
    pub(crate) id: TextureId,
    pub(crate) view: wgpu::TextureView,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

/// The images imported into a context (see the module documentation).
#[derive(Default)]
pub(crate) struct External {
    #[cfg(feature = "gpu-interop")]
    images: HashMap<ExternalImageId, Image>,
    /// The frame being drawn ([`GpuContext::with_external`]), while one is.
    #[cfg(feature = "gpu-interop")]
    frame: Option<Frame>,
    /// Frames not drawn, to be noted.
    #[cfg(feature = "gpu-interop")]
    missed: Vec<ExternalFrame>,
    #[cfg(feature = "gpu-interop")]
    next: u64,
}

/// What one frame of the context draws of the images.
#[cfg(feature = "gpu-interop")]
#[derive(Default)]
struct Frame {
    /// The frames drawable in it.
    bound: HashMap<ExternalFrame, Bound>,
    /// The textures acquired for it.
    acquired: Vec<(ExternalImageId, usize)>,
    /// Its draws were submitted.
    submitted: bool,
}

/// One imported image.
#[cfg(feature = "gpu-interop")]
struct Image {
    import: kivact_gpu_share::import::Import,
    /// The textures' views and ids, by slot.
    views: Vec<(TextureId, wgpu::TextureView)>,
    width: u16,
    height: u16,
    /// The largest value drawn.
    highest: u64,
    /// The last value the release fence was signaled to.
    released: u64,
    /// The last value not drawn (noted).
    missed: Option<u64>,
}

impl External {
    /// The frame `frame`, when the frame being drawn can draw it.
    pub(crate) fn bound(&self, frame: &ExternalFrame) -> Option<&Bound> {
        #[cfg(feature = "gpu-interop")]
        return self.frame.as_ref()?.bound.get(frame);
        #[cfg(not(feature = "gpu-interop"))]
        {
            let _ = frame;
            None
        }
    }

    /// The frame being drawn was submitted (its draws reach the queue).
    pub(crate) fn submitted(&mut self) {
        #[cfg(feature = "gpu-interop")]
        {
            self.frame
                .as_mut()
                .expect("invariant: draws are submitted inside `GpuContext::with_external`")
                .submitted = true;
        }
    }

    /// The frames not drawn since the last call (each noted once).
    pub(crate) fn take_missed(&mut self) -> Vec<ExternalFrame> {
        #[cfg(feature = "gpu-interop")]
        return std::mem::take(&mut self.missed);
        #[cfg(not(feature = "gpu-interop"))]
        Vec::new()
    }

    /// Make the frames `scene` names ready to draw (steps 1 and 2 of the
    /// module documentation). Only [`GpuContext::with_external`] calls it.
    #[cfg(feature = "gpu-interop")]
    pub(super) fn begin(&mut self, scene: &RenderScene) -> Result<(), RenderError> {
        assert!(
            self.frame.is_none(),
            "invariant: frames of a context do not nest"
        );
        let open = self.frame.insert(Frame::default());
        if self.images.is_empty() {
            return Ok(());
        }
        let started = Instant::now();
        // The value each texture is acquired at (the largest drawn).
        let mut slots: Vec<(ExternalImageId, usize, u64)> = Vec::new();
        for command in &scene.commands {
            let RenderCmd::External { frame, .. } = command else {
                continue;
            };
            if open.bound.contains_key(frame) {
                continue;
            }
            let Some(image) = self.images.get_mut(&frame.image) else {
                // Not imported into this context: nothing is drawn.
                continue;
            };
            let slot = frame.slot as usize;
            let ready = slot < image.views.len()
                && frame.value > image.released
                && (image.import.ready_value() >= frame.value
                    || (image.missed != Some(frame.value)
                        && image.import.wait_ready(
                            frame.value,
                            READY_WAIT.saturating_sub(started.elapsed()),
                        )));
            if !ready {
                if image.missed != Some(frame.value) {
                    image.missed = Some(frame.value);
                    self.missed.push(*frame);
                }
                continue;
            }
            let (id, view) = image.views[slot].clone();
            open.bound.insert(
                *frame,
                Bound {
                    id,
                    view,
                    width: image.width,
                    height: image.height,
                },
            );
            match slots
                .iter_mut()
                .find(|(i, s, _)| *i == frame.image && *s == slot)
            {
                Some((_, _, value)) => *value = (*value).max(frame.value),
                None => slots.push((frame.image, slot, frame.value)),
            }
        }
        for (id, slot, value) in slots {
            let image = self
                .images
                .get_mut(&id)
                .expect("invariant: the frame's image is imported");
            image
                .import
                .acquire(slot, value)
                .map_err(|error| RenderError::Backend(error.to_string()))?;
            open.acquired.push((id, slot));
        }
        Ok(())
    }

    /// Give the textures of the frame back and signal what it drew
    /// (step 3 of the module documentation). Only
    /// [`GpuContext::with_external`] calls it.
    #[cfg(feature = "gpu-interop")]
    pub(super) fn end(&mut self) -> Result<(), RenderError> {
        let Some(open) = self.frame.take() else {
            return Ok(());
        };
        let mut result = Ok(());
        for (id, slot) in open.acquired {
            if let Some(image) = self.images.get_mut(&id)
                && let Err(error) = image.import.release(slot)
            {
                result = Err(RenderError::Backend(error.to_string()));
            }
        }
        if !open.submitted {
            return result;
        }
        // The smallest and largest value drawn, by image.
        let mut drawn: HashMap<ExternalImageId, (u64, u64)> = HashMap::new();
        for frame in open.bound.keys() {
            let entry = drawn
                .entry(frame.image)
                .or_insert((frame.value, frame.value));
            entry.0 = entry.0.min(frame.value);
            entry.1 = entry.1.max(frame.value);
        }
        for (id, (lowest, highest)) in drawn {
            let Some(image) = self.images.get_mut(&id) else {
                continue;
            };
            image.highest = image.highest.max(highest);
            let value = lowest - 1;
            if value > image.released {
                if let Err(error) = image.import.signal(value) {
                    result = Err(RenderError::Backend(error.to_string()));
                }
                image.released = value;
            }
        }
        result
    }

    /// Import `shared` as the image `id` into `device`, on the adapter
    /// `adapter` (an image already imported as `id` is dropped first).
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn import(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        adapter: &GpuAdapter,
        id: ExternalImageId,
        shared: SharedTextures,
    ) -> Result<(), RenderError> {
        self.drop_image(id);
        if !adapter.matches(&shared.adapter) {
            return Err(RenderError::External(format!(
                "the textures are on another adapter than the window's ({})",
                adapter.name
            )));
        }
        let import = kivact_gpu_share::import::Import::new(device, queue, shared)
            .map_err(|error| RenderError::External(error.to_string()))?;
        let (width, height) = import.size();
        let views = import
            .textures()
            .iter()
            .map(|texture| {
                self.next += 1;
                (
                    TextureId(FIRST_ID + self.next),
                    texture.create_view(&wgpu::TextureViewDescriptor::default()),
                )
            })
            .collect();
        self.images.insert(
            id,
            Image {
                import,
                views,
                width: u16::try_from(width).expect("invariant: the import checked the size"),
                height: u16::try_from(height).expect("invariant: the import checked the size"),
                highest: 0,
                released: 0,
                missed: None,
            },
        );
        Ok(())
    }

    /// Drop the image `id`, if imported, releasing every frame of it.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn drop_image(&mut self, id: ExternalImageId) {
        if let Some(image) = self.images.remove(&id) {
            release_all(image);
        }
    }
}

/// Signal the release fence past every frame `image` could still hold,
/// then drop it (waiting, bounded, for the queue).
#[cfg(feature = "gpu-interop")]
fn release_all(mut image: Image) {
    // A lost Direct3D 12 device reads every fence as `u64::MAX`.
    let reached = image.import.ready_value();
    let reached = if reached == u64::MAX { 0 } else { reached };
    let value = reached.max(image.highest);
    if value > image.released {
        // Best effort: a failed device signals nothing.
        let _ = image.import.signal(value);
    }
}

#[cfg(feature = "gpu-interop")]
impl Drop for External {
    fn drop(&mut self) {
        let _ = self.end();
        for (_, image) in self.images.drain() {
            release_all(image);
        }
    }
}

#[cfg(not(feature = "gpu-interop"))]
impl External {
    /// No imports: nothing to make ready.
    pub(super) fn begin(&mut self, _: &RenderScene) -> Result<(), RenderError> {
        Ok(())
    }

    /// No imports: nothing to give back.
    pub(super) fn end(&mut self) -> Result<(), RenderError> {
        Ok(())
    }
}
