//! Role: Window loop that runs the client's turns, input and frames on demand.
//!
//! The window loop: the client's turns (writing the tree through
//! `write::Writer`), input and frames over an Arvane window, scheduled on
//! demand.
//!
//! Client callbacks and model live on the UI thread; only wakers cross
//! threads (the client's and the asset store's worker, both through the
//! shell's waker). The store's finished work is applied before each turn
//! (`Model::poll_assets`).
//!
//! `run_native` opens an `arvane::Shell`
//! window at `HostOptions::window_size` (960×720 logical pixels without
//! one), which it does not validate again (kivact's `App::run` did, once),
//! and passes no renderer choice: whether and when Arvane starts the GPU,
//! falls back to the CPU or switches is Arvane's (its default prefers the
//! GPU). The title and minimum size arrive with the first revision's
//! `Writer::set_window` and are applied after the turn
//! (`Window::set_min_size`). The engine does not wrap `arvane::Window`
//! again (size, scale, clock, environment, capabilities, surface
//! generation, frame plan, present, IME, keyboard, cursor, accessibility,
//! clipboard, dialogs, open and reveal); the native window and the test
//! window go through the same `Host::begin`/`Host::handle`.
//!
//! The host keeps Arvane's text `Engine` in an `Rc<RefCell<..>>` shared
//! with kivact's `Reader` (a draw callback's `Draw::measure_text`, borrowed
//! for one call). During a layout pass the model's measuring borrows it
//! and lends kivact's measure functions a `LeafTextMeasurer` instead.
//!
//! Invariants:
//! - A pump runs turns while events are queued, at most 64 in a row
//!   (`TURN_LIMIT`, then `HostEvent::TurnsCut` and the window draws), each
//!   on a laid-out tree. After them it applies the window settings, allows
//!   the IME and sets its caret area while an editable text input has the
//!   focus, asks for the software keyboard when that changes (whatever
//!   moved the focus: a tap, a click, Tab, assistive technology; a no-op
//!   on desktops), hands the window the IME's text around the caret when
//!   it changed (`Model::ime_changed`, which the IME's queries are answered
//!   from), shows the cursor of the model's rule (`Model::cursor`) when it
//!   changed, asks for a redraw when a frame is due and publishes the
//!   accessibility tree (after the frame when the native window defers
//!   it). Input that arrives meanwhile is applied to the model at once and
//!   its turn deferred (`Host::defer`), the IME text and the cursor updated.
//! - Platform requests (`Writer::effect`) are carried out after the turn;
//!   those written during the shell's start (the client's start and the
//!   first pump's turns, `HostPhase::Starting`) wait for the first entry after
//!   it, which the host asks for at once (an embedding host is still inside
//!   its editor's open call and takes no size request or dialog then);
//!   the window's `Err` is reported as `PlatformResult::Failed(reason)`,
//!   or `PlatformResult::Unsupported` when the platform has no such
//!   service (`ShellError::is_unsupported`). A font is registered by the
//!   write itself.
//! - A frame is drawn from the window's frame plan (logical size, raster
//!   scale: images are rasterized at it) and handed to `Window::present`;
//!   which renderer draws it and how a failure is handled is Arvane's. The
//!   host only takes the result: `Shown` (timed in the frame log),
//!   `Skipped { retry_at_ns }` (no buffer: drawn again at that time, or at
//!   once when the window regains the focus), `Dropped` (the scene cannot
//!   be drawn: the picture stays until the next change), `SurfaceLost`
//!   (Arvane recovers its GPU resources and surface; layout and text
//!   measurement are kept; `HostFailure` is sent once and the frame is
//!   drawn again). `Err` means the
//!   window can present no more (three surface losses in a row) and ends
//!   the host with a failure.
//! - The engine has no platform branch, renderer limit or GPU state: OS,
//!   GPU and CPU differences (renderer and GPU fallback, frame budget,
//!   refresh period, click counts, occluded windows, environment variables)
//!   are Arvane's.
//! - GPU texture sharing (feature `gpu-interop`): the imports and drops a
//!   turn wrote (`Writer::import_external`, `Writer::drop_external`) are
//!   handed to the window after the turn, in order, as the platform
//!   effects are (the window exists from the client's start on), and each
//!   import is answered with `HostEvent::ExternalImported`. Each pump first
//!   reads the window's adapter (`Window::gpu_adapter`, the same for the
//!   native and the test window) and sends `HostEvent::GpuAdapterChanged`
//!   when it is not the one sent last (the client starts with `None`). The
//!   frames the window could not draw are noted with the renderer's other
//!   changes (`RendererChange::ExternalFrameMissed`, in the frame log).
mod client;
mod events;
mod launch;
pub(crate) mod options;
mod pump;
#[cfg(feature = "testing")]
pub mod testing;
pub(crate) mod write;

pub use client::{Boundary, Client, HostWaker};
pub(crate) use launch::new;
pub use launch::run_native;
