//! Role: Verifies that closing a real host joins its asset decoding worker.
//!
//! A separate binary keeps process-wide thread counts independent of other tests.
//! Run through window-lock with --test-threads=1.

#[path = "support/thread_names.rs"]
mod thread_names;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use kivact_engine::bridge::Client;
use kivact_engine::testing::{ArvaneTestHost, TestHostOptions, WindowMode};
use kivact_wire::{Asset, Dimension, ImageData, LayoutProps, NodeId, NodeKind, TurnOutput};

fn asset_threads() -> usize {
    thread_names::names()
        .iter()
        .filter(|name| name.starts_with("arvane-asset"))
        .count()
}

fn decode_and_close() {
    #[cfg(feature = "embed")]
    {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<kivact_engine::embed::EmbeddedHost>();
    }
    let writer = Rc::new(RefCell::new(None));
    let captured = writer.clone();
    let mut host = ArvaneTestHost::new(
        Client::new(
            move |_, _, writer| {
                *captured.borrow_mut() = Some(writer);
                None
            },
            |_, _, _| TurnOutput::default(),
        ),
        TestHostOptions {
            window: WindowMode::Real,
            inline_assets: false,
            width: 32.0,
            height: 32.0,
            ..TestHostOptions::default()
        },
    )
    .expect("the real test host opens");
    let writer = writer.borrow_mut().take().expect("client started");
    writer.create(NodeId(1), NodeKind::Root);
    writer.create(
        NodeId(2),
        NodeKind::Image(ImageData {
            image: Some(Asset::Static(br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="#ff0000"/></svg>"##)),
            ..ImageData::default()
        }),
    );
    writer.set_layout(
        NodeId(2),
        LayoutProps {
            width: Dimension::Px(16.0),
            height: Dimension::Px(16.0),
            ..LayoutProps::default()
        },
    );
    writer.insert(NodeId(1), NodeId(2), None);
    writer.commit();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        host.wake();
        while host.redraw() {}
        assert!(host.failure().is_none(), "{:?}", host.failure());
        if host.frame().and_then(|frame| frame.pixel(8, 8)) == Some(0x00ff0000) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the worker decodes and draws the image"
        );
        std::thread::park_timeout(Duration::from_millis(1));
    }
    assert!(
        asset_threads() > 0,
        "the host actually started an asset worker"
    );
    drop(writer);
    drop(host);
}

#[test]
fn closing_the_host_joins_the_asset_worker() {
    decode_and_close(); // Warm up the process-wide real display and font setup.
    let before = asset_threads();
    assert_eq!(before, 0, "warm-up leaves no asset worker behind");
    for _ in 0..3 {
        decode_and_close();
        assert_eq!(asset_threads(), before, "closing joins the asset worker");
    }
}
