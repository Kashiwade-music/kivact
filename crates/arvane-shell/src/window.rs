//! Role: Object-safe window services and borrowed event-loop callbacks.
//!
//! Implementations confine window operations and callbacks to their UI thread.
//! Sizes and positions use logical pixels except explicit physical dimensions;
//! clocks and wake deadlines use monotonic nanoseconds. Rendering, frame policy,
//! readback and scene validation belong to the caller, not this interface.

use crate::events::ShellEvent;
use crate::ime::ShellImeText;
use crate::types::{ShellControl, ShellError, ShellWaker};
use kivact_types::{Cursor as ShellCursor, Rect, Size};

/// Pixel layout accepted by a CPU presentation surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuFormat {
    Xrgb8888,
}

/// Window services implemented by a platform backend or an in-memory shell.
/// This trait imposes no `Send` or `Sync` bound on the window itself.
pub trait ShellWindow {
    /// Execute a platform request synchronously on this window's UI thread.
    fn perform(&mut self, request: kivact_types::PlatformRequest) -> kivact_types::PlatformResult;

    /// Pixel format accepted by the CPU surface.
    fn cpu_format(&self) -> CpuFormat;

    /// Whether the backend retains presented frames for inspection.
    fn keeps_frames(&self) -> bool;

    /// Retain a frame read back by the presenter.
    fn keep_frame(&mut self, frame: kivact_types::render::CpuFrame);

    /// Wait for UI-thread work to finish, up to `limit`. Windows backends
    /// dispatch only sent messages while waiting, never input or posted
    /// messages; other backends sleep for one millisecond between polls.
    fn wait_until(&mut self, done: &mut dyn FnMut() -> bool, limit: std::time::Duration) -> bool;

    /// Recover after a presentation failure; the next presentation may recreate resources.
    fn recover_surface(&mut self) -> Result<(), ShellError>;

    /// Whether the most recently committed presentation used the CPU surface. An offer that was declined
    /// or failed does not change it.
    fn shows_cpu_frame(&self) -> bool;

    /// Release CPU presentation resources before another presenter takes ownership.
    fn release_cpu_surface(&mut self);

    /// Validate and replace the accepted semantics tree transactionally. Rejection retains the old tree.
    fn replace_accessibility(
        &mut self,
        tree: &kivact_types::render::SemanticsTree,
    ) -> Result<(), crate::a11y::error::AccessibilityError>;

    /// Validate and apply a semantics update transactionally. A new root requires a complete tree.
    fn update_accessibility(
        &mut self,
        update: &kivact_types::render::SemanticsUpdate,
    ) -> Result<(), crate::a11y::error::AccessibilityError>;

    /// Current logical content extent in pixels.
    fn size(&self) -> Size;

    /// Current physical presentation width and height in pixels.
    fn physical_size(&self) -> (u32, u32);

    /// Physical pixels per logical pixel.
    fn scale(&self) -> f64;

    /// Monotonic window clock in nanoseconds; comparable to this loop's wake deadlines.
    fn now_ns(&self) -> u64;

    /// Backend-supported input, environment and service capabilities for this window.
    fn capabilities(&self) -> kivact_types::Capabilities;

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
    fn environment(&self) -> kivact_types::Environment;

    /// Whether the window is currently fullscreen.
    fn fullscreen(&self) -> bool;

    /// Set the cursor shown over the content.
    fn set_cursor(&self, cursor: ShellCursor);

    /// Set the focused editor's content purpose and keyboard hint.
    fn set_ime_purpose(&self, purpose: kivact_types::InputPurpose);
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
    /// Called exactly once for every handler whose `start` was called, also when `start` failed or the
    /// handler asked to exit; no event follows it.
    fn close(&mut self, window: &mut dyn ShellWindow);

    /// Initialize state after the backend creates its window.
    fn start(
        &mut self,
        window: &mut dyn ShellWindow,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError>;
    /// Consume one ordered backend event and choose what the loop does next. After
    /// [`ShellControl::Exit`] or an error, no further event is delivered.
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut dyn ShellWindow,
    ) -> Result<ShellControl, ShellError>;
}

/// A UI-thread event loop consumed by one synchronous run.
pub trait Backend {
    /// Borrow the handler until exit, reporting backend or callback failures.
    /// A callback panic propagates out of `run`; a backend whose callbacks
    /// run under foreign frames carries it across them and resumes it after
    /// the loop.
    fn run(self: Box<Self>, handler: &mut dyn ShellHandler) -> Result<(), ShellError>;
}

/// A backend driven explicitly by its caller.
pub trait SteppedBackend {
    /// Create the window and call the handler before returning.
    fn start(
        self: Box<Self>,
        handler: Box<dyn ShellHandler>,
    ) -> Result<Box<dyn SteppedLoop>, ShellError>;
}

/// A running backend with independently controlled input, wake and redraw turns.
pub trait SteppedLoop {
    fn step(&mut self, step: Step) -> StepOutcome;
}

/// The event source to advance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Input,
    Wake,
    Redraw,
}

/// The state after a backend turn.
#[derive(Clone, Debug)]
pub enum StepOutcome {
    Idle {
        next_deadline: Option<u64>,
        redraw_requested: bool,
        woken: bool,
    },
    Exited,
    Failed(ShellError),
}
