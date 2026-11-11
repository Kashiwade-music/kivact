//! Role: The GPU device of windows opened and closed while the App runs.
//!
//! Reading `gpu_interop::adapter` asks the window the read belongs to for
//! its GPU device. Whether a device waits to start is the window's own (its
//! presenter's), so the ask lives and ends with the window: every window
//! opened again starts a device of its own when it reads its adapter, the
//! window that did not read it starts none, and asking again starts no
//! second device. A window on the CPU only starts none however often it is
//! opened and asks. The adapter is read in a text's closure, a pure
//! computation: asking for the device is allowed there.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::gpu_interop;
use kivact::prelude::*;
use kivact::test::{RenderOptions, Renderer, Screen};

use super::gpu_on_demand::{GRACE, cpu_forced, cpu_only, gpu_lane, on_demand};
use super::support::mount_with;

/// How many times a test opens and closes the window.
const ROUNDS: u32 = 3;
/// How many times a round makes the window read its adapter again.
const READS: u32 = 3;

/// The generations of the adapters the child window's text read, `None`
/// for a read without one, in order.
type Seen = Rc<RefCell<Vec<Option<u64>>>>;

/// A main window that opens the window "Child" while `open`. The child's
/// text reads the child's adapter ("gpu" or "cpu"), again whenever `reads`
/// changes, and notes each read in `seen`.
#[track_caller]
fn mount(options: RenderOptions) -> (Screen, Signal<bool>, Signal<u32>, Seen) {
    let seen: Seen = Rc::default();
    let noted = seen.clone();
    let (screen, (open, reads)) = mount_with(options, move || {
        let open = signal(false);
        let reads = signal(0_u32);
        let view = column((
            text("main"),
            window("Child", open).content(move || {
                let noted = noted.clone();
                text(move || {
                    reads.get();
                    let adapter = gpu_interop::adapter();
                    noted
                        .borrow_mut()
                        .push(adapter.as_ref().map(|adapter| adapter.generation));
                    if adapter.is_some() { "gpu" } else { "cpu" }.to_owned()
                })
            }),
        ));
        ((open, reads), view)
    });
    (screen, open, reads, seen)
}

/// Make the child window read its adapter `READS` times more.
fn read_again(screen: &Screen, reads: Signal<u32>) {
    for _ in 0..READS {
        reads.set(reads.get_untracked() + 1);
        screen.run_until_idle();
    }
}

/// The generation of the adapter the child window read last.
#[track_caller]
fn last_generation(seen: &Seen) -> Option<u64> {
    *seen
        .borrow()
        .last()
        .expect("the child window read its adapter")
}

/// Windows opened and closed `ROUNDS` times, started with `options`, never
/// start a device, however often they read their adapter.
#[track_caller]
fn windows_stay_on_cpu(options: RenderOptions) {
    let (screen, open, reads, seen) = mount(options);
    for round in 0..ROUNDS {
        open.set(true);
        screen.run_until_idle();
        let child = screen.window("Child");
        read_again(&screen, reads);
        std::thread::sleep(GRACE);
        screen.run_until_idle();
        assert_eq!(
            child.renderer(),
            Renderer::Cpu,
            "round {round}: a window on the CPU only starts no device"
        );
        child.get_by_text("cpu");
        open.set(false);
        screen.run_until_idle();
        assert!(!child.is_open(), "round {round}: the window closes");
    }
    assert_eq!(screen.renderer(), Renderer::Cpu, "nor does the main window");
    assert!(
        seen.borrow().iter().all(Option::is_none),
        "no read has an adapter: {:?}",
        seen.borrow()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Each window opened again starts a device of its own when it reads its
/// adapter; reading it again, while the device starts and after, makes no
/// other device; the main window, which never reads its adapter, starts
/// none.
#[test]
fn each_window_opened_again_starts_its_own_device_on_demand() {
    if !gpu_lane("each_window_opened_again_starts_its_own_device_on_demand") {
        return;
    }
    if cpu_forced() {
        return windows_stay_on_cpu(on_demand());
    }
    let (screen, open, reads, seen) = mount(on_demand());
    let mut generations = Vec::new();
    for round in 0..ROUNDS {
        open.set(true);
        screen.run_until_idle();
        let child = screen.window("Child");
        assert!(
            matches!(child.renderer(), Renderer::GpuStarting | Renderer::Gpu),
            "round {round}: reading the adapter starts the new window's device, not {:?}",
            child.renderer()
        );
        read_again(&screen, reads);
        child.wait_for_gpu();
        child.get_by_text("gpu");
        let generation = last_generation(&seen).expect("the ready device has an adapter");
        read_again(&screen, reads);
        assert_eq!(child.renderer(), Renderer::Gpu, "round {round}");
        assert_eq!(
            last_generation(&seen),
            Some(generation),
            "round {round}: reading the adapter again makes no other device"
        );
        generations.push(generation);
        open.set(false);
        screen.run_until_idle();
        assert!(!child.is_open(), "round {round}: the window closes");
    }
    assert!(
        generations.windows(2).all(|pair| pair[0] < pair[1]),
        "each window opened made a device of its own: {generations:?}"
    );
    assert_eq!(
        screen.renderer(),
        Renderer::Cpu,
        "the main window never read its adapter: it starts no device"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Windows on the CPU only (as with `ARVANE_RENDERER=cpu`) start no device,
/// however often they are opened and read their adapter.
#[test]
fn cpu_windows_start_no_device_however_often_they_ask() {
    if !gpu_lane("cpu_windows_start_no_device_however_often_they_ask") {
        return;
    }
    windows_stay_on_cpu(cpu_only());
}
