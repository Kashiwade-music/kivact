//! Role: Worker frames wake canvas tasks and stop at the view owner's lifetime.

use std::sync::{Arc, mpsc};
use std::time::Duration;

use kivact::view::canvas;
use kivact::draw::{ImageOptions, Pixels};
use kivact::geometry::Rect;
use kivact::prelude::*;
use kivact::task::spawn;

use super::paint::pixel;
use super::support::mount;

fn frame(red: u8) -> Pixels {
    Pixels::new(1, 1, Arc::from([red, 0, 0, 255]))
}

#[test]
fn worker_frames_wake_without_input_and_receiver_closes_with_window() {
    let (tx, rx) = async_channel::bounded::<Pixels>(100);
    let (screen, received) = mount(move || {
        let image = signal(None::<Pixels>);
        let received = signal(0);
        spawn(async move {
            while let Ok(frame) = rx.recv().await {
                image.set(Some(frame));
                received.update(|count| *count += 1);
            }
        });
        (
            received,
            canvas(move |d, size| {
                if let Some(frame) = image.get() {
                    d.pixels(
                        &frame,
                        Rect::new(0., 0., size.width, size.height),
                        ImageOptions::default(),
                    );
                }
            })
            .style(style().size(32, 32)),
        )
    });
    // The receive task is already pending. No pointer, key or redraw input
    // follows: the worker's channel waker must request the UI turn.
    let (sent, sent_rx) = mpsc::channel();
    let (closed, closed_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        for red in 1..=100 {
            tx.send_blocking(frame(red)).expect("receiver is mounted");
        }
        sent.send(()).expect("test is waiting for frames");
        if closed_rx.recv().is_err() {
            return false;
        }
        tx.send_blocking(frame(101)).is_err()
    });
    sent_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("worker sent 100 frames");
    screen.run_until_idle();
    assert_eq!(received.get_untracked(), 100);
    assert_eq!(pixel(&screen.screenshot(), 16, 16), 0x640000);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    drop(screen);
    closed.send(()).expect("worker awaits window close");
    assert!(worker.join().expect("worker finishes"));
}
