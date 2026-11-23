//! Role: Object-safe window services and borrowed event-loop callbacks.
//!
//! Implementations confine window operations and callbacks to their UI thread.
//! Sizes and positions use logical pixels except explicit physical dimensions;
//! clocks and wake deadlines use monotonic nanoseconds. Rendering, frame policy,
//! readback and scene validation belong to the caller, not this interface.

use crate::events::ShellEvent;
use crate::ime::ImeRequest;
use crate::input::InjectedInput;
use crate::target::GpuTarget;
use crate::types::{ShellControl, ShellError, ShellWaker};
use arvane_protocol::PixelRect;
use kivact_types::{Cursor as ShellCursor, KeyCode, RequestId, Size, WindowId, WindowRequest};
use std::time::Duration;

/// The answer to [`ShellWindow::request_size`].
#[derive(Clone, Debug, PartialEq)]
pub enum SizeReply {
    /// The request is answered now.
    Now(kivact_types::PlatformResult),
    /// The window answers later with `ShellEvent::PlatformAnswer { request, .. }`.
    Later,
}

/// Whether a window can show a frame now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drawable {
    /// Yes, on the surface of `generation` (a new surface, a new generation).
    Yes {
        /// The surface's generation; the first surface is 1.
        generation: std::num::NonZeroU64,
    },
    /// The window is hidden by its host (an embedded editor); its
    /// surface is kept.
    Hidden,
    /// The platform destroyed the window's surface (a mobile suspension).
    NoSurface,
    /// The window has no pixels (minimized, or sized 0).
    NoPixels,
}

/// Whether an offered buffer still holds the frame presented last time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferAge {
    /// The buffer holds the previously presented frame.
    Previous,
    /// The buffer's content is unknown and must be fully written.
    Unknown,
}

/// Window services implemented by a platform backend or an in-memory shell.
/// This trait imposes no `Send` or `Sync` bound on the window itself.
pub trait ShellWindow {
    /// Execute a platform request synchronously on this window's UI thread.
    fn perform(&mut self, request: kivact_types::PlatformRequest) -> kivact_types::PlatformResult;

    /// Validate and replace the accepted semantics tree transactionally. Rejection retains the old tree.
    fn replace_accessibility(
        &mut self,
        tree: &arvane_protocol::SemanticsTree,
    ) -> Result<(), crate::a11y::error::AccessibilityError>;

    /// Validate and apply a semantics update transactionally. A new root requires a complete tree.
    fn update_accessibility(
        &mut self,
        update: &arvane_protocol::SemanticsUpdate,
    ) -> Result<(), crate::a11y::error::AccessibilityError>;

    /// Resolve an assistive-technology request against the window's
    /// accepted tree, as the platform's own request would be.
    fn request_accessibility_action(
        &self,
        node: u64,
        request: kivact_types::A11yRequest,
    ) -> Option<ShellEvent>;

    /// Current logical content extent in pixels.
    fn size(&self) -> Size;

    /// Request logical content dimensions from the platform, as one effect of
    /// `request`. Most windows answer at once. A window that must ask its host
    /// (an embedded editor) may answer `Later`; it then sends
    /// `ShellEvent::PlatformAnswer` with the same `request` on this window.
    fn request_size(&mut self, size: Size, request: RequestId) -> SizeReply;

    /// The App did not handle the last press of `code`. Desktop windows ignore
    /// it; an embedded editor may give it back to its host
    /// (`Parent::forward_unhandled_keys`).
    fn unhandled_key(&mut self, code: KeyCode) {
        let _ = code;
    }

    /// Current physical presentation width and height in pixels.
    fn physical_size(&self) -> (u32, u32);

    /// Physical pixels per logical pixel.
    fn scale(&self) -> f64;

    /// Monotonic window clock in nanoseconds; comparable to this loop's wake deadlines.
    fn now_ns(&self) -> u64;

    /// Backend-supported input, environment and service capabilities for this window.
    fn capabilities(&self) -> kivact_types::Capabilities;

    /// Whether the window can show a frame now, and on which surface generation.
    fn drawable(&self) -> Drawable;

    /// Request a future redraw event; repeated requests may be coalesced.
    fn request_redraw(&self);

    /// Set the window title.
    fn set_title(&self, title: &str);

    /// Set a minimum logical content extent; `None` removes it.
    fn set_min_size(&self, size: Option<Size>);

    /// Whether the user can resize the window.
    fn set_resizable(&self, resizable: bool);

    /// Current scale, refresh period, preferences, safe area, mouse settings and locale.
    fn environment(&self) -> kivact_types::Environment;

    /// Whether the window is currently fullscreen.
    fn fullscreen(&self) -> bool;

    /// Set the cursor shown over the content.
    fn set_cursor(&self, cursor: ShellCursor);

    /// Set the focused editor's IME state: `Some` enables IME input and
    /// positions it as the request asks, `None` disables IME input.
    fn set_ime(&self, ime: Option<ImeRequest>);

    /// The native child window of a foreign-content window, made when the
    /// window opened; `None` for any other window.
    fn foreign_child(&self) -> Option<kivact_types::NativeWindow> {
        None
    }

    /// The window's presentation surface (the presenter's only).
    fn surface(&mut self) -> &mut dyn WindowSurface;
}

