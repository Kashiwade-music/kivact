//! Role: SVG parsing and rasterization at an exact device size.
//!
//! SVG parsing (usvg) and rasterization (resvg on tiny-skia) at an exact
//! device size, so SVGs stay sharp at any scale.
//!
//! Invariants:
//! - Built without resvg's `text` feature and with an empty font database,
//!   so `<text>` is never drawn, the same in every build. Raster images
//!   inside an SVG are not drawn either (resvg's `raster-images` is off).
//! - External files are never read: `<image href>` paths resolve to
//!   nothing, so an SVG cannot make the process read other files.
//! - The natural size is the SVG's `width`/`height`, else its `viewBox`.
//! - A raster is exactly the requested size: the drawing is scaled to it
//!   (the caller asks for the aspect ratio it draws at).
//! - The rasterizer's premultiplied pixels are converted to straight alpha.
//! - SVGZ is decompressed here through a reader limited to
//!   `MAX_ENCODED_BYTES + 1` bytes ([`decompress`]), so a small file that
//!   inflates without bound fails as too large; usvg only ever gets plain
//!   text.
//! - Text nested deeper than `nesting::MAX_SVG_DEPTH` elements, or with
//!   entities that expand to markup, is refused before usvg sees it
//!   (`nesting`); the jobs that parse and rasterize run on a thread with a
//!   large stack (`store::worker`), since usvg and resvg recurse per level.

pub(crate) mod nesting;

use std::io::Read;

use resvg::{tiny_skia, usvg};
use ui_types::Size;

use crate::pixels::{Bitmap, Crop, PixelSize};

/// A parsed SVG, shared between the store and the worker for re-rasters.
pub(crate) struct Svg {
    tree: usvg::Tree,
    /// An estimate of the memory the tree holds (see [`Svg::bytes`]).
    bytes: usize,
}

/// Why SVGZ did not decompress.
pub(crate) enum DecompressError {
    /// More than the limit once decompressed.
    TooLarge,
    /// Not a valid gzip stream.
    Malformed(String),
}

/// The plain text of gzip-compressed `bytes`, reading at most `limit + 1`
/// bytes of output (more is `TooLarge`).
pub(crate) fn decompress(bytes: &[u8], limit: u64) -> Result<Vec<u8>, DecompressError> {
    let mut plain = Vec::new();
    flate2::read::GzDecoder::new(bytes)
        .take(limit.saturating_add(1))
        .read_to_end(&mut plain)
        .map_err(|e| DecompressError::Malformed(e.to_string()))?;
    if plain.len() as u64 > limit {
        return Err(DecompressError::TooLarge);
    }
    Ok(plain)
}

/// Parses plain SVG text `bytes` (SVGZ goes through [`decompress`] first).
pub(crate) fn parse(bytes: &[u8]) -> Result<Svg, String> {
    nesting::check(bytes)?;
    let options = usvg::Options {
        resources_dir: None,
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_string: Box::new(|_, _| None),
            ..usvg::ImageHrefResolver::default()
        },
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(bytes, &options).map_err(|e| e.to_string())?;
    let bytes = estimate(&tree);
    Ok(Svg { tree, bytes })
}

/// The memory a parsed tree holds, estimated: each node's struct and each
/// path's points and verbs (shared resources such as gradients and clip
/// paths count as one node each). Walks the tree with an explicit stack.
fn estimate(tree: &usvg::Tree) -> usize {
    let node = std::mem::size_of::<usvg::Path>().max(std::mem::size_of::<usvg::Group>());
    let shared = tree.clip_paths().len()
        + tree.masks().len()
        + tree.patterns().len()
        + tree.filters().len()
        + tree.linear_gradients().len()
        + tree.radial_gradients().len();
    let mut bytes = shared * node;
    let mut groups = vec![tree.root()];
    while let Some(group) = groups.pop() {
        bytes += node;
        for child in group.children() {
            match child {
                usvg::Node::Group(g) => groups.push(g),
                usvg::Node::Path(p) => {
                    let data = p.data();
                    bytes += node + data.points().len() * 8 + data.verbs().len();
                }
                usvg::Node::Image(_) | usvg::Node::Text(_) => bytes += node,
            }
        }
    }
    bytes
}

impl Svg {
    /// The memory the parsed tree holds, estimated; the store counts it in
    /// its budget while it keeps the tree.
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// The natural size in logical px.
    pub(crate) fn natural(&self) -> Size {
        let size = self.tree.size();
        Size {
            width: size.width(),
            height: size.height(),
        }
    }

    /// The drawing scaled to exactly `size` device pixels (both edges
    /// non-zero), only the part `crop` of it when given, as RGBA.
    pub(crate) fn rasterize(&self, size: PixelSize, crop: Option<Crop>) -> Bitmap {
        let area = crop.unwrap_or(Crop {
            x: 0,
            y: 0,
            width: size.width,
            height: size.height,
        });
        let mut pixmap = tiny_skia::Pixmap::new(area.width, area.height)
            .expect("invariant: callers ask for non-empty rasters within the raster limits");
        let natural = self.tree.size();
        let transform = tiny_skia::Transform::from_row(
            size.width as f32 / natural.width(),
            0.0,
            0.0,
            size.height as f32 / natural.height(),
            -(area.x as f32),
            -(area.y as f32),
        );
        resvg::render(&self.tree, transform, &mut pixmap.as_mut());
        let mut rgba = pixmap.take();
        unpremultiply(&mut rgba);
        Bitmap {
            width: area.width,
            height: area.height,
            rgba: rgba.into(),
        }
    }
}

/// Premultiplied RGBA8 to straight alpha, rounding to nearest.
fn unpremultiply(rgba: &mut [u8]) {
    for px in rgba.as_chunks_mut::<4>().0 {
        let a = u32::from(px[3]);
        match a {
            0 => px[..3].fill(0),
            255 => {}
            _ => {
                for c in &mut px[..3] {
                    *c = ((u32::from(*c) * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
    }
}
