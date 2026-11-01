//! Role: Arvane's embedded host contract, without Kivact or a virtual window.
#![cfg(any(windows, target_os = "linux"))]
use arvane::embed::{self, EmbedOptions, Embedder};
use arvane::test_display::TestParent;
use arvane::{
    Handler, NativeWindow, ShellControl, ShellError, ShellEvent, ShellWaker, Window as _,
};
use kivact_types::Size;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
static SERIAL: Mutex<()> = Mutex::new(());
#[derive(Default)]
struct Seen {
    events: Vec<ShellEvent>,
    sizes: Vec<(Size, f32)>,
    waker: Option<ShellWaker>,
    deadline: u64,
    timer: bool,
    wheel_settings: Option<kivact_types::MouseSettings>,
}
struct Host(Arc<Mutex<Seen>>);
impl Handler for Host {
    fn start(
        &mut self,
        window: &mut NativeWindow<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        assert_eq!(window.surface_generation(), Some(1));
        assert!(!window.fullscreen());
        assert!(
            window
                .set_fullscreen(true)
                .expect_err("embedded fullscreen is unsupported")
                .is_unsupported()
        );
        let deadline = window.now_ns() + 50_000_000;
        let mut seen = self
            .0
            .lock()
            .expect("handler observations are not poisoned");
        seen.wheel_settings = Some(window.environment().mouse);
        seen.waker = Some(waker);
        seen.deadline = deadline;
        window.request_redraw();
        Ok(ShellControl {
            exit: false,
            next_wake_ns: Some(deadline),
        })
    }
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut NativeWindow<'_>,
    ) -> Result<ShellControl, ShellError> {
        let mut seen = self
            .0
            .lock()
            .expect("handler observations are not poisoned");
        if event == ShellEvent::Resized {
            seen.sizes.push((window.size(), window.scale()));
            window.request_redraw();
        }
        if event == ShellEvent::Wake && window.now_ns() >= seen.deadline {
            seen.timer = true;
        }
        if event == ShellEvent::Redraw {
            let scene =
                crate::support::scene(vec![crate::support::fill(0., 0., 320., 240., 0xffffff)]);
            let plan = window.frame_plan();
            window
                .present(&scene, plan)
                .expect("embedded CPU frame presents");
        }
        seen.events.push(event);
        Ok(ShellControl {
            exit: false,
            next_wake_ns: (!seen.timer).then_some(seen.deadline),
        })
    }
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
    let child = embed::open(
        parent.parent(),
        EmbedOptions {
            size: Size {
                width: 160.,
                height: 100.,
            },
            scale: Some(1.),
            prefer_gpu: false,
            ..Default::default()
        },
        move || Embedder::new(Host(output)),
    )
    .unwrap();
    wait(&parent, || {
        seen.lock().unwrap().timer && parent.capture().is_ok()
    });
    assert_eq!(child.size(), (160, 100));
    assert_eq!(child.scale(), 1.);
    let waker = seen.lock().unwrap().waker.clone().unwrap();
    let before = seen.lock().unwrap().events.len();
    std::thread::spawn(move || waker.wake()).join().unwrap();
    wait(&parent, || {
        seen.lock().unwrap().events[before..].contains(&ShellEvent::Wake)
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
    assert!(child.failure().is_none());
    drop(child);
    wait(&parent, || parent.capture().is_err());
}

#[test]
fn native_wheel_scrolls_right_and_down_by_system_settings() {
    let _serial = SERIAL.lock().unwrap();
    let parent = TestParent::open(640, 480).unwrap();
    let seen = Arc::new(Mutex::new(Seen::default()));
    let output = seen.clone();
    let child = embed::open(
        parent.parent(),
        EmbedOptions {
            size: Size {
                width: 160.,
                height: 100.,
            },
            scale: Some(1.),
            prefer_gpu: false,
            ..Default::default()
        },
        move || Embedder::new(Host(output)),
    )
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
            .filter(|event| matches!(event, ShellEvent::Wheel { .. }))
            .count()
            == 2
    });
    let seen = seen.lock().unwrap();
    let settings = seen.wheel_settings.as_ref().unwrap();
    let distance = |count: u32| {
        if count > 100 {
            100.
        } else {
            count as f32 * 20.
        }
    };
    let wheels: Vec<_> = seen
        .events
        .iter()
        .filter_map(|event| match event {
            ShellEvent::Wheel { x, y } => Some((*x, *y)),
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
    assert!(child.failure().is_none());
}
