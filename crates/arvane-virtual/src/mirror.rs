//! Role: Adapt native mirror output without introducing native input.

use crate::window::{GpuMode, MirrorMode, VirtualOptions};
use crate::window::{Mirror, physical};
use arvane_native::mirror::{MirrorStats, MirrorWindow, MirrorWindowOptions, TestDisplay};
use arvane_shell::{Accessibility, GpuTarget, ShellError};
use std::cell::RefCell;
use std::sync::Arc;

struct MirrorBatch {
    width: u32,
    height: u32,
    pixels: Arc<[u32]>,
}
enum MirrorPending {
    Idle,
    Scheduled(MirrorBatch),
}
struct Adapter {
    window: MirrorWindow,
    pending: RefCell<MirrorPending>,
}
impl Adapter {
    fn flush(&self) {
        if let MirrorPending::Scheduled(batch) = self.pending.replace(MirrorPending::Idle) {
            self.window.present(batch.width, batch.height, batch.pixels);
        }
    }
}
impl Mirror for Adapter {
    fn present(&self, width: u32, height: u32, pixels: Arc<[u32]>) {
        self.pending.replace(MirrorPending::Scheduled(MirrorBatch {
            width,
            height,
            pixels,
        }));
        self.flush();
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
        self.flush();
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
    pub(crate) fn open_mirror(&self) -> Result<Option<Box<dyn Mirror>>, ShellError> {
        if self.mirror == MirrorMode::Off {
            return Ok(None);
        }
        let (width, height) = physical(self.size, self.scale);
        match TestDisplay::global().and_then(|display| {
            display.open(MirrorWindowOptions {
                title: self.title.clone(),
                width,
                height,
                scale: self.scale,
            })
        }) {
            Ok(window) => Ok(Some(Box::new(Adapter {
                window,
                pending: RefCell::new(MirrorPending::Idle),
            }))),
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
