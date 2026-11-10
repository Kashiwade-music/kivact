//! Role: Tests for animated images (GIF, APNG, WebP) and SVG text and images.
//!
//! The fixtures are made here: a GIF with the `image` crate's encoder; an
//! APNG and an animated WebP assembled from the still PNG and lossless
//! WebP the same crate encodes (its APNG and animated WebP encoders are not
//! in the crate). Each frame is a solid color, so a pixel of the frame
//! names the frame. Pixels are compared at tier 2 (an image blit of
//! a solid color, ±2/255).

use std::time::Duration;

use super::paint::{BLUE, GREEN, RED, WHITE, at, canvas_of, expect, pixel};
use image::codecs::gif::GifEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{Delay, ExtendedColorType, Frame, ImageEncoder, Rgba, RgbaImage};
use kivact::prelude::*;
use kivact::test::{RenderOptions, render, render_with};
use kivact::view::Asset;

/// The frames' colors, one per 100 ms.
const COLORS: [u32; 3] = [RED, GREEN, BLUE];
/// The side of each frame.
const SIDE: u32 = 40;
/// Inter's variable font, for the SVG text.
static INTER: &[u8] = include_bytes!("../../../../../crates/arvane/tests/fonts/InterVariable.ttf");

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn solid(color: u32) -> RgbaImage {
    let channel = |shift: u32| ((color >> shift) & 255) as u8;
    RgbaImage::from_pixel(SIDE, SIDE, Rgba([channel(16), channel(8), channel(0), 255]))
}

fn frames() -> Vec<RgbaImage> {
    COLORS.iter().map(|&c| solid(c)).collect()
}

/// A GIF of the three frames, 100 ms each.
pub(super) fn gif() -> Vec<u8> {
    let mut out = Vec::new();
    let frames = frames()
        .into_iter()
        .map(|f| Frame::from_parts(f, 0, 0, Delay::from_numer_denom_ms(100, 1)));
    GifEncoder::new(&mut out)
        .encode_frames(frames)
        .expect("a GIF of the frames");
    out
}

/// A still PNG of `image`.
fn still_png(image: &RgbaImage) -> Vec<u8> {
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        )
        .expect("a PNG of the frame");
    out
}

/// The chunks of a PNG file: their type and data.
fn png_chunks(bytes: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
    let mut out = Vec::new();
    let mut at = 8;
    while at + 8 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[at..at + 4].try_into().expect("4 bytes")) as usize;
        let kind: [u8; 4] = bytes[at + 4..at + 8].try_into().expect("4 bytes");
        out.push((kind, bytes[at + 8..at + 8 + len].to_vec()));
        at += 12 + len;
    }
    out
}

/// CRC-32 (ISO 3309), as PNG chunks carry it.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend(kind);
    out.extend(data);
    let crc = crc32(&out[start..]);
    out.extend(crc.to_be_bytes());
}

/// An APNG of the three frames, `delay_ms` each, played forever.
fn apng(delay_ms: u16) -> Vec<u8> {
    let stills: Vec<Vec<([u8; 4], Vec<u8>)>> =
        frames().iter().map(|f| png_chunks(&still_png(f))).collect();
    let ihdr = stills[0]
        .iter()
        .find(|(kind, _)| kind == b"IHDR")
        .map(|(_, data)| data.clone())
        .expect("an IHDR");
    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    png_chunk(&mut out, b"IHDR", &ihdr);
    let mut actl = Vec::new();
    actl.extend((COLORS.len() as u32).to_be_bytes());
    actl.extend(0u32.to_be_bytes());
    png_chunk(&mut out, b"acTL", &actl);
    let mut seq = 0u32;
    for (index, chunks) in stills.iter().enumerate() {
        let mut fctl = Vec::new();
        fctl.extend(seq.to_be_bytes());
        seq += 1;
        fctl.extend(SIDE.to_be_bytes());
        fctl.extend(SIDE.to_be_bytes());
        fctl.extend(0u32.to_be_bytes());
        fctl.extend(0u32.to_be_bytes());
        fctl.extend(delay_ms.to_be_bytes());
        fctl.extend(1000u16.to_be_bytes());
        fctl.extend([0, 0]);
        png_chunk(&mut out, b"fcTL", &fctl);
        let data: Vec<u8> = chunks
            .iter()
            .filter(|(kind, _)| kind == b"IDAT")
            .flat_map(|(_, data)| data.iter().copied())
            .collect();
        if index == 0 {
            png_chunk(&mut out, b"IDAT", &data);
        } else {
            let mut payload = seq.to_be_bytes().to_vec();
            seq += 1;
            payload.extend(data);
            png_chunk(&mut out, b"fdAT", &payload);
        }
    }
    png_chunk(&mut out, b"IEND", &[]);
    out
}

/// The VP8L chunk payload of a lossless WebP of `image`.
fn still_vp8l(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    WebPEncoder::new_lossless(&mut bytes)
        .encode(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        )
        .expect("a lossless WebP");
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let kind = &bytes[at..at + 4];
        let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().expect("4 bytes")) as usize;
        if kind == b"VP8L" {
            return bytes[at + 8..at + 8 + len].to_vec();
        }
        at += 8 + len + (len & 1);
    }
    panic!("the WebP has no VP8L chunk");
}

fn riff_chunk(out: &mut Vec<u8>, kind: &[u8; 4], payload: &[u8]) {
    out.extend(kind);
    out.extend((payload.len() as u32).to_le_bytes());
    out.extend(payload);
    if payload.len() % 2 == 1 {
        out.push(0);
    }
}

fn push_u24(out: &mut Vec<u8>, value: u32) {
    out.extend(&value.to_le_bytes()[..3]);
}

