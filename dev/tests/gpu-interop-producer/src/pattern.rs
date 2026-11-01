//! Role: The known picture of each frame and its check against shown pixels.
//!
//! A frame is a gradient (red grows to the right, green grows downwards)
//! whose blue is one of four variants (`frame % 4`), with the frame
//! number written in five places (the four corners and the centre) as
//! [`BITS`] blocks of [`BLOCK`] × [`BLOCK`] px, most significant bit first,
//! white for 1 and black for 0. A stale frame decodes to an older number
//! and has another blue; a torn frame decodes differently in different
//! places or mixes blues; an empty place decodes nowhere. Alpha is 255
//! everywhere, so premultiplied and straight pixels are the same bytes,
//! and a frame composited over white is the same color.

/// Side of a bit block, in pixels.
pub const BLOCK: u32 = 8;
/// Bits of the frame number (frames count modulo 2^`BITS`).
pub const BITS: u32 = 8;
/// Distance of the corner groups from the edges.
const MARGIN: u32 = 4;
/// Gradient variants (blue values), chosen by `frame % VARIANTS`.
const VARIANTS: u64 = 4;

/// Whether a `width` × `height` frame holds the pattern.
pub fn fits(width: u32, height: u32) -> bool {
    width >= 2 * (MARGIN + BITS * BLOCK) + BITS * BLOCK && height >= 4 * (MARGIN + BLOCK)
}

/// Top-left corners of the five block groups.
fn groups(width: u32, height: u32) -> [(u32, u32); 5] {
    let row = BITS * BLOCK;
    [
        (MARGIN, MARGIN),
        (width - MARGIN - row, MARGIN),
        (MARGIN, height - MARGIN - BLOCK),
        (width - MARGIN - row, height - MARGIN - BLOCK),
        ((width - row) / 2, (height - BLOCK) / 2),
    ]
}

/// Blue of `frame`'s variant.
fn blue(frame: u64) -> u8 {
    (64 * (frame % VARIANTS) + 32) as u8
}

/// The RGB of `frame` at `(x, y)` of a `width` × `height` frame.
pub fn pixel(width: u32, height: u32, frame: u64, x: u32, y: u32) -> [u8; 3] {
    for (gx, gy) in groups(width, height) {
        if (gx..gx + BITS * BLOCK).contains(&x) && (gy..gy + BLOCK).contains(&y) {
            let bit = (x - gx) / BLOCK;
            let one = (frame >> (BITS - 1 - bit)) & 1 == 1;
            return if one { [255; 3] } else { [0; 3] };
        }
    }
    [
        (x * 256 / width) as u8,
        (y * 256 / height) as u8,
        blue(frame),
    ]
}

/// `frame`'s RGBA8 pixels, rows `pitch` bytes apart.
pub fn rgba(width: u32, height: u32, frame: u64, pitch: usize) -> Vec<u8> {
    let mut out = vec![0; pitch * height as usize];
    for y in 0..height {
        for x in 0..width {
            let [r, g, b] = pixel(width, height, frame, x, y);
            let at = y as usize * pitch + x as usize * 4;
            out[at..at + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    out
}

/// What a shown frame turned out to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The frame, every channel within the tolerance.
    Ok,
    /// Every group decodes to the same other frame.
    Stale(u64),
    /// Groups disagree, or another frame's blue shows.
    Torn,
    /// No group decodes (nothing, or something else, is shown).
    Empty,
    /// It decodes right, but a channel is off by this much.
    Off(u8),
}

/// Check `width` × `height` pixels at `(left, top)` of a frame of 0x00RRGGBB
/// `pixels` (rows `stride` pixels apart) against `frame`'s picture, with
/// channels at most `tolerance` off.
pub fn check(
    pixels: &[u32],
    stride: usize,
    (left, top): (u32, u32),
    (width, height): (u32, u32),
    frame: u64,
    tolerance: u8,
) -> Verdict {
    let at = |x: u32, y: u32| -> [u8; 3] {
        let p = pixels[(top + y) as usize * stride + (left + x) as usize];
        [(p >> 16) as u8, (p >> 8) as u8, p as u8]
    };
    let mut decoded = Vec::new();
    for (gx, gy) in groups(width, height) {
        let mut number = 0u64;
        let mut valid = true;
        for bit in 0..BITS {
            let [r, g, b] = at(gx + bit * BLOCK + BLOCK / 2, gy + BLOCK / 2);
            let one = r > 200 && g > 200 && b > 200;
            valid &= one || (r < 55 && g < 55 && b < 55);
            number = number << 1 | u64::from(one);
        }
        if valid {
            decoded.push(number);
        }
    }
    let Some(&first) = decoded.first() else {
        return Verdict::Empty;
    };
    if decoded.len() < 5 || decoded.iter().any(|&d| d != first) {
        return Verdict::Torn;
    }
    if first != frame % (1 << BITS) {
        return Verdict::Stale(first);
    }
    let mut off = 0;
    // A column away from the block groups: between the left ones and the
    // centre one.
    let row = BITS * BLOCK;
    let clear = (MARGIN + row + (width - row) / 2) / 2;
    for y in 0..height {
        if at(clear, y)[2] != blue(frame) {
            return Verdict::Torn;
        }
        for x in 0..width {
            let want = pixel(width, height, frame, x, y);
            for (got, want) in at(x, y).into_iter().zip(want) {
                off = off.max(got.abs_diff(want));
            }
        }
    }
    if off > tolerance {
        Verdict::Off(off)
    } else {
        Verdict::Ok
    }
}
