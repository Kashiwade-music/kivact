//! Role: Tests for image handling and input validation.
//!
//! Images end to end on real windows: untrusted or odd inputs never take
//! the App down and are one located diagnostic each, and what an image
//! shows is checked in the frame's pixels (fit modes, device-scale SVG
//! rasters, JPEG orientation, a GIF's first frame, the format read from
//! the content, the monochrome tint in the node's effective text color).

use std::path::Path;

use kivact::Asset;
use kivact::control::ImageFit;
use kivact::diagnostics::ErrorKind;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{RenderOptions, Screen, render, render_with};
use kivact::view::MouseButton;

static PNG_2X2: &[u8] = include_bytes!("../fixtures/rgba.png");
static SVG_VIEWBOX_30X10: &[u8] = include_bytes!("../fixtures/viewbox.svg");
/// Stored 16×8 with red left and blue right, EXIF orientation 6.
static JPEG_ORIENTATION_6: &[u8] = include_bytes!("../fixtures/orientation-6.jpg");
/// 2×2, first frame red, green / blue, white; the second frame differs.
static GIF_TWO_FRAMES: &[u8] = include_bytes!("../fixtures/two-frames.gif");

/// 30×10: red, green and blue thirds.
static THIRDS: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="30" height="10"><rect width="10" height="10" fill="#ff0000"/><rect x="10" width="10" height="10" fill="#00ff00"/><rect x="20" width="10" height="10" fill="#0000ff"/></svg>"##;

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const BLUE: u32 = 0x0000ff;

fn asset_errors(screen: &Screen) -> Vec<UiError> {
    screen
        .errors()
        .into_iter()
        .filter(|e| *e.kind() == ErrorKind::Asset)
        .collect()
}

/// The pixel at `(x, y)` logical px inside the element's box (scale 1).
fn pixel_in(screen: &Screen, id: &str, x: f32, y: f32) -> u32 {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    let s = screen.scale();
    screen
        .screenshot()
        .pixel(((r.x + x) * s) as u32, ((r.y + y) * s) as u32)
        .expect("inside the frame")
}

/// A 10×10 red square inside `depth` nested elements (the root `<svg>`
/// counts as one).
fn nested_svg(depth: usize) -> Vec<u8> {
    let mut s = String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">"#);
    s.push_str(&"<g>".repeat(depth - 1));
    s.push_str(r##"<rect width="10" height="10" fill="#ff0000"/>"##);
    s.push_str(&"</g>".repeat(depth - 1));
    s.push_str("</svg>");
    s.into_bytes()
}

/// A gzip stream (an SVGZ by its magic bytes) that inflates to about 70 MB
/// of spaces from about 440 KB: one fixed-Huffman deflate block of one
/// literal and then back references of the longest length at distance 1.
fn svgz_bomb() -> Vec<u8> {
    struct Bits {
        out: Vec<u8>,
        acc: u32,
        n: u32,
    }
    impl Bits {
        /// `count` bits of `value`, least significant first (header fields).
        fn value(&mut self, value: u32, count: u32) {
            for i in 0..count {
                self.bit((value >> i) & 1);
            }
        }
        /// A Huffman code, most significant bit first.
        fn code(&mut self, code: u32, count: u32) {
            for i in (0..count).rev() {
                self.bit((code >> i) & 1);
            }
        }
        fn bit(&mut self, b: u32) {
            self.acc |= b << self.n;
            self.n += 1;
            if self.n == 8 {
                self.out.push(self.acc as u8);
                (self.acc, self.n) = (0, 0);
            }
        }
    }
    let mut bits = Bits {
        out: vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff],
        acc: 0,
        n: 0,
    };
    bits.value(1, 1); // BFINAL
    bits.value(1, 2); // BTYPE: fixed Huffman
    bits.code(0x30 + u32::from(b' '), 8); // the literal ' '
    for _ in 0..270_000 {
        bits.code(0b1100_0101, 8); // length 258 (code 285)
        bits.code(0, 5); // distance 1
    }
    bits.code(0, 7); // end of block
    bits.value(0, 7);
    bits.out.extend([0; 8]); // CRC and size: never reached
    bits.out
}