/// An animated WebP of the three frames, 100 ms each, looping forever.
fn animated_webp() -> Vec<u8> {
    let mut body = b"WEBP".to_vec();
    // VP8X: the animation and alpha flags, then the canvas size minus one.
    let mut vp8x = vec![0x02 | 0x10, 0, 0, 0];
    push_u24(&mut vp8x, SIDE - 1);
    push_u24(&mut vp8x, SIDE - 1);
    riff_chunk(&mut body, b"VP8X", &vp8x);
    riff_chunk(&mut body, b"ANIM", &[0; 6]);
    for frame in frames() {
        // ANMF: the frame's offset (halved), size minus one, duration, flags.
        let mut anmf = Vec::new();
        push_u24(&mut anmf, 0);
        push_u24(&mut anmf, 0);
        push_u24(&mut anmf, SIDE - 1);
        push_u24(&mut anmf, SIDE - 1);
        push_u24(&mut anmf, 100);
        anmf.push(0);
        riff_chunk(&mut anmf, b"VP8L", &still_vp8l(&frame));
        riff_chunk(&mut body, b"ANMF", &anmf);
    }
    let mut out = b"RIFF".to_vec();
    out.extend((body.len() as u32).to_le_bytes());
    out.extend(body);
    out
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[test]
fn gif_frames_advance() {
    // Tier 2: each frame is a solid color; the GIF's three frames are shown
    // one after another, 100 ms each.
    let screen = render(|| canvas_of(image(gif()).test_id("gif").style(at(0, 0, 40, 40))));
    let shot = || screen.screenshot();
    expect(&shot(), 20, 20, RED, 2, "frame 0");
    screen.advance(ms(100));
    expect(&shot(), 20, 20, GREEN, 2, "frame 1 after 100 ms");
    screen.advance(ms(100));
    expect(&shot(), 20, 20, BLUE, 2, "frame 2 after 200 ms");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn playing_false_shows_first_frame() {
    // Tier 2: `playing(false)` keeps the first frame.
    let screen = render(|| {
        canvas_of(
            image(gif())
                .test_id("gif")
                .playing(false)
                .style(at(0, 0, 40, 40)),
        )
    });
    expect(&screen.screenshot(), 20, 20, RED, 2, "frame 0");
    screen.advance(ms(100));
    expect(&screen.screenshot(), 20, 20, RED, 2, "still frame 0");
    screen.advance(ms(300));
    expect(&screen.screenshot(), 20, 20, RED, 2, "still frame 0 later");
}

#[test]
fn reduced_motion_shows_first_frame() {
    // Tier 2: under reduced motion the animation shows its first frame.
    let screen = render_with(RenderOptions::from_env().reduced_motion(), || {
        canvas_of(image(gif()).test_id("gif").style(at(0, 0, 40, 40)))
    });
    expect(&screen.screenshot(), 20, 20, RED, 2, "frame 0");
    screen.advance(ms(100));
    expect(&screen.screenshot(), 20, 20, RED, 2, "frame 0 after 100 ms");
}

#[test]
fn gif_cut_off_in_a_later_frame_keeps_the_frames_before_it() {
    // Tier 2: a GIF whose last frame is cut off (a download that stopped) still
    // animates through the frames before it; the image is not lost.
    let mut bytes = gif();
    bytes.truncate(bytes.len() - 20);
    let screen = render(move || canvas_of(image(bytes).test_id("gif").style(at(0, 0, 40, 40))));
    expect(&screen.screenshot(), 20, 20, RED, 2, "frame 0");
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        GREEN,
        2,
        "frame 1 after 100 ms",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn apng_and_webp_animate() {
    // Tier 2: an APNG (left) and an animated WebP (right), both of three
    // 100 ms frames, advance together.
    let screen = render(|| {
        canvas_of((
            image(apng(100)).test_id("apng").style(at(0, 0, 40, 40)),
            image(animated_webp())
                .test_id("webp")
                .style(at(60, 0, 40, 40)),
        ))
    });
    for (when, want) in [(0, RED), (100, GREEN), (100, BLUE)] {
        screen.advance(ms(when));
        let shot = screen.screenshot();
        expect(&shot, 20, 20, want, 2, "the APNG frame");
        expect(&shot, 80, 20, want, 2, "the WebP frame");
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn svg_text_is_drawn() {
    // A `<text>` in a registered font: its glyphs are dark where the text
    // is, on the white background.
    static SVG_TEXT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="60"><rect width="200" height="60" fill="#ffffff"/><text x="10" y="45" font-family="Inter Variable" font-size="40" fill="#000000">Hamburg</text></svg>"##;
    let screen = render_with(RenderOptions::from_env().font(Asset::from(INTER)), || {
        canvas_of(image(SVG_TEXT).test_id("svg").style(at(0, 0, 200, 60)))
    });
    let shot = screen.get_by_test_id("svg").screenshot();
    assert_eq!(pixel(&shot, 0, 0), WHITE, "the background is white");
    let dark = (0..shot.height)
        .flat_map(|y| (0..shot.width).map(move |x| (x, y)))
        .filter(|&(x, y)| pixel(&shot, x, y) & 255 < 80)
        .count();
    assert!(dark > 100, "the text is drawn: {dark} dark pixels");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn svg_data_url_image_is_drawn() {
    // Tier 2: a PNG in a data URL of an SVG is drawn at the SVG's size.
    let png = still_png(&solid(BLUE));
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40"><image width="40" height="40" href="data:image/png;base64,{}"/></svg>"#,
        base64(&png)
    );
    let screen = render(move || {
        canvas_of(
            image(svg.into_bytes())
                .test_id("svg")
                .style(at(0, 0, 40, 40)),
        )
    });
    expect(&screen.screenshot(), 20, 20, BLUE, 2, "the embedded PNG");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
