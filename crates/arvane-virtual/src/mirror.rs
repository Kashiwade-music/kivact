//! Role: Adapt native mirror output without introducing native input.

use crate::window::{GpuMode, MirrorMode, VirtualOptions};
use crate::window::{Mirror, physical};
use arvane_native::mirror::{MirrorStats, MirrorWindow, MirrorWindowOptions, TestDisplay};
use arvane_shell::{Accessibility, GpuTarget, ShellError};
use kivact_types::WindowRequest;
use std::sync::Arc;

struct Adapter {
    window: MirrorWindow,
}
impl Mirror for Adapter {
    fn present(&self, width: u32, height: u32, pixels: Arc<[u32]>) {
        self.window.present(width, height, pixels);
    }
    fn accessibility(&self) -> &Accessibility {
        self.window.accessibility()
    }
    fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }
    fn resize(&self, width: u32, height: u32, scale: f64) {
        self.window.resize(width, height, scale);
    }
    fn sync(&self) {
        self.window.sync();
    }
    fn with_gpu_target(&self, make: Box<dyn FnOnce(GpuTarget) + Send>) -> Result<(), ShellError> {
        self.window.with_gpu_target(make)
    }
    fn gpu_presents(&self) {
        self.window.gpu_presents();
    }
}
impl VirtualOptions {
    pub(crate) fn open_mirror(
        &self,
        request: &WindowRequest,
    ) -> Result<Option<Box<dyn Mirror>>, ShellError> {
        if self.mirror == MirrorMode::Off {
            return Ok(None);
        }
        let (width, height) = physical(request.size, self.scale);
        match TestDisplay::global().and_then(|display| {
            display.open(MirrorWindowOptions {
                title: request.title.clone(),
                width,
                height,
                scale: self.scale,
            })
        }) {
            Ok(window) => Ok(Some(Box::new(Adapter { window }))),
            Err(error) if self.mirror == MirrorMode::Required || self.gpu == GpuMode::Required => {
                Err(error)
            }
            Err(_) => Ok(None),
        }
    }
}
/// Counters for the process-wide native mirror, if it can be started.
pub fn mirror_stats() -> Option<MirrorStats> {
    TestDisplay::global().ok().map(TestDisplay::stats)
}
