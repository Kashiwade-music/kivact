//! Role: Arvane's embedded host contract, without Kivact or a virtual window.
#![cfg(any(windows, target_os = "linux"))]
use arvane::embed::{self, EmbedOptions, Embedder};
use arvane::{EventTarget, Handler, ShellControl, ShellError, ShellEvent, ShellWaker, Windows};
use arvane_native::embed::TestParent;
use kivact_types::{Size, WindowId};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
static SERIAL: Mutex<()> = Mutex::new(());
#[derive(Default)]
struct Seen {
    events: Vec<ShellEvent>,
    sizes: Vec<(Size, f64)>,
    waker: Option<ShellWaker>,
    deadline: u64,
    timer: bool,
    wheel_settings: Option<kivact_types::PointerSettings>,
}
struct Host(Arc<Mutex<Seen>>);
impl Handler for Host {
    fn start(
        &mut self,
        windows: &mut Windows<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        let mut window = windows
            .get(WindowId::MAIN)
            .expect("invariant: the main window is open");
        assert_eq!(window.surface_generation(), Some(1));
        assert!(!window.fullscreen());
        assert_eq!(
            window.perform(kivact_types::PlatformRequest::Fullscreen(true)),
            kivact_types::PlatformResult::Unsupported
        );
        assert_eq!(
            window.perform(kivact_types::PlatformRequest::FocusWindow),
            kivact_types::PlatformResult::Unsupported,
            "an embedded window cannot be raised"
        );
        let deadline = window.now_ns() + 50_000_000;
        let mut seen = self
            .0
            .lock()
            .expect("handler observations are not poisoned");
        seen.wheel_settings = Some(window.environment().pointer);
        seen.waker = Some(waker);
        seen.deadline = deadline;
        window.request_redraw();
        Ok(ShellControl::WakeAt(deadline))
    }
    fn event(
        &mut self,
        _: EventTarget,
        event: ShellEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError> {
        let mut window = windows
            .get(WindowId::MAIN)
            .expect("invariant: the main window is open");
        let mut seen = self
            .0
            .lock()
            .expect("handler observations are not poisoned");
        if event == ShellEvent::Resized {
            seen.sizes.push((window.size(), window.scale()));
            window.request_redraw();
        }
        if matches!(event, ShellEvent::Wake(_)) && window.now_ns() >= seen.deadline {
            seen.timer = true;
        }
        if event == ShellEvent::Redraw {
            let plan = window.frame_plan();
            let composition = crate::support::commands_composition(
                vec![crate::support::fill(0., 0., 320., 240., 0xffffff)],
                plan.size,
            );
            window
                .present(&composition, plan)
                .expect("embedded CPU frame presents");
        }
        seen.events.push(event);
        Ok(if seen.timer {
            ShellControl::Wait
        } else {
            ShellControl::WakeAt(seen.deadline)
        })
    }
    fn close(&mut self, _: &mut Windows<'_>) {}
}
fn wait(parent: &TestParent, mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "embedded SPI operation timed out"
        );
        parent.pump(Duration::from_millis(5));
    }
}
#[test]
fn host_drives_redraw_worker_wake_deadline_and_geometry() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<embed::Embedded>();
    let _serial = SERIAL.lock().unwrap();
    let parent = TestParent::open(640, 480).unwrap();
    let seen = Arc::new(Mutex::new(Seen::default()));
    let output = seen.clone();
    let options = EmbedOptions {
        size: Size {
            width: 160.,
            height: 100.,
        },
        scale: Some(1.),
        prefer_gpu: false,
        ..Default::default()
    };
    let make = embed::driver(&options, move || Embedder::new(Host(output)));
    let child =
        arvane_native::embed::open(parent.parent(), options, embed::ChildHooks::default(), make)
            .map(embed::Embedded::new)
            .unwrap();
    wait(&parent, || {
        seen.lock().unwrap().timer && parent.capture().is_ok()
    });
    assert_eq!(child.size(), (160, 100));
    assert_eq!(child.scale(), 1.);
    let waker = seen.lock().unwrap().waker.clone().unwrap();
    let before = seen.lock().unwrap().events.len();
    std::thread::spawn(move || waker.wake(arvane::WakeReasons::CLIENT))
        .join()
        .unwrap();
    wait(&parent, || {
        seen.lock().unwrap().events[before..]
            .iter()
            .any(|event| matches!(event, ShellEvent::Wake(_)))
    });
    child.set_scale(2.);
    child.set_size(320, 200);
    wait(&parent, || {
        seen.lock().unwrap().sizes.iter().any(|(size, scale)| {
            *size
                == Size {
                    width: 160.,
                    height: 100.,
                }
                && *scale == 2.
        })
    });
    wait(&parent, || {
        child.size() == (320, 200) && child.scale() == 2.
    });
    // A geometry callback must precede the redraw requested from that callback.
    wait(&parent, || {
        let seen = seen.lock().unwrap();
        seen.events
            .iter()
            .position(|event| *event == ShellEvent::Resized)
            .is_some_and(|at| seen.events[at + 1..].contains(&ShellEvent::Redraw))
    });
    assert!(child.stop().is_none());
    drop(child);
    wait(&parent, || parent.capture().is_err());
}

