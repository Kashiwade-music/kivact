//! Role: Keeps GPU contracts available when GPU rendering is not built.

use crate::RenderError;
use kivact_types::Size;
use kivact_types::render::{CpuFrame, RenderScene};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct GpuEnv;
impl GpuEnv {
    pub fn from_env(_: kivact_types::GpuPreference) -> Self {
        Self
    }
}
pub enum GpuContext {}
pub enum GpuSurface {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presentation {
    Shown,
    Skipped,
}

impl GpuContext {
    pub fn release_caches(&mut self) {
        match *self {}
    }
    pub fn new(_env: &GpuEnv) -> Result<Self, RenderError> {
        Err(RenderError::NoAdapter(
            "built without the gpu feature".into(),
        ))
    }
    pub fn describe(&self) -> String {
        match *self {}
    }
    pub fn interop_adapter(&self) -> Option<kivact_types::GpuAdapter> {
        match *self {}
    }
    pub fn failure(&self) -> Option<String> {
        match *self {}
    }
    pub fn present(
        &mut self,
        _surface: &mut GpuSurface,
        _scene: &RenderScene,
        _size: Size,
        _scale: f64,
        _physical: (u32, u32),
    ) -> Result<(Presentation, Duration), RenderError> {
        match *self {}
    }
    pub fn present_pixels(
        &mut self,
        _surface: &mut GpuSurface,
        _pixels: &[u32],
        (_width, _height): (u32, u32),
    ) -> Result<(Presentation, Duration), RenderError> {
        match *self {}
    }
    pub fn read_back(&self, _surface: &GpuSurface) -> Result<CpuFrame, RenderError> {
        match *self {}
    }
    pub fn configure(
        &self,
        _surface: &mut GpuSurface,
        _width: u32,
        _height: u32,
    ) -> Result<(), RenderError> {
        match *self {}
    }
    pub fn surface<T>(&self, _target: T) -> Result<GpuSurface, RenderError>
    where
        T: raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle
            + Send
            + Sync
            + 'static,
    {
        match *self {}
    }
    pub fn surface_maker<T: 'static>(
        &self,
    ) -> impl FnOnce(T) -> Result<GpuSurface, RenderError> + Send + 'static + use<T> {
        // No window handles are read: an unbuilt GPU context has no values.
        surface_callback::<T>(self)
    }
    pub fn render(
        &mut self,
        _scene: &RenderScene,
        _size: Size,
        _scale: f64,
    ) -> Result<CpuFrame, RenderError> {
        match *self {}
    }
    pub fn import_external(
        &mut self,
        _id: kivact_types::ExternalImageId,
        _shared: kivact_types::SharedTextures,
    ) -> Result<(), RenderError> {
        match *self {}
    }
    pub fn drop_external(&mut self, _id: kivact_types::ExternalImageId) {
        match *self {}
    }
    pub fn take_missed_external(&mut self) -> Vec<kivact_types::ExternalFrame> {
        match *self {}
    }
}

impl GpuSurface {
    pub fn keep_frames(&mut self) {
        match *self {}
    }
}

fn surface_callback<T>(context: &GpuContext) -> fn(T) -> Result<GpuSurface, RenderError> {
    match *context {}
}
