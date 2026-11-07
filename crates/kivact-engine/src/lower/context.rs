//! Role: What painting a scene or a fill needs besides the scene itself.
//!
//! What painting a scene or a fill needs besides the scene: the asset
//! store, raster scale, first failure, failure count and dropped-command count. A window
//! client may continue around resource failures; independent image output
//! can return them after applying its own dropped-command error priority.
//!
//! Invariants:
//! - A failure belongs to the command that was actually drawn: the context
//!   records the first resource that was asked for and is not usable (a
//!   failed or missing image, a text that cannot be shaped), never one that
//!   a command left out. A window keeps painting around a failure; image
//!   output returns it.
//! - A command whose values cannot be drawn (a non-finite coordinate, arc
//!   or gradient stop, ...) is dropped and counted, never skipped silently:
//!   a window reports the count to the client, image output returns it as
//!   an error.
//! - What is counted: a `PopClip`, `PopTransform` or `PopGroup` that does not
//!   close the innermost group its scene opened; a non-finite coordinate; a size, radius or stroke
//!   width that is non-finite or negative; a path whose points or arcs are not finite (or
//!   that draws before it moves), a text run whose origin or size is not
//!   finite, a fill Arvane cannot draw (a resolved gradient's stops or
//!   geometry, a pattern's tile, a mask's paint). The geometry is checked
//!   before the command asks for a resource (a path before its arcs are
//!   lowered and its paint resolved, an image's rect before its pixels), so
//!   a dropped command loads and fails nothing. The rects of `Pixels` and
//!   of an image part are not checked here.
//! - A gradient by coordinates (a canvas gradient) that `sanitize::fill`
//!   rejects (no stop, an offset not finite, outside `0..=1` or out of
//!   order, a point or angle not finite, a negative radius) paints nothing
//!   and is not counted.
//! - A window client can count per node and draw layer: the
//!   first frame that drops commands of a layer sends one
//!   `HostEvent::DrawDropped { node, count }`, and the layer reports again
//!   only after it painted once without dropping one.

use kivact_types::Size;
use std::cell::{Cell, RefCell};

use crate::lower::assets::ArvaneAssets;
use crate::lower::scene_cache::{Dependency, SceneCache};
use std::sync::Arc;

/// A UI-local, non-reentrant lowering operation borrowing resources and scale.
/// Callers must not hold an overlapping resource borrow while invoking it.
/// It records failures and drops; publishing them remains the caller's policy.
pub(crate) struct Context<'a> {
    pub(crate) assets: &'a RefCell<ArvaneAssets>,
    /// Device pixels per logical pixel of what is painted now.
    pub(crate) raster: Cell<f64>,
    pub(crate) scene_cache: &'a RefCell<SceneCache>,
    /// Every resource resolved so far, in order; a scene lowering keeps the ones it resolved.
    pub(crate) dependencies: RefCell<Vec<Dependency>>,
    dropped: Cell<u32>,
    failures: RefCell<Vec<kivact_types::Failure>>,
}

/// The completed lowering operation.
pub(crate) struct LowerReport {
    pub(crate) dropped: u32,
    pub(crate) failures: Vec<kivact_types::Failure>,
}

/// The failure state of a context at one point, to restore it or to tell
/// whether anything failed or was dropped since.
pub(crate) struct Checkpoint {
    dropped: u32,
    failures: Vec<kivact_types::Failure>,
}

impl<'a> Context<'a> {
    /// Borrow resources, take the device scale and the host-owned retained scene cache.
    /// Resource revisions invalidate cached payloads before this operation.
    /// The context and its operations cannot outlive either borrowed value.
    pub(crate) fn new(
        assets: &'a RefCell<ArvaneAssets>,
        raster: f64,
        scene_cache: &'a RefCell<SceneCache>,
    ) -> Self {
        scene_cache
            .borrow_mut()
            .revision(assets.borrow().revision());
        Self {
            assets,
            raster: Cell::new(raster),
            scene_cache,
            dependencies: RefCell::new(Vec::new()),
            dropped: Cell::new(0),
            failures: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            dropped: self.dropped.get(),
            failures: self.failures.borrow().clone(),
        }
    }