#[test]
fn untrusted_images_are_one_diagnostic_each_and_the_app_goes_on() {
    let cases: Vec<(&str, Asset, &str)> = vec![
        (
            "deep",
            Asset::from(nested_svg(257)),
            "nested more than 256 levels",
        ),
        (
            "bomb",
            Asset::from(svgz_bomb()),
            "decompressed SVGZ is larger than the 64 MiB limit",
        ),
        (
            "bmp",
            Asset::from(&b"BM\x46\0\0\0\0\0\0\0\x36\0\0\0"[..]),
            "BMP",
        ),
        ("tiff", Asset::from(&b"II*\0\x08\0\0\0"[..]), "TIFF"),
        ("woff", Asset::from(&b"wOFF\0\x01\0\0"[..]), "WOFF"),
        (
            "unknown",
            Asset::from(&b"hello"[..]),
            "unknown content starting with 68 65 6c 6c 6f",
        ),
        ("empty", Asset::from(&b""[..]), "empty content"),
        (
            "corrupt png",
            Asset::from(&b"\x89PNG\r\n\x1a\nnot really"[..]),
            "cannot decode PNG",
        ),
        (
            "broken svg",
            Asset::from(&b"<svg broken"[..]),
            "cannot decode SVG: SVG data parsing failed",
        ),
        (
            "url",
            Asset::from("https://example.com/logo.png"),
            "URLs are not supported",
        ),
        ("missing", Asset::from("no/such/file.png"), "file.png"),
    ];
    let sources: Vec<(String, Asset)> = cases
        .iter()
        .map(|(id, a, _)| (id.to_string(), a.clone()))
        .collect();
    let screen = render(move || {
        let images: Vec<_> = sources
            .iter()
            .map(|(id, a)| image(a.clone()).test_id(id.as_str()))
            .collect();
        column((
            column(images),
            image(nested_svg(256)).test_id("deepest allowed"),
            text("still here"),
        ))
        .style(style().align_items(Align::Start))
    });
    screen.run_until_idle();
    let errors = asset_errors(&screen);
    assert_eq!(errors.len(), cases.len(), "{errors:#?}");
    for (id, _, reason) in &cases {
        let hits = errors
            .iter()
            .filter(|e| e.message().contains(reason))
            .count();
        assert_eq!(hits, 1, "{id}: one diagnostic with {reason:?}: {errors:#?}");
        let r = screen.get_by_test_id(id).rect().expect("laid out");
        assert_eq!((r.width, r.height), (0.0, 0.0), "{id} draws nothing");
    }
    // Nesting up to the limit draws; the App keeps answering.
    assert_eq!(pixel_in(&screen, "deepest allowed", 5.0, 5.0), RED);
    screen.get_by_text("still here");
    assert_eq!(asset_errors(&screen).len(), cases.len(), "reported once");
}

