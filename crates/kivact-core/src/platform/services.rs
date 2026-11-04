//! Role: Platform-specific view operations.
//!
//! Platform services (fullscreen, clipboard, file and folder dialogs,
//! opening and revealing paths) as free functions returning futures; run
//! one with `spawn` to send its failure to `on_error`. The test driver
//! scripts all of them (an in-memory clipboard, scripted dialogs, recorded
//! open and reveal requests), so they are testable.
//!
//! Invariants:
//! - A request is a `Writer::effect` with a `RequestId`, matched to the
//!   host's `EffectResult` by that id (`crate::view::requests`).
//! - The window service's `Err` arrives as `PlatformResult::Failed(reason)`
//!   and is an `ErrorKind::Platform` error; only a platform that lacks the
//!   service (`ShellError::unsupported`) answers `Unsupported`, an
//!   `ErrorKind::Unsupported` error.
//! - The file dialogs keep their order and their cancel: [`pick_files`]
//!   is one native dialog for several files, `Ok(None)` when cancelled and
//!   in the order the platform gives, beside the single [`pick_file`] and
//!   [`save_file`]. The public test driver (`Screen::respond_file_dialog`,
//!   `Screen::respond_files_dialog`) and Arvane's facade contract check
//!   cancel, error and order; components get no private path of their own.
//! - [`set_fullscreen`] sends the request, and `platform::fullscreen()`
//!   reads the host's state as it reports it.

use std::future::Future;
use std::panic::Location;
use std::path::PathBuf;

use kivact_types::{PlatformRequest, PlatformResult};

use crate::view::requests;
use crate::view::requests::platform_result;
use crate::view::state::EffectWaiter;
use kivact_reactive::{Owner, UiError, require_effectful};
use kivact_task::Completion;

pub use kivact_types::FileDialog;

/// Every platform request: the scope must be alive and the caller must
/// not be a memo or render effect.
#[track_caller]
fn request(owner: Owner, api: &'static str, req: PlatformRequest, waiter: EffectWaiter) {
    let rt = owner.runtime();
    require_effectful(api);
    requests::request_effect(&rt, req, waiter);
}

#[track_caller]
fn request_async(
    owner: Owner,
    api: &'static str,
    req: PlatformRequest,
) -> Completion<Result<PlatformResult, UiError>> {
    let slot = Completion::new();
    request(owner, api, req, EffectWaiter::Future(slot.clone()));
    slot
}

/// Send `req` on behalf of the nearest owner of the call.
#[track_caller]
fn ask(
    api: &'static str,
    req: PlatformRequest,
    at: &'static Location<'static>,
) -> Completion<Result<PlatformResult, UiError>> {
    request_async(Owner::ambient(api, at), api, req)
}

/// Request a content size in logical pixels, clamped to the App minimum.
/// A desktop window asks the OS for that size. Embedded windows ask
/// `Parent::on_resize_request` in physical pixels. A
/// refusal is a platform error; an absent callback is unsupported. Invalid
/// dimensions return a message error. Await the future to observe the result.
#[track_caller]
pub fn request_window_size(
    width: impl Into<f64>,
    height: impl Into<f64>,
) -> impl Future<Output = Result<(), UiError>> + 'static {
    let at = Location::caller();
    let owner = Owner::ambient("request_window_size", at);
    require_effectful("request_window_size");
    let (width, height) = (width.into(), height.into());
    let valid = [width, height]
        .into_iter()
        .all(|v| v.is_finite() && v > 0.0);
    let result = if valid {
        let mut size = kivact_types::Size {
            width: width,
            height: height,
        };
        if let Some(min) = crate::view::state::ViewState::of(&owner.runtime()).min_window_size() {
            size.width = size.width.max(min.width);
            size.height = size.height.max(min.height);
        }
        Ok(request_async(
            owner,
            "request_window_size",
            PlatformRequest::RequestWindowSize(size),
        ))
    } else {
        Err(UiError::new(
            kivact_reactive::ErrorKind::Message,
            "request_window_size dimensions must be finite and positive in f64",
            Some(at),
        ))
    };
    async move { result?.await.and_then(platform_result).map(|_| ()) }
}

/// Enter or leave borderless fullscreen on this window.
///
/// Like every service of this module, it sends the request at the call for
/// the nearest owner (setup, handler, task or effect); the request fails
/// with `Disposed` once that owner is gone, and the call is a located
/// `SetupOnly` misuse outside every owner. Run the future with `spawn` to
/// send a failure to the nearest `on_error`, or await it to handle the
/// failure yourself.
#[track_caller]
pub fn set_fullscreen(on: bool) -> impl Future<Output = Result<(), UiError>> + 'static {
    let slot = ask(
        "set_fullscreen",
        PlatformRequest::Fullscreen(on),
        Location::caller(),
    );
    async move { slot.await.and_then(platform_result).map(|_| ()) }
}

