//! Role: The Linux desktop's reduced-motion preference, read from the settings portal.
//!
//! Invariants: one watcher thread, started by the first [`watch`], reads
//! `org.freedesktop.portal.Settings` and keeps [`reduced_motion`] current. A change
//! wakes every event loop whose [`Watch`] is alive (`WakeReasons::ENVIRONMENT`),
//! which each turns into `ShellEvent::EnvironmentChanged`; dropping the `Watch` when
//! the loop ends unregisters it. Without a session bus or portal the preference stays off.

use arvane_shell::{ShellWaker, WakeReasons};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedValue;

const DESTINATION: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const INTERFACE: &str = "org.freedesktop.portal.Settings";
const GNOME_NAMESPACE: &str = "org.gnome.desktop.interface";
const KDE_NAMESPACE: &str = "org.kde.kdeglobals.KDE";

// xtask-allow(state-fields): cache: the reduced-motion preference last read from the portal
/// The reduced-motion preference last read from the portal.
static REDUCED_MOTION: AtomicBool = AtomicBool::new(false);

/// The wakers of the event loops that show the environment, by registration.
static WAKERS: Mutex<Vec<(u64, ShellWaker)>> = Mutex::new(Vec::new());

// xtask-allow(state-fields): identity: the next waker registration number
/// The next registration number.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// Starts the one watcher thread every loop shares.
static WATCHER: std::sync::Once = std::sync::Once::new();

/// Whether the desktop asks to reduce motion, as last read from the portal.
pub(crate) fn reduced_motion() -> bool {
    REDUCED_MOTION.load(Ordering::Acquire)
}

/// An event loop's registration for preference changes; dropping it unregisters the loop.
#[must_use = "the loop is woken only while the registration is held"]
pub(crate) struct Watch(u64);

impl Drop for Watch {
    fn drop(&mut self) {
        WAKERS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|(id, _)| *id != self.0);
    }
}

/// Register `waker` to be woken when the preference changes, for as long as the returned
/// [`Watch`] is held, and start the watcher thread on the first call.
pub(crate) fn watch(waker: ShellWaker) -> Watch {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    WAKERS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((id, waker));
    WATCHER.call_once(|| {
        let spawned = std::thread::Builder::new()
            .name("arvane-settings".into())
            .spawn(run);
        if let Err(error) = spawned {
            tracing::debug!(%error, "settings watcher thread not started");
        }
    });
    Watch(id)
}

fn run() {
    let Ok(connection) = Connection::session() else {
        tracing::debug!("no session bus; reduced motion stays off");
        return;
    };
    let Ok(proxy) = Proxy::new(&connection, DESTINATION, PATH, INTERFACE) else {
        tracing::debug!("no settings portal; reduced motion stays off");
        return;
    };
    // Subscribe before the first read so that no change in between is missed.
    let Ok(changes) = proxy.receive_signal("SettingChanged") else {
        tracing::debug!("settings portal sends no change signal; reduced motion is read once");
        refresh(&proxy);
        return;
    };
    refresh(&proxy);
    for _ in changes {
        refresh(&proxy);
    }
}

/// Read the preference again and wake the event loops when it changed.
fn refresh(proxy: &Proxy<'_>) {
    let reduced = read_reduced_motion(proxy);
    if REDUCED_MOTION.swap(reduced, Ordering::AcqRel) != reduced {
        let wakers = WAKERS.lock().unwrap_or_else(PoisonError::into_inner);
        for (_, waker) in wakers.iter() {
            waker.wake(WakeReasons::ENVIRONMENT);
        }
    }
}

/// Reduced motion when animations are disabled or their duration factor is zero. A preference
/// that cannot be read does not count, and when neither can be read the answer is `false`.
fn read_reduced_motion(proxy: &Proxy<'_>) -> bool {
    let animations = read_setting(proxy, GNOME_NAMESPACE, "enable-animations")
        .and_then(|value| bool::try_from(value).ok());
    let factor = read_setting(proxy, KDE_NAMESPACE, "AnimationDurationFactor")
        .and_then(|value| f64::try_from(value).ok());
    animations == Some(false) || factor == Some(0.0)
}

/// One setting from the portal: `ReadOne`, or `Read` where the portal lacks it.
fn read_setting(proxy: &Proxy<'_>, namespace: &str, key: &str) -> Option<OwnedValue> {
    proxy
        .call::<_, _, OwnedValue>("ReadOne", &(namespace, key))
        .or_else(|_| proxy.call::<_, _, OwnedValue>("Read", &(namespace, key)))
        .ok()
}