#[test]
fn fit_modes_scale_and_formats_show_in_pixels() {
    let dir = std::env::temp_dir().join(format!("kivact-image-inputs-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let disguised = dir.join("really-a-png.jpg");
    std::fs::write(&disguised, PNG_2X2).expect("write");
    let fitted = |fit: ImageFit, id: &str, side: f32| {
        image(THIRDS)
            .fit(fit)
            .test_id(id)
            .style(style().size(side, side))
    };
    let screen = render(move || {
        column((
            row((
                image(THIRDS).test_id("contain").style(style().size(60, 60)),
                fitted(ImageFit::Cover, "cover", 60.0),
                fitted(ImageFit::Fill, "fill", 60.0),
                fitted(ImageFit::None, "none", 60.0),
                fitted(ImageFit::ScaleDown, "scale down", 15.0),
            )),
            row((
                image(SVG_VIEWBOX_30X10).test_id("viewbox"),
                image(JPEG_ORIENTATION_6).test_id("jpeg"),
                image(GIF_TWO_FRAMES).test_id("gif"),
                image(disguised.clone()).test_id("disguised"),
            ))
            .style(style().align_items(Align::Start)),
        ))
        .style(style().align_items(Align::Start))
    });
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    let px = |id: &str, x: f32, y: f32| pixel_in(&screen, id, x, y);
    // Contain (the default): 60×20, centred; the box shows through above.
    assert_eq!(px("contain", 5.0, 30.0), RED);
    assert_eq!(px("contain", 55.0, 30.0), BLUE);
    assert!(![RED, GREEN, BLUE].contains(&px("contain", 30.0, 10.0)));
    // Cover: 180×60 centred, the middle third fills the box, clipped.
    assert_eq!(px("cover", 2.0, 30.0), GREEN);
    assert_eq!(px("cover", 57.0, 30.0), GREEN);
    // Fill stretches; None keeps the natural size, centred.
    assert_eq!(px("fill", 5.0, 5.0), RED);
    assert_eq!(px("fill", 55.0, 55.0), BLUE);
    assert_eq!(px("none", 20.0, 30.0), RED);
    assert_eq!(px("none", 30.0, 30.0), GREEN);
    assert!(![RED, GREEN, BLUE].contains(&px("none", 10.0, 30.0)));
    // ScaleDown into a smaller box contains: 15×5.
    assert_eq!(px("scale down", 2.0, 7.0), RED);
    assert_eq!(px("scale down", 13.0, 7.0), BLUE);

    let size = |id: &str| {
        let r = screen.get_by_test_id(id).rect().expect("laid out");
        (r.width, r.height)
    };
    assert_eq!(
        size("viewbox"),
        (30.0, 10.0),
        "natural size from the viewBox"
    );
    assert_eq!(
        size("jpeg"),
        (8.0, 16.0),
        "EXIF orientation 6 turns it upright"
    );
    let near = |got: u32, want: u32| {
        (0..3).all(|i| ((got >> (8 * i)) & 0xff).abs_diff((want >> (8 * i)) & 0xff) <= 24)
    };
    assert!(near(px("jpeg", 4.0, 3.0), RED), "red on top");
    assert!(near(px("jpeg", 4.0, 12.0), BLUE), "blue below");
    let gif: Vec<u32> = [(0.5, 0.5), (1.5, 0.5), (0.5, 1.5), (1.5, 1.5)]
        .into_iter()
        .map(|(x, y)| px("gif", x, y))
        .collect();
    assert_eq!(gif, [RED, GREEN, BLUE, 0xffffff], "the first frame");
    assert_eq!(size("disguised"), (2.0, 2.0), "a PNG named .jpg");

    // At scale 2 the SVG is rasterized again at the device size: both
    // thirds of a 2-px-wide area keep their own color.
    screen.set_scale(2.0);
    assert_eq!(size("none"), (60.0, 60.0), "logical sizes stay");
    assert_eq!(px("none", 24.75, 30.0), RED);
    assert_eq!(px("none", 25.25, 30.0), GREEN);
    let _ = std::fs::remove_dir_all(Path::new(&dir));
}

#[test]
fn a_monochrome_image_takes_the_effective_text_color() {
    let gray = rgb(0x808080);
    let screen = render(move || {
        let off = signal(false);
        column((
            node()
                .role(Role::Button)
                .name("Tinted")
                .on_press(|| {})
                .disabled(off)
                .style(
                    style()
                        .size(40, 40)
                        .color(rgb(0x202020))
                        .hover(style().color(rgb(0x2040d0)))
                        .pressed(style().color(rgb(0xd02020)))
                        .disabled(style().color(gray)),
                )
                .child(
                    image(THIRDS)
                        .monochrome(true)
                        .test_id("tinted")
                        .style(style().size(30, 10)),
                ),
            node().style(style().color(rgb(0x202020))).child(
                image(THIRDS)
                    .test_id("own colors")
                    .style(style().size(30, 10)),
            ),
            node()
                .test_id("disable")
                .style(style().size(40, 20))
                .role(Role::Generic)
                .on_press(move || off.set(true)),
        ))
        .style(style().align_items(Align::Start))
    });
    let r = screen.get_by_test_id("tinted").rect().expect("laid out");
    let (x, y) = (r.x + 25.0, r.y + 5.0);
    let tint = || screen.screenshot().pixel(x as u32, y as u32);
    assert_eq!(tint(), Some(0x202020), "inherited text color, alpha only");
    screen.mouse_move(x, y);
    assert_eq!(tint(), Some(0x2040d0), "the hover color");
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(tint(), Some(0xd02020), "the pressed color");
    screen.mouse_up(MouseButton::Primary);
    screen.mouse_leave();
    assert_eq!(tint(), Some(0x202020));
    assert_eq!(
        pixel_in(&screen, "own colors", 25.0, 5.0),
        BLUE,
        "not tinted"
    );
    screen.get_by_test_id("disable").click();
    assert_eq!(tint(), Some(0x808080), "the disabled color");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// What a window draws and reports while its scale changes (it moved to a
/// monitor with another DPI).
fn scaled_view() -> impl IntoView {
    column((
        text("Hinted text, at every scale").test_id("text"),
        image(THIRDS).style(style().size(60, 20)),
        node().style(
            style()
                .size(80, 30)
                .radius(6)
                .border(1.5, rgb(0x335577))
                .background(rgb(0xeeeeff))
                .shadow(kivact::draw::Shadow::new(2.0, 3.0, 8.0, rgb(0x000000)).spread(1.0)),
        ),
        kivact::control::node()
            .name("Name")
            .part(kivact::interaction::use_text_edit(signal(String::new())).control()),
    ))
    .style(style().padding(10).gap(8).align_items(Align::Start))
}

/// A window whose scale changes draws exactly what a fresh window at that
/// scale draws (text hinted for the new device pixels, the SVG rasterized
/// again, shadows, borders, a field with an open composition), and keeps
/// its layout, accessibility tree and IME character boxes in logical
/// pixels.
#[test]
fn a_scale_change_draws_what_a_fresh_window_at_that_scale_draws() {
    let screen = render(scaled_view);
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.compose("ab");
    let (tree, ime, rect) = (screen.accessibility_tree(), screen.ime_text(), field.rect());
    for scale in [2.0, 1.5, 1.0] {
        screen.set_scale(scale);
        let shot = screen.screenshot();
        let fresh = render_with(
            RenderOptions {
                scale,
                ..RenderOptions::from_env()
            },
            scaled_view,
        );
        fresh.get_by_role(Role::TextBox, "Name").compose("ab");
        let want = fresh.screenshot();
        assert_eq!(
            (shot.width, shot.height),
            ((800.0 * scale) as u32, (600.0 * scale) as u32),
            "the frame has the device pixels of scale {scale}"
        );
        assert!(
            shot.hash() == want.hash(),
            "scale {scale}: the frame differs from a fresh window's"
        );
        assert_eq!(field.rect(), rect, "the layout stays in logical pixels");
        assert_eq!(screen.accessibility_tree(), tree);
        assert_eq!(screen.ime_text(), ime);
    }
}
