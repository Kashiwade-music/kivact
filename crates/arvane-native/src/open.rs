//! Role: Opening and revealing file paths.
//!
//! Opening a path with the OS default handler and revealing it in the OS
//! file manager: Explorer on Windows, `xdg-open` on Linux and the other
//! Unix systems but Apple's and Android. Elsewhere, macOS included, both
//! are [`ShellError::unsupported`] until the platform table lists them.
//!
//! Both start a helper program and return once it started: what the
//! handler does afterwards (an unknown file type, a closed window) is not
//! reported. Windows releases the child handle without waiting. Unix waits
//! only for a short-lived shell that backgrounds the handler, so the handler
//! is reaped by init and no library thread outlives the window.
//!
//! Invariants:
//! - An empty path, and a path that does not exist, is refused before
//!   anything is started.
//! - Relative paths are made absolute against the current directory, the
//!   form every file manager accepts.
//! - Every error names the path and the reason.

use std::path::Path;
use std::process::Command;

use arvane_shell::ShellError;

/// What to do with a path.
#[derive(Clone, Copy)]
enum Verb {
    /// Open it with the default handler.
    Open,
    /// Show it selected in the file manager.
    Reveal,
}

/// Open the file or folder at `path` with the OS default handler.
pub(crate) fn open_path(path: &Path) -> Result<(), ShellError> {
    start(Verb::Open, path)
}

/// Show the file or folder at `path` selected in the OS file manager.
pub(crate) fn reveal_path(path: &Path) -> Result<(), ShellError> {
    start(Verb::Reveal, path)
}

/// Check `path` and start the program that does `verb` with it.
fn start(verb: Verb, path: &Path) -> Result<(), ShellError> {
    let name = match verb {
        Verb::Open => "open",
        Verb::Reveal => "reveal",
    };
    let fail =
        |reason: String| ShellError::new(format!("cannot {name} {}: {reason}", path.display()));
    if path.as_os_str().is_empty() {
        return Err(fail("the path is empty".into()));
    }
    let absolute = std::path::absolute(path).map_err(|e| fail(e.to_string()))?;
    let Some(mut command) = command(verb, &absolute) else {
        return Err(ShellError::unsupported(format!(
            "cannot {name} {}: unsupported on this platform",
            path.display()
        )));
    };
    match absolute.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(fail("no such file or folder".into())),
        Err(e) => return Err(fail(e.to_string())),
    }
    let program = command.get_program().to_string_lossy().into_owned();
    #[cfg(windows)]
    return command
        .spawn()
        .map(drop)
        .map_err(|e| fail(format!("{program} could not be started: {e}")));
    #[cfg(not(windows))]
    command
        .status()
        .map(|_| ())
        .map_err(|e| fail(format!("{program} could not be started: {e}")))
}

/// Explorer with the quoted `path` (after `/select,` to reveal) as one
/// verbatim argument: Explorer does not follow the usual argument quoting,
/// and Windows paths cannot contain `"`.
#[cfg(windows)]
fn command(verb: Verb, path: &Path) -> Option<Command> {
    use std::os::windows::process::CommandExt;
    let mut arg = std::ffi::OsString::from(match verb {
        Verb::Open => "",
        Verb::Reveal => "/select,",
    });
    arg.push("\"");
    arg.push(path.as_os_str());
    arg.push("\"");
    let mut command = Command::new("explorer.exe");
    command.raw_arg(arg);
    Some(command)
}

/// `xdg-open` with the path. It cannot select an item: to reveal, it opens
/// the folder containing it (the path itself when it has no parent).
#[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "android")))]
fn command(verb: Verb, path: &Path) -> Option<Command> {
    let target = match verb {
        Verb::Open => path,
        Verb::Reveal => path.parent().unwrap_or(path),
    };
    let mut command = Command::new("sh");
    command
        .args(["-c", "xdg-open \"$1\" >/dev/null 2>&1 &", "sh"])
        .arg(target);
    Some(command)
}

/// Not implemented here (macOS among them).
#[cfg(not(any(
    windows,
    all(unix, not(target_vendor = "apple"), not(target_os = "android"))
)))]
fn command(_: Verb, _: &Path) -> Option<Command> {
    None
}
