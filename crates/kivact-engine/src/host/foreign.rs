//! Role: The native child window of a window whose content another library
//! draws: create it with the window and report its size.

use super::pump::Host;
use crate::contract::{ForeignChange, HostEvent};
use crate::model::window::Content;
use arvane::Windows;
use kivact_types::cell::Sent;
use kivact_types::{SurfaceSize, WindowId};

impl Host {
    /// Ask the open window `id` for its native child and record it. Returns
    /// the child and its first surface size for `HostEvent::Foreign`.
    pub(super) fn prepare_foreign(
        &mut self,
        id: WindowId,
        windows: &mut Windows<'_>,
    ) -> Result<(kivact_types::NativeWindow, SurfaceSize), String> {
        let Some(window) = windows.get(id) else {
            return Err("the foreign window is not open".into());
        };
        let Some(child) = window.foreign_child() else {
            return Err("the platform did not create a native child window".into());
        };
        let size = surface_size(&window);
        if let Some(host_window) = self.app.borrow_mut().host_mut(id) {
            let mut reported = Sent::new();
            reported.update(size);
            host_window.content = Content::Foreign { child, reported };
        }
        Ok((child, size))
    }

    /// Report the foreign child's new extent when the window resized.
    pub(super) fn foreign_resized(&mut self, id: WindowId, windows: &mut Windows<'_>) {
        let Some(window) = windows.get(id) else {
            return;
        };
        let size = surface_size(&window);
        let changed = {
            let mut app = self.app.borrow_mut();
            let Some(host_window) = app.host_mut(id) else {
                return;
            };
            let Content::Foreign { reported, .. } = &mut host_window.content else {
                return;
            };
            reported.update(size).is_some()
        };
        if changed {
            self.app.borrow().push_event(HostEvent::Foreign {
                window: id,
                change: ForeignChange::Resized(size),
            });
        }
    }
}

/// The logical size, physical extent and scale the window reports now.
fn surface_size(window: &arvane::Window<'_>) -> SurfaceSize {
    let logical = window.size();
    let (physical_width, physical_height) = window.physical_size();
    SurfaceSize {
        logical,
        physical_width,
        physical_height,
        scale_factor: window.scale(),
    }
}
