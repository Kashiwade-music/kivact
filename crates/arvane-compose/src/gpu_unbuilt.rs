//! Role: The GPU executor's public names without feature `gpu`: every call fails.
//! No device exists in this build, so no call can succeed.
//!
//! [`GpuDevice`] is uninhabited. [`GpuContext`] and [`SurfaceConfigurer`] are not,
//! so that code which only the `gpu` build reaches does not look unreachable here:
//! their constructors are unreachable, and no value of them can be made.

use crate::{GpuReport, RenderError};
use arvane_protocol::{Composition, CpuFrame};
use kivact_types::ids::DrawableId;
use kivact_types::{ExternalFrame, ExternalImageId, GpuAdapter, SharedTextures};
use std::rc::Rc;
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

/// A GPU device cannot exist without the `gpu` feature.
pub enum GpuDevice {}

/// What configuring a window's surface needs. No value exists without the `gpu` feature.
#[derive(Clone)]
pub struct SurfaceConfigurer(());

/// A window's GPU context. No value exists without the `gpu` feature.
pub struct GpuContext(());

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

impl GpuDevice {
    /// Always fails: the `gpu` feature is not built.
    pub fn new(_env: &GpuEnv) -> Result<Self, RenderError> {
        Err(RenderError::NoAdapter(
            "built without the gpu feature".into(),
        ))
    }

    /// Unreachable: no device exists.
    pub fn describe(&self) -> String {
        match *self {}
    }

    /// Unreachable: no device exists.
    pub fn interop_adapter(&self) -> Option<GpuAdapter> {
        match *self {}
    }

    /// Unreachable: no device exists.
    pub fn failure(&self) -> Option<String> {
        match *self {}
    }

    /// Unreachable: no device exists.
    pub fn configurer(&self) -> SurfaceConfigurer {
        match *self {}
    }

    /// Unreachable: no device exists.
    pub fn surface<T>(&self, _target: T) -> Result<GpuSurface, RenderError>
    where
        T: Send + Sync + 'static,
    {
        match *self {}
    }

    /// Unreachable: no device exists.
    pub fn surface_maker<T: Send + Sync + 'static>(
        &self,
    ) -> impl FnOnce(T) -> Result<GpuSurface, RenderError> + Send + 'static {
        unbuilt_maker(self)
    }

    /// Unreachable: no device exists.
    pub fn import_external(
        &self,
        _id: ExternalImageId,
        _shared: SharedTextures,
    ) -> Result<(), RenderError> {
        match *self {}
    }

    /// Unreachable: no device exists.
    pub fn drop_external(&self, _id: ExternalImageId) {
        match *self {}
    }

    /// Unreachable: no device exists.
    pub fn take_missed_external(&self) -> Vec<ExternalFrame> {
        match *self {}
    }
}

impl SurfaceConfigurer {
    /// Unreachable: no configurer exists.
    pub fn configure(
        &self,
        _surface: &mut GpuSurface,
        _width: u32,
        _height: u32,
    ) -> Result<(), RenderError> {
        unreachable!("built without the gpu feature: no configurer exists")
    }
}

impl GpuContext {
    /// Unreachable: no device exists to share.
    pub fn new(gpu: Rc<GpuDevice>) -> Self {
        match *gpu {}
    }

    /// Unreachable: no context exists.
    pub fn device(&self) -> &Rc<GpuDevice> {
        unreachable!("built without the gpu feature: no context exists")
    }

    /// Nothing to release without a device.
    pub fn release_caches(&mut self) {
        unreachable!("built without the gpu feature: no context exists")
    }

    /// Unreachable: no context exists.
    pub fn present(
        &mut self,
        _surface: &mut GpuSurface,
        _composition: &Composition,
        _scale: f64,
        _physical: (u32, u32),
    ) -> Result<(Presentation, Duration, GpuReport), RenderError> {
        unreachable!("built without the gpu feature: no context exists")
    }

    /// Unreachable: no context exists.
    pub fn present_pixels(
        &mut self,
        _surface: &mut GpuSurface,
        _pixels: &[u32],
        _size: (u32, u32),
    ) -> Result<(Presentation, Duration), RenderError> {
        unreachable!("built without the gpu feature: no context exists")
    }

    /// Unreachable: no context exists.
    pub fn read_back(&self, _surface: &GpuSurface) -> Result<CpuFrame, RenderError> {
        unreachable!("built without the gpu feature: no context exists")
    }

    /// Unreachable: no context exists.
    pub fn render(
        &mut self,
        _composition: &Composition,
        _size: (u32, u32),
        _scale: f64,
    ) -> Result<CpuFrame, RenderError> {
        unreachable!("built without the gpu feature: no context exists")
    }

    /// Unreachable: no context exists.
    pub fn export_drawable(
        &mut self,
        _sink: DrawableId,
        _generation: u64,
        _adapter: &GpuAdapter,
        _size: (u32, u32),
    ) -> Result<SharedTextures, RenderError> {
        unreachable!("built without the gpu feature: no context exists")
    }

    /// Unreachable: no context exists.
    pub fn drop_export(&mut self, _sink: DrawableId) {
        unreachable!("built without the gpu feature: no context exists")
    }

    /// Unreachable: no context exists.
    pub fn export_losses(&self) -> Vec<arvane_protocol::ExportLoss> {
        unreachable!("built without the gpu feature: no context exists")
    }
}

/// Always fails: the `gpu` feature is not built, so no adapter is probed.
pub fn probe_adapter(_env: &GpuEnv) -> Result<String, RenderError> {
    Err(RenderError::NoAdapter(
        "built without the gpu feature".into(),
    ))
}

impl GpuSurface {
    /// Unreachable: no surface exists.
    pub fn keep_frames(&mut self) {
        match *self {}
    }
}

/// The surface maker of a device that cannot exist: never called.
fn unbuilt_maker<T: Send + Sync + 'static>(
    device: &GpuDevice,
) -> fn(T) -> Result<GpuSurface, RenderError> {
    match *device {}
}
