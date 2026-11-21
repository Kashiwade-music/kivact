//! Role: Waits on the X connection and the command eventfd, with an optional deadline.

use arvane_shell::ShellError;
use rustix::event::{EventfdFlags, PollFd, PollFlags, Timespec};
use std::os::fd::{AsFd, OwnedFd};

pub(super) fn eventfd() -> Result<OwnedFd, ShellError> {
    rustix::event::eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)
        .map_err(super::error::platform)
}

pub(super) fn signal(fd: &OwnedFd) {
    // EAGAIN means another pending notification already guarantees a wake.
    let _ = rustix::io::write(fd, &1_u64.to_ne_bytes());
}

pub(super) fn drain(fd: &OwnedFd) -> Result<bool, ShellError> {
    match rustix::io::read(fd, &mut [0_u8; 8]) {
        Ok(_) => Ok(true),
        Err(rustix::io::Errno::AGAIN) => Ok(false),
        Err(error) => Err(super::error::platform(error)),
    }
}

/// Wait until the connection or the command eventfd is readable, or until
/// `remaining_ns` has passed (`None` waits without a deadline).
pub(super) fn wait(
    x: &impl AsFd,
    wake: &OwnedFd,
    remaining_ns: Option<u64>,
) -> Result<(), ShellError> {
    let timeout = remaining_ns.map(|ns| Timespec {
        tv_sec: (ns / 1_000_000_000) as i64,
        tv_nsec: (ns % 1_000_000_000) as i64,
    });
    let mut fds = [
        PollFd::new(x, PollFlags::IN),
        PollFd::new(wake, PollFlags::IN),
    ];
    loop {
        match rustix::event::poll(&mut fds, timeout.as_ref()) {
            Ok(_) => break,
            Err(rustix::io::Errno::INTR) => continue,
            Err(error) => return Err(super::error::platform(error)),
        }
    }
    if fds.iter().any(|fd| {
        fd.revents()
            .intersects(PollFlags::ERR | PollFlags::HUP | PollFlags::NVAL)
    }) {
        return Err(super::error::platform(
            "connection or notification fd closed",
        ));
    }
    Ok(())
}
