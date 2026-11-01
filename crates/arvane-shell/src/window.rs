//! Role: Object-safe window services and borrowed event-loop callbacks.
//!
//! Implementations confine window operations and callbacks to their UI thread.
//! Sizes and positions use logical pixels except explicit physical dimensions;
//! clocks and wake deadlines use monotonic nanoseconds. Rendering, frame policy,
//! readback and scene validation belong to the caller, not this interface.

use crate::events::ShellEvent;
use crate::ime::ShellImeText;
use crate::types::{ShellControl, ShellError, ShellWaker};
use ui_types::{Cursor as ShellCursor, Rect, Size};

/// Window services implemented by a platform backend or an in-memory shell.
/// This trait imposes no `Send` or `Sync` bound on the window itself.
pub trait ShellWindow {
    /// Wait for UI-thread work to finish, up to `limit`. Windows backends
    /// dispatch only sent messages while waiting, never input or posted
    /// messages; other backends sleep for one millisecond between polls.
    fn wait_until(&mut self, done: &mut dyn FnMut() -> bool, limit: std::time::Duration) -> bool;

    /// Write UTF-8 text to the window's clipboard service.
    fn copy_text(&mut self, text: &str) -> Result<(), ShellError>;

    /// Read UTF-8 text from the window's clipboard service.
    fn read_text(&mut self) -> Result<String, ShellError>;