/// Override the window's reduced-motion preference. Pass `None` to follow
/// the system again. The override also applies to the host's transitions.
#[track_caller]
pub fn set_reduced_motion(
    override_value: Option<bool>,
) -> impl Future<Output = Result<(), UiError>> + 'static {
    let slot = ask(
        "set_reduced_motion",
        PlatformRequest::ReducedMotion(override_value),
        Location::caller(),
    );
    async move { slot.await.and_then(platform_result).map(|_| ()) }
}

/// Copies `text` to the system clipboard.
#[track_caller]
pub fn copy_text(text: impl Into<String>) -> impl Future<Output = Result<(), UiError>> + 'static {
    let slot = ask(
        "copy_text",
        PlatformRequest::CopyText(text.into()),
        Location::caller(),
    );
    async move { slot.await.and_then(platform_result).map(|_| ()) }
}

/// Reads text from the system clipboard; empty when it holds no text.
#[track_caller]
pub fn read_text() -> impl Future<Output = Result<String, UiError>> + 'static {
    let slot = ask("read_text", PlatformRequest::ReadText, Location::caller());
    async move {
        match slot.await.and_then(platform_result)? {
            PlatformResult::Text(t) => Ok(t),
            _ => Ok(String::new()),
        }
    }
}

/// `Ok(None)` when the user cancels.
#[track_caller]
pub fn pick_file(
    opts: FileDialog,
) -> impl Future<Output = Result<Option<PathBuf>, UiError>> + 'static {
    let slot = ask(
        "pick_file",
        PlatformRequest::PickFile(opts.clone()),
        Location::caller(),
    );
    async move {
        match slot.await.and_then(platform_result)? {
            PlatformResult::Path(p) => Ok(p),
            _ => Ok(None),
        }
    }
}

/// Select multiple files in one native dialog. `Ok(None)` when cancelled.
/// The returned order is the order supplied by the platform picker.
#[track_caller]
pub fn pick_files(
    opts: FileDialog,
) -> impl Future<Output = Result<Option<Vec<PathBuf>>, UiError>> + 'static {
    let slot = ask(
        "pick_files",
        PlatformRequest::PickFiles(opts.clone()),
        Location::caller(),
    );
    async move {
        match slot.await.and_then(platform_result)? {
            PlatformResult::Paths(paths) => Ok(paths),
            _ => Ok(None),
        }
    }
}

/// Select a folder in one native dialog. `Ok(None)` when the user
/// cancels. Of `opts`, `title` and `directory` apply; `file_name` and
/// `filters` do not apply to a folder and are ignored (they are still
/// checked like a file dialog's). The test host answers it from the
/// same script as the file dialogs (`Screen::respond_file_dialog`).
#[track_caller]
pub fn pick_folder(
    opts: FileDialog,
) -> impl Future<Output = Result<Option<PathBuf>, UiError>> + 'static {
    let slot = ask(
        "pick_folder",
        PlatformRequest::PickFolder(opts.clone()),
        Location::caller(),
    );
    async move {
        match slot.await.and_then(platform_result)? {
            PlatformResult::Path(p) => Ok(p),
            _ => Ok(None),
        }
    }
}

/// Shows a save dialog. `Ok(None)` when the user cancels.
#[track_caller]
pub fn save_file(
    opts: FileDialog,
) -> impl Future<Output = Result<Option<PathBuf>, UiError>> + 'static {
    let slot = ask(
        "save_file",
        PlatformRequest::SaveFile(opts.clone()),
        Location::caller(),
    );
    async move {
        match slot.await.and_then(platform_result)? {
            PlatformResult::Path(p) => Ok(p),
            _ => Ok(None),
        }
    }
}

/// Open the file or folder at `path` with the operating system's
/// default handler (the application associated with a file, the file
/// manager for a folder); a relative path is resolved against the
/// process's current directory. Windows and Linux have the service
/// (Explorer, `xdg-open`); elsewhere, macOS included, the request is
/// an `ErrorKind::Unsupported` error. A path that does not exist or a
/// handler that cannot be started is an `ErrorKind::Platform` error
/// naming the path. `Ok(())` means the handler was started: what it
/// does with the item afterwards is not reported. The test host records
/// the path (`Screen::opened_paths`) instead of starting anything.
#[track_caller]
pub fn open_path(path: impl Into<PathBuf>) -> impl Future<Output = Result<(), UiError>> + 'static {
    let slot = ask(
        "open_path",
        PlatformRequest::OpenPath(path.into()),
        Location::caller(),
    );
    async move { slot.await.and_then(platform_result).map(|_| ()) }
}

/// Show the file or folder at `path` selected in the operating
/// system's file manager: Explorer (`explorer /select,`) on Windows.
/// `xdg-open` cannot select an item, so on Linux the folder containing
/// it is opened instead. Errors, platforms and the test host as for
/// [`open_path`] (`Screen::revealed_paths`).
#[track_caller]
pub fn reveal_path(
    path: impl Into<PathBuf>,
) -> impl Future<Output = Result<(), UiError>> + 'static {
    let slot = ask(
        "reveal_path",
        PlatformRequest::RevealPath(path.into()),
        Location::caller(),
    );
    async move { slot.await.and_then(platform_result).map(|_| ()) }
}
