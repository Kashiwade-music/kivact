//! Role: Process-isolated real X11 child/waker descriptor lifetime regression.
#![cfg(target_os = "linux")]
use arvane::{
    Handler, NativeWindow, Presented, ShellControl, ShellError, ShellEvent, ShellWaker,
    Window as _,
    embed::{self, EmbedOptions, Embedder},
    test_display::TestParent,
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use ui_types::{DrawCommand, Paint, Rect, Rgba, Scene, Size};

#[derive(Default)]
struct Seen {
    waker: Option<ShellWaker>,
    shown: bool,
}
struct CpuHost(Arc<Mutex<Seen>>);
impl Handler for CpuHost {
    fn start(
        &mut self,
        window: &mut NativeWindow<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        self.0.lock().expect("store child waker").waker = Some(waker);
        window.request_redraw();
        Ok(ShellControl::default())
    }
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut NativeWindow<'_>,
    ) -> Result<ShellControl, ShellError> {
        if event == ShellEvent::Redraw {
            let Size { width, height } = window.size();
            let scene = Scene {
                commands: vec![DrawCommand::Rect {
                    rect: Rect {
                        x: 0.,
                        y: 0.,
                        width,
                        height,
                    },
                    fill: Paint::Solid(Rgba {
                        r: 1.,
                        g: 1.,
                        b: 1.,
                        a: 1.,
                    }),
                }],
            };
            let plan = window.frame_plan();
            let presented = window
                .present(&scene, plan)
                .expect("present a real CPU frame");
            self.0.lock().expect("record child presentation").shown |=
                matches!(presented, Presented::Shown { .. });
        }
        Ok(ShellControl::default())
    }
}

#[test]
fn retained_and_cloned_wakers_release_descriptors_after_child_close() {
    fn cycle(cycle: usize) -> [ShellWaker; 2] {
        let parent = TestParent::open(640, 480).expect("open a visible resource-test host");
        let seen = Arc::new(Mutex::new(Seen::default()));
        let output = seen.clone();
        let child = embed::open(
            EmbedOptions {
                parent: parent.parent(),
                size: Size {
                    width: 160.,
                    height: 100.,
                },
                scale: Some(1.),
                on_resize_request: None,
                prefer_gpu: false,
                gpu_preference: Default::default(),
            },
            move || Embedder::new(CpuHost(output)),
        )
        .expect("open a real child for retained-waker measurement");
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            // Snapshot observations before making a synchronous native capture.
            let shown = seen.lock().expect("read presentation status").shown;
            let capture = parent.capture();
            let white = capture.as_ref().is_ok_and(|frame| {
                let at = (20 * frame.width as usize + 20) * 4;
                frame.width > 20
                    && frame.height > 20
                    && frame.rgba.get(at..at + 4) == Some(&[255, 255, 255, 255][..])
            });
            if shown && white {
                break;
            }
            let failure = child.failure();
            assert!(failure.is_none(), "cycle {cycle} before close: {failure:?}");
            assert!(
                Instant::now() < deadline,
                "cycle {cycle} before close: shown={shown}, capture error={:?}, pixel={:?}",
                capture.as_ref().err(),
                capture.as_ref().ok().map(|frame| {
                    let at = (20 * frame.width as usize + 20) * 4;
                    (frame.width, frame.height, frame.rgba.get(at..at + 4))
                })
            );
            parent.pump(Duration::from_millis(5));
        }
        let waker = seen
            .lock()
            .expect("read native waker")
            .waker
            .clone()
            .expect("handler received its waker");
        let cloned = waker.clone();
        assert!(child.failure().is_none());
        drop(child);
        let deadline = Instant::now() + Duration::from_secs(10);
        while parent.capture().is_ok() {
            assert!(
                Instant::now() < deadline,
                "cycle {cycle} after close timed out: native child remains capturable"
            );
            parent.pump(Duration::from_millis(5));
        }
        parent.close();
        assert!(parent.is_closed());
        [waker, cloned]
    }
    fn descriptors() -> usize {
        std::fs::read_dir("/proc/self/fd")
            .expect("enumerate process descriptors")
            .count()
    }
    // Initialize rendering and native lazy resources before measuring closure.
    drop(cycle(0));
    let baseline = descriptors();
    let mut retained = Vec::new();
    for index in 0..20 {
        retained.extend(cycle(index + 1));
        // Retained notifications must be inert after the real child closes.
        for waker in &retained {
            waker.wake();
        }
        assert_eq!(
            descriptors(),
            baseline,
            "child descriptors leaked on cycle {index}"
        );
    }
    drop(retained);
    assert_eq!(descriptors(), baseline);
}