    /// Request one file; cancellation returns `None`.
    fn pick_file(
        &mut self,
        request: &crate::services::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError>;

    /// Request multiple files; cancellation returns `None`.
    fn pick_files(
        &mut self,
        request: &crate::services::ShellFileDialog,
    ) -> Result<Option<Vec<std::path::PathBuf>>, ShellError>;

    /// Request a save path; cancellation returns `None`.
    fn save_file(
        &mut self,
        request: &crate::services::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError>;

    /// Request one folder; cancellation returns `None`. File filters do not apply.
    fn pick_folder(
        &mut self,
        request: &crate::services::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError>;

    /// Ask the platform to open a path. `Ok` acknowledges the request, not its completion.
    fn open_path(&mut self, path: &std::path::Path) -> Result<(), ShellError>;

    /// Ask the platform to reveal a path in its file manager.
    fn reveal_path(&mut self, path: &std::path::Path) -> Result<(), ShellError>;

    /// Recover after a presentation failure; the next presentation may recreate resources.
    fn recover_surface(&mut self) -> Result<(), ShellError>;

    /// Whether the most recently committed presentation used the CPU surface.
    fn shows_cpu_frame(&self) -> bool;

    /// Release CPU presentation resources before another presenter takes ownership.
    fn release_cpu_surface(&mut self);

    /// Stable window identity within this backend's live window domain; independent of surface generation.
    fn key(&self) -> u64;

    /// Validate and replace the accepted semantics tree transactionally. Rejection retains the old tree.
    fn replace_accessibility(
        &mut self,
        tree: &ui_types::SemanticsTree,
    ) -> Result<(), crate::a11y::error::AccessibilityError>;

    /// Validate and apply a semantics update transactionally. A new root requires a complete tree.
    fn update_accessibility(
        &mut self,
        update: &ui_types::SemanticsUpdate,
    ) -> Result<(), crate::a11y::error::AccessibilityError>;

    /// Whether assistive technology currently requests this window's accepted semantics.
    fn accessibility_active(&self) -> bool;

    /// Current logical content extent in pixels.
    fn size(&self) -> Size;

    /// Current physical presentation width and height in pixels.
    fn physical_size(&self) -> (u32, u32);

    /// Physical pixels per logical pixel.
    fn scale(&self) -> f32;

    /// Monotonic window clock in nanoseconds; comparable to this loop's wake deadlines.
    fn now_ns(&self) -> u64;

    /// Backend-supported input, environment and service capabilities for this window.
    fn capabilities(&self) -> ui_types::Capabilities;

    /// Current surface generation, or `None` while unavailable. Re-creation uses a new generation.
    fn surface_generation(&self) -> Option<u64>;

    /// Request a future redraw event; repeated requests may be coalesced.
    fn request_redraw(&self);

    /// Set the window title.
    fn set_title(&self, title: &str);

    /// Set a minimum logical content extent; `None` removes it.
    fn set_min_size(&self, size: Option<Size>);

    /// Current reduced-motion preference, consistent with `environment`.
    fn reduced_motion(&self) -> bool;

    /// Current scale, refresh period, preferences, safe area, mouse settings and locale.
    fn environment(&self) -> ui_types::Environment;

    /// Request a logical content size.
    fn request_size(&mut self, size: Size) -> Result<(), ShellError>;
    /// Request fullscreen state; platform policy may constrain the result.
    fn set_fullscreen(&self, fullscreen: bool) -> Result<(), ShellError>;

    /// Whether the window is currently fullscreen.
    fn fullscreen(&self) -> bool;

    /// Set the cursor shown over the content.
    fn set_cursor(&self, cursor: ShellCursor);

    /// Allow IME input for the focused editor; disabling releases its caret area.
    fn set_ime_allowed(&self, allowed: bool);

    /// Request software keyboard visibility where the backend supports it.
    fn set_soft_keyboard(&self, shown: bool);

    /// Position IME UI at this logical caret rectangle.
    fn set_ime_cursor_area(&self, area: Rect);

    /// Publish surrounding text and UTF-16 selection with logical character bounds; `None` clears it.
    fn set_ime_text(&self, text: Option<ShellImeText>);

    /// Current display period in nanoseconds, consistent with `environment`.
    fn frame_period_ns(&self) -> u64;

    /// Offer exactly `width * height` row-major 0x00RRGGBB pixels to `draw`.
    /// The callback runs synchronously at most once. `false` discards its
    /// buffer without presentation. `Ok(true)` acknowledges a committed frame;
    /// `Ok(false)` means no buffer was offered or the callback declined it.
    /// A backend error is independent of an error captured by the caller's
    /// callback. Zero dimensions or no surface skip the callback. Dimensions
    /// must match the current physical surface or return an error.
    fn present_with(
        &mut self,
        width: u32,
        height: u32,
        draw: &mut dyn FnMut(&mut [u32]) -> bool,
    ) -> Result<bool, ShellError>;

    /// Run `make` on the native target's owning UI thread.
    ///
    /// The owned callback may cross a display-thread queue, but raw handle
    /// access and surface creation must happen inside it on that thread.
    /// `Ok(())` means the callback ran or was accepted for execution; callers
    /// needing its result use their own response channel. A backend without
    /// a native target returns `ShellError::unsupported` without invoking it.
    /// Targets retain native lifetime, not permission to access handles from
    /// arbitrary worker threads. This operation exists without a GPU feature.
    fn with_gpu_target(
        &self,
        make: Box<dyn FnOnce(crate::target::GpuTarget) + Send>,
    ) -> Result<(), ShellError>;
}

/// Application callbacks borrowed for the synchronous event-loop invocation.
/// Neither the handler nor data it borrows needs a `'static` lifetime.
pub trait ShellHandler {
    /// Finish work needing this window's thread before the handler drops.
    /// Called once on close; no subsequent event is delivered.
    fn close(&mut self, window: &mut dyn ShellWindow);

    /// Initialize state after the backend creates its window.
    fn start(
        &mut self,
        window: &mut dyn ShellWindow,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError>;
    /// Consume one ordered backend event and choose its next wake deadline.
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut dyn ShellWindow,
    ) -> Result<ShellControl, ShellError>;
}

/// A UI-thread event loop consumed by one synchronous run.
pub trait ShellEventLoop {
    /// Borrow the handler until exit, reporting backend or callback failures.
    /// The backend must contain callback panics at its platform boundary.
    fn run(self: Box<Self>, handler: &mut dyn ShellHandler) -> Result<(), ShellError>;
}