#[test]
fn native_wheel_scrolls_right_and_down_by_system_settings() {
    let _serial = SERIAL.lock().unwrap();
    let parent = TestParent::open(640, 480).unwrap();
    let seen = Arc::new(Mutex::new(Seen::default()));
    let output = seen.clone();
    let options = EmbedOptions {
        size: Size {
            width: 160.,
            height: 100.,
        },
        scale: Some(1.),
        prefer_gpu: false,
        ..Default::default()
    };
    let make = embed::driver(&options, move || Embedder::new(Host(output)));
    let child =
        arvane_native::embed::open(parent.parent(), options, embed::ChildHooks::default(), make)
            .map(embed::Embedded::new)
            .unwrap();
    wait(&parent, || seen.lock().unwrap().wheel_settings.is_some());
    parent.move_to(23., 31.).unwrap();
    wait(&parent, || {
        seen.lock().unwrap().events.iter().any(|event| matches!(event,
        ShellEvent::Pointer(pointer) if pointer.position == kivact_types::Point { x: 23., y: 31. }
    ))
    });
    seen.lock().unwrap().events.clear();
    parent.wheel(0., 1.).unwrap();
    parent.wheel(1., 0.).unwrap();
    wait(&parent, || {
        seen.lock()
            .unwrap()
            .events
            .iter()
            .filter(|event| matches!(event, ShellEvent::Wheel(_)))
            .count()
            == 2
    });
    let seen = seen.lock().unwrap();
    let settings = seen.wheel_settings.as_ref().unwrap();
    let distance = |count: u32| {
        if count > 100 {
            100.
        } else {
            count as f64 * 20.
        }
    };
    let wheels: Vec<_> = seen
        .events
        .iter()
        .filter_map(|event| match event {
            ShellEvent::Wheel(kivact_types::WheelInput {
                delta: kivact_types::Point { x, y },
                ..
            }) => Some((*x, *y)),
            _ => None,
        })
        .collect();
    assert_eq!(
        wheels,
        [
            (0., distance(settings.wheel_lines)),
            (distance(settings.wheel_chars), 0.)
        ]
    );
    assert!(
        !seen.events.iter().any(|event| matches!(event,
            ShellEvent::Pointer(pointer) if pointer.phase == kivact_types::PointerPhase::Move
        )),
        "wheel at the existing mouse point must not duplicate pointer movement"
    );
    assert!(child.stop().is_none());
}

/// A handler whose App code panics while it starts: `open` returns an error
/// and keeps the panic's message instead of unwinding into the host.
#[test]
fn a_panic_while_the_handler_starts_is_returned_and_kept() {
    struct Panicking;
    impl Handler for Panicking {
        fn start(
            &mut self,
            _: &mut Windows<'_>,
            _: ShellWaker,
        ) -> Result<ShellControl, ShellError> {
            panic!("start exploded");
        }
        fn event(
            &mut self,
            _: EventTarget,
            _: ShellEvent,
            _: &mut Windows<'_>,
        ) -> Result<ShellControl, ShellError> {
            Ok(ShellControl::Wait)
        }
        fn close(&mut self, _: &mut Windows<'_>) {}
    }

    let _serial = SERIAL.lock().unwrap();
    let parent = TestParent::open(320, 240).unwrap();
    let panic = embed::PanicSlot::default();
    let options = EmbedOptions {
        size: Size {
            width: 160.,
            height: 100.,
        },
        scale: Some(1.),
        prefer_gpu: false,
        ..Default::default()
    };
    let make = embed::driver(&options, || Embedder::new(Panicking));
    let hooks = embed::ChildHooks {
        panic: panic.clone(),
        ..Default::default()
    };
    let result = arvane_native::embed::open(parent.parent(), options, hooks, make);
    assert!(result.is_err(), "the failed start is an error");
    assert!(panic.is_set(), "the panic message is kept");
    assert_eq!(panic.message().as_deref(), Some("start exploded"));
}
