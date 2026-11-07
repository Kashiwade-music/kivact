//! Role: The GPU executor's public names when feature `gpu` is not built: no device exists, so every call fails.

use crate::{ComposeError, GpuReport};
use kivact_types::ids::DrawableId;
use kivact_types::render::{Composition, CpuFrame};
use kivact_types::{ExternalFrame, ExternalImageId, GpuAdapter, SharedTextures};
use std::time::Duration;

/// How GPU devices are made (inert without the `gpu` feature).
#[derive(Clone, Copy, Debug)]
pub struct GpuEnv;

impl GpuEnv {
    /// The environment is not read without the `gpu` feature.
    pub fn from_env(_: kivact_types::GpuPreference) -> Self {
        Self
    }
}

/// A GPU executor cannot exist without the `gpu` feature.
pub enum GpuContext {}

/// A window surface cannot exist without the `gpu` feature.
pub enum GpuSurface {}

/// What [`GpuContext::present`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presentation {
    /// The frame is on its way to the screen.
    Shown,
    /// The surface had no buffer to give; the frame was not shown.
    Skipped,
}

impl GpuContext {
    /// Nothing to release without a device.
    pub fn release_caches(&mut self) {
        match *self {}
    }

    /// Always fails: the `gpu` feature is not built.
    pub fn new(_env: &GpuEnv) -> Result<Self, ComposeError> {
        Err(ComposeError::NoAdapter(
            "built without the gpu feature".into(),
        ))
    }

    /// Unreachable: no context exists.
    pub fn describe(&self) -> String {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn interop_adapter(&self) -> Option<GpuAdapter> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn failure(&self) -> Option<String> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn present(
        &mut self,
        _surface: &mut GpuSurface,
        _composition: &Composition,
        _scale: f64,
        _physical: (u32, u32),
    ) -> Result<(Presentation, Duration, GpuReport), ComposeError> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn present_pixels(
        &mut self,
        _surface: &mut GpuSurface,
        _pixels: &[u32],
        _size: (u32, u32),
    ) -> Result<(Presentation, Duration), ComposeError> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn read_back(&self, _surface: &GpuSurface) -> Result<CpuFrame, ComposeError> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn configure(
        &self,
        _surface: &mut GpuSurface,
        _width: u32,
        _height: u32,
    ) -> Result<(), ComposeError> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn surface<T>(&self, _target: T) -> Result<GpuSurface, ComposeError>
    where
        T: Send + Sync + 'static,
    {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn surface_maker<T: Send + Sync + 'static>(
        &self,
    ) -> impl FnOnce(T) -> Result<GpuSurface, ComposeError> + Send + 'static {
        unbuilt_maker(self)
    }

    /// Unreachable: no context exists.
    pub fn render(
        &mut self,
        _composition: &Composition,
        _size: (u32, u32),
        _scale: f64,
    ) -> Result<CpuFrame, ComposeError> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn import_external(
        &mut self,
        _id: ExternalImageId,
        _shared: SharedTextures,
    ) -> Result<(), ComposeError> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn drop_external(&mut self, _id: ExternalImageId) {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn take_missed_external(&mut self) -> Vec<ExternalFrame> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn export_drawable(
        &mut self,
        _sink: DrawableId,
        _generation: u64,
        _adapter: &GpuAdapter,
        _size: (u32, u32),
    ) -> Result<SharedTextures, ComposeError> {
        match *self {}
    }

    /// Unreachable: no context exists.
    pub fn drop_export(&mut self, _sink: DrawableId) {
        match *self {}
    }
}

impl GpuSurface {
    /// Unreachable: no surface exists.
    pub fn keep_frames(&mut self) {
        match *self {}
    }
}

/// The surface maker of an executor that cannot exist: never called.
fn unbuilt_maker<T: Send + Sync + 'static>(
    context: &GpuContext,
) -> fn(T) -> Result<GpuSurface, ComposeError> {
    match *context {}
}
