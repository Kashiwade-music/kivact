//! Role: Waits on the X connection, command eventfd and deadline timerfd.

use arvane_shell::ShellError;
use rustix::{
    event::{EventfdFlags, PollFd, PollFlags},
    time::{Itimerspec, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags, Timespec},
};
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

pub(super) struct Waiter {
    timer: OwnedFd,
}

impl Waiter {
    pub(super) fn new() -> Result<Self, ShellError> {
        let timer = rustix::time::timerfd_create(
            TimerfdClockId::Monotonic,
            TimerfdFlags::CLOEXEC | TimerfdFlags::NONBLOCK,
        )
        .map_err(super::error::platform)?;
        Ok(Self { timer })
    }

    pub(super) fn wait(
        &self,
        x: &impl AsFd,
        wake: &OwnedFd,
        remaining_ns: Option<u64>,
    ) -> Result<(), ShellError> {
        let ns = remaining_ns.map(|ns| ns.max(1)).unwrap_or(0);
        rustix::time::timerfd_settime(
            &self.timer,
            TimerfdTimerFlags::empty(),
            &Itimerspec {
                it_interval: Timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                it_value: Timespec {
                    tv_sec: (ns / 1_000_000_000) as _,
                    tv_nsec: (ns % 1_000_000_000) as _,
                },
            },
        )
        .map_err(super::error::platform)?;
        let mut fds = [
            PollFd::new(x, PollFlags::IN),
            PollFd::new(wake, PollFlags::IN),
            PollFd::new(&self.timer, PollFlags::IN),
        ];
        loop {
            match rustix::event::poll(&mut fds, None) {
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
        let _ = drain(&self.timer)?;
        Ok(())
    }
}