    pub(crate) fn restore(&self, checkpoint: Checkpoint) {
        self.dropped.set(checkpoint.dropped);
        *self.failures.borrow_mut() = checkpoint.failures;
    }

    /// Whether nothing failed and no command was dropped since `at`, however
    /// many failures an earlier part of the frame already kept.
    pub(crate) fn succeeded_since(&self, at: &Checkpoint) -> bool {
        self.dropped.get() == at.dropped && self.failures.borrow().len() == at.failures.len()
    }

    fn record(&self, dependency: Dependency) {
        self.dependencies.borrow_mut().push(dependency);
    }

    /// Keep classified failures in production order.
    pub(crate) fn fail(&self, error: kivact_types::Failure) {
        self.failures.borrow_mut().push(error);
    }

    /// Count a command dropped for values that cannot be drawn.
    pub(crate) fn drop_command(&self) {
        self.dropped.set(self.dropped.get().saturating_add(1));
    }

    /// Consume this operation and return its final report.
    pub(crate) fn finish(self) -> LowerReport {
        LowerReport {
            dropped: self.dropped.into_inner(),
            failures: self.failures.into_inner(),
        }
    }

    /// Why `asset` has no pixels now.
    fn unavailable(&self, asset: &kivact_types::Asset) {
        match self.assets.borrow().status(asset) {
            Some(arvane::asset::ImageState::Failed(error)) => self.fail(
                kivact_types::Failure::new(kivact_types::FailureKind::Resource, error.to_string()),
            ),
            Some(arvane::asset::ImageState::Pending) => self.fail(kivact_types::Failure::new(
                kivact_types::FailureKind::Unavailable,
                "image resource is not ready",
            )),
            _ => self.fail(kivact_types::Failure::new(
                kivact_types::FailureKind::Resource,
                "image resource pixels are unavailable",
            )),
        }
    }

    /// The natural size of `asset` (`None`: not loaded; a failure is kept).
    pub(crate) fn natural(&self, asset: &kivact_types::Asset) -> Option<Size> {
        let size = self.assets.borrow_mut().natural(asset);
        if let Some(size) = size {
            self.record(Dependency::Natural {
                asset: asset.clone(),
                size,
            });
        } else {
            self.unavailable(asset);
        }
        size
    }

    /// The pixels of the whole of `asset` at `size` (`None`: nothing to
    /// draw; a failure is kept unless `size` is empty).
    pub(crate) fn pixels(
        &self,
        asset: &kivact_types::Asset,
        size: arvane::asset::PixelSize,
    ) -> Option<(kivact_types::Pixels, arvane::asset::Region)> {
        self.pixels_in(asset, size, arvane::asset::Region::WHOLE, None)
    }

    fn pixels_in(
        &self,
        asset: &kivact_types::Asset,
        size: arvane::asset::PixelSize,
        region: arvane::asset::Region,
        tint: Option<kivact_types::Rgba>,
    ) -> Option<(kivact_types::Pixels, arvane::asset::Region)> {
        if size.is_empty() {
            return None;
        }
        let pixels = self
            .assets
            .borrow_mut()
            .pixels(asset, size, region, tint, 0);
        if let Some((image, _)) = &pixels {
            self.record(Dependency::Pixels {
                asset: asset.clone(),
                device: size,
                region,
                tint,
                width: image.width,
                height: image.height,
                pixels: Arc::downgrade(image.rgba.as_shared()),
            });
        } else {
            self.unavailable(asset);
        }
        pixels
    }

    pub(crate) fn resources_match(&self, dependencies: &[Dependency]) -> bool {
        dependencies.iter().all(|dependency| match dependency {
            Dependency::Natural { asset, size } => self.natural(asset) == Some(*size),
            Dependency::Pixels {
                asset,
                device,
                region,
                tint,
                width,
                height,
                pixels,
            } => self
                .pixels_in(asset, *device, *region, *tint)
                .is_some_and(|(image, _)| {
                    image.width == *width
                        && image.height == *height
                        && pixels.ptr_eq(&Arc::downgrade(image.rgba.as_shared()))
                }),
        })
    }
}
