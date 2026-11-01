//! Role: Asset loading contract tests.
//!
//! Assets: images decoded by the store's worker (which wakes the host),
//! by content (PNG, JPEG with its EXIF orientation, SVG), failures
//! reported once, assets compared by identity; the pixels drawn as they
//! are.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use arvane::asset::{Asset, AssetStore, ImageState, PixelSize, Pixels};

use kivact_types::Size;

/// 2 × 2: opaque red, half green / opaque blue, quarter white.
static PNG: &[u8] = include_bytes!("../fixtures/rgba.png");
/// Stored 16 × 8, red left and blue right, with EXIF orientation 6
/// (rotated a quarter turn clockwise to show).
static JPEG: &[u8] = include_bytes!("../fixtures/orientation-6.jpg");
/// 4 × 2: a red square and a half-transparent blue one.
static SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="2"><rect width="2" height="2" fill="#ff0000"/><rect x="2" width="2" height="2" fill="#0000ff" fill-opacity="0.5"/></svg>"##;

/// A store on its worker thread, and the wakes it sends the host.
fn store() -> (AssetStore, Receiver<()>) {
    let (wake, woken) = channel();
    let store = AssetStore::new(move || {
        let _ = wake.send(());
    });
    (store, woken)
}

/// Wait (woken by the store) until `ready` gives an answer.
fn wait<T>(
    store: &mut AssetStore,
    woken: &Receiver<()>,
    mut ready: impl FnMut(&mut AssetStore) -> Option<T>,
) -> T {
    loop {
        if let Some(answer) = ready(store) {
            return answer;
        }
        woken
            .recv_timeout(Duration::from_secs(10))
            .expect("the store's worker wakes the host");
        store.poll();
    }
}

fn state(store: &mut AssetStore, woken: &Receiver<()>, asset: &Asset) -> ImageState {
    wait(store, woken, |store| match store.image(asset) {
        ImageState::Pending => None,
        state => Some(state),
    })
}

fn pixels(
    store: &mut AssetStore,
    woken: &Receiver<()>,
    asset: &Asset,
    width: u32,
    height: u32,
) -> Pixels {
    wait(store, woken, |store| {
        store.pixels(asset, PixelSize { width, height }, None)
    })
}

fn size(width: f32, height: f32) -> Size {
    Size { width, height }
}

/// The straight RGBA of pixel `(x, y)`.
fn rgba(pixels: &Pixels, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * pixels.width() + x) * 4) as usize;
    let bytes = &pixels.rgba()[at..at + 4];
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

#[test]
fn the_worker_decodes_png_jpeg_and_svg_by_their_content() {
    let (mut store, woken) = store();
    let png = Asset::Static(PNG);
    assert_eq!(store.peek(&png), None, "peeking starts nothing");
    assert_eq!(
        state(&mut store, &woken, &png),
        ImageState::Ready {
            natural: size(2.0, 2.0)
        }
    );
    let decoded = pixels(&mut store, &woken, &png, 2, 2);
    assert_eq!((decoded.width(), decoded.height()), (2, 2));
    assert_eq!(
        &decoded.rgba()[..],
        [
            255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 255, 255, 255, 255, 64
        ],
        "straight alpha, as stored"
    );
    // The JPEG shows rotated: 8 × 16, red on top, blue below.
    let jpeg = Asset::Shared(Arc::from(JPEG));
    assert_eq!(
        state(&mut store, &woken, &jpeg),
        ImageState::Ready {
            natural: size(8.0, 16.0)
        }
    );
    let rotated = pixels(&mut store, &woken, &jpeg, 8, 16);
    assert_eq!((rotated.width(), rotated.height()), (8, 16));
    let [r, _, b, a] = rgba(&rotated, 4, 2);
    assert!(r > 200 && b < 60 && a == 255, "red on top: {r} {b}");
    let [r, _, b, _] = rgba(&rotated, 4, 13);
    assert!(b > 200 && r < 60, "blue below: {r} {b}");
    // The SVG, named by no extension, is rasterized at the device size
    // asked for.
    let svg = Asset::Static(SVG);
    assert_eq!(
        state(&mut store, &woken, &svg),
        ImageState::Ready {
            natural: size(4.0, 2.0)
        }
    );
    let raster = pixels(&mut store, &woken, &svg, 8, 4);
    assert_eq!((raster.width(), raster.height()), (8, 4));
    assert_eq!(rgba(&raster, 1, 1), [255, 0, 0, 255]);
    let [r, g, b, a] = rgba(&raster, 6, 2);
    assert_eq!((r, g, b), (0, 0, 255), "straight color");
    assert!(a.abs_diff(128) <= 1, "half transparent: {a}");
}