/// The presentation side of a window, used by the presenter only.
pub trait WindowSurface {
    /// Offer exactly `width * height` row-major 0x00RRGGBB pixels to `draw`.
    /// `damage` lists the device pixels that changed since the last frame, or
    /// `None` for the whole frame. The callback receives the buffer's
    /// [`BufferAge`]: with `Previous`, the pixels outside `damage` already hold
    /// the last presented frame. The callback runs synchronously at most once.
    /// The callback returns `false` only before it has written any pixel, so the
    /// window can keep the last presented frame as the base of the next damaged frame.
    /// `false` discards its buffer without presentation. `Ok(true)` acknowledges
    /// a committed frame; `Ok(false)` means no buffer was offered or the callback
    /// declined it. A backend error is independent of an error captured by the
    /// caller's callback. Zero dimensions or no surface skip the callback.
    /// Dimensions must match the current physical surface or return an error.
    fn present_with(
        &mut self,
        width: u32,
        height: u32,
        damage: Option<&[PixelRect]>,
        draw: &mut dyn FnMut(&mut [u32], BufferAge) -> bool,
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
    fn with_gpu_target(&self, make: Box<dyn FnOnce(GpuTarget) + Send>) -> Result<(), ShellError>;

    /// Recover after a presentation failure; the next presentation may recreate resources.
    fn recover_surface(&mut self) -> Result<(), ShellError>;

    /// Whether the most recently committed presentation used the CPU surface. An offer that was declined
    /// or failed does not change it.
    fn shows_cpu_frame(&self) -> bool;

    /// Release CPU presentation resources before another presenter takes ownership.
    fn release_cpu_surface(&mut self);

    /// Wait for UI-thread work to finish, up to `limit`. Windows backends
    /// dispatch only sent messages while waiting, never input or posted
    /// messages; other backends sleep for one millisecond between polls.
    fn wait_until(&mut self, done: &mut dyn FnMut() -> bool, limit: Duration) -> bool;

    /// Where the presenter hands frames it read back for inspection;
    /// `None` for windows that keep none (every native window).
    fn frame_sink(&mut self) -> Option<&mut dyn FrameSink>;
}

/// A keeper of read-back frames (test windows).
pub trait FrameSink {
    /// Keep `frame`, a frame the presenter read back after showing it.
    fn keep(&mut self, frame: arvane_protocol::CpuFrame);
}

/// What one backend event is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventTarget {
    /// The loop as a whole: SaveState, Wake, Suspended, Resumed, MemoryWarning.
    Loop,
    /// One window: every other event.
    Window(WindowId),
}

impl EventTarget {
    /// The target of `event` for a loop whose window is `window`: `Loop` for
    /// SaveState, Wake, Suspended, Resumed and MemoryWarning, else `Window(window)`.
    pub fn of(event: &ShellEvent, window: WindowId) -> EventTarget {
        match event {
            ShellEvent::SaveState
            | ShellEvent::Wake(_)
            | ShellEvent::Suspended
            | ShellEvent::Resumed
            | ShellEvent::MemoryWarning => EventTarget::Loop,
            _ => EventTarget::Window(window),
        }
    }
}

/// The running loop as the handler sees it during one callback.
pub trait ShellLoop {
    /// An open window, or `None` once it is closed.
    fn window(&mut self, id: WindowId) -> Option<&mut dyn ShellWindow>;
    /// The open windows, ascending by id.
    fn windows(&self) -> Vec<WindowId>;
    /// Open a window. It becomes visible when the current callback returns.
    /// `ShellError::unsupported` where `Capabilities.multi_window` is false.
    fn open_window(&mut self, request: WindowRequest) -> Result<(), ShellError>;
    /// Destroy a window. No event for it follows, and input queued for it
    /// (`inject`) is dropped without `ShellEvent::Injected`. The handler
    /// releases what it holds for the window (its presenter) before calling this.
    fn close_window(&mut self, id: WindowId);
    /// Queue input for a window; it goes through that window's `WindowInput`
    /// like the platform's own input and is delivered right after the current
    /// callback returns, followed by `ShellEvent::Injected { request }` once
    /// all of it was delivered.
    fn inject(&mut self, id: WindowId, inputs: Vec<InjectedInput>, request: RequestId);
    /// The loop clock, comparable to `ShellControl::WakeAt`.
    fn now_ns(&self) -> u64;
    /// The waker of this loop (the same one `ShellHandler::start` received).
    fn waker(&self) -> ShellWaker;
    /// What the loop can do (`multi_window`, `window_placement`, ...).
    fn capabilities(&self) -> kivact_types::Capabilities;
}

/// Application callbacks borrowed for the synchronous event-loop invocation.
/// Neither the handler nor data it borrows needs a `'static` lifetime.
pub trait ShellHandler {
    /// Called once, after the backend opened the main window (`WindowId::MAIN`).
    fn start(
        &mut self,
        shell: &mut dyn ShellLoop,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError>;
    /// One ordered event. After [`ShellControl::Exit`] or an error, none follows.
    fn event(
        &mut self,
        target: EventTarget,
        event: ShellEvent,
        shell: &mut dyn ShellLoop,
    ) -> Result<ShellControl, ShellError>;
    /// Called exactly once for every handler whose `start` was called; the
    /// windows are still open during the call.
    fn close(&mut self, shell: &mut dyn ShellLoop);
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
    /// Open the main window and call `handler.start` before returning.
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
        /// Whether a window's redraw is still pending when the turn ends (asked and not delivered yet).
        redraw_requested: bool,
        woken: bool,
    },
    Exited,
    Failed(ShellError),
}
