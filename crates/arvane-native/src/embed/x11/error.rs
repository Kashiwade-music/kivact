//! Role: Adds X11 embed context to platform failures without owning resources.

pub(super) fn platform(error: impl std::fmt::Display) -> arvane_shell::ShellError {
    arvane_shell::ShellError::new(format!("X11 embed: {error}"))
}

/// Whether the server rejected a request because `id` no longer exists. The
/// host destroying our parent destroys this window, which is not a failure.
pub(super) fn destroyed(error: &x11rb::errors::ReplyError, id: u32) -> bool {
    matches!(error, x11rb::errors::ReplyError::X11Error(e) if gone(e, id))
}

pub(super) fn gone(error: &x11rb::x11_utils::X11Error, id: u32) -> bool {
    error.error_kind == x11rb::protocol::ErrorKind::Window && error.bad_value == id
}
