//! Role: Wall-clock values shared by everything that ticks at one period.

use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use crate::driver::Runtime;
use crate::{Owner, Value};

/// One shared clock: the owner that holds its signal and timer task, and
/// how many live callers use it.
pub(crate) struct ClockEntry {
    every_ns: u64,
    owner: Owner,
    value: Value<SystemTime>,
    users: Cell<u32>,
}

/// The wall-clock time, updated every `every` (setup only). Calls with the
/// same period share one timer and one value; it stops when the last
/// component that asked for it is removed.
///
/// ```ignore
/// let now = clock(Duration::from_secs(1));
/// text(move || format_time(now.get()))
/// ```
#[track_caller]
pub fn clock(every: Duration) -> Value<SystemTime> {
    let at = Location::caller();
    let caller = Owner::setup("clock", at);
    let rt = caller.runtime();
    let every_ns = u64::try_from(every.as_nanos()).unwrap_or(u64::MAX).max(1);
    let found = rt
        .tasks()
        .clocks
        .borrow()
        .iter()
        .find(|c| c.every_ns == every_ns)
        .cloned();
    let entry = match found {
        Some(entry) => entry,
        None => {
            let owner = rt.root().child(at);
            let signal = owner.signal(SystemTime::now(), at);
            let period = Duration::from_nanos(every_ns);
            let _task = super::spawn::spawn_owned(
                &rt,
                owner,
                async move {
                    loop {
                        super::time::sleep(period).await;
                        signal.update(|t| *t = SystemTime::now());
                    }
                },
                at,
            );
            let entry = Rc::new(ClockEntry {
                every_ns,
                owner,
                value: signal.into(),
                users: Cell::new(0),
            });
            rt.tasks().clocks.borrow_mut().push(Rc::clone(&entry));
            entry
        }
    };
    entry.users.set(entry.users.get() + 1);
    let value = entry.value;
    let app = rt.id();
    caller.on_cleanup(move || {
        let users = entry.users.get() - 1;
        entry.users.set(users);
        if users == 0
            && let Some(rt) = Runtime::get(app)
        {
            rt.tasks()
                .clocks
                .borrow_mut()
                .retain(|c| !Rc::ptr_eq(c, &entry));
            entry.owner.dispose();
        }
    });
    value
}
