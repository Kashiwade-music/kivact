//! Role: A parsed SVG: parsing, memory estimate and exact-size rasterization.

use std::io::Cursor;
use std::sync::Arc;

use crate::pixels::{Crop, PixelSize};
use kivact_types::Pixels;
use kivact_types::Size;
use kivact_types::color::unpremultiply;
use resvg::{tiny_skia, usvg};

/// The font database an SVG's `<text>` is shaped with.
pub(crate) type FontDb = Arc<super::SvgFonts>;

/// A parsed SVG takes at most `budget / SVG_SHARE` of the store's budget.
const SVG_SHARE: usize = 4;

/// Bytes a character of `<text>` is estimated to take in the parsed tree
/// (its glyphs converted to outlines).
const TEXT_CHAR_BYTES: usize = 512;

/// A parsed SVG, shared between the store and the worker for re-rasters.
pub(crate) struct Svg {
    tree: usvg::Tree,
    /// An estimate of the memory the tree holds (see [`Svg::bytes`]).
    bytes: usize,
}

/// Why plain SVG text did not parse.
pub(crate) enum ParseError {
    /// Not an SVG this crate accepts, with the reason.
    Malformed(String),
    /// The parsed tree would take `bytes`, more than its share of the budget.
    TooLarge { bytes: usize },
}

/// Parses plain SVG text `bytes` with `fonts` for its `<text>` (SVGZ goes
/// through [`decompress`](super::decompress) first). A tree estimated above
/// a quarter of `budget` is too large.
pub(crate) fn parse(bytes: &[u8], fonts: &FontDb, budget: usize) -> Result<Svg, ParseError> {
    super::nesting::check(bytes).map_err(ParseError::Malformed)?;
    let options = usvg::Options {
        resources_dir: None,
        fontdb: fonts.for_document(bytes),
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_string: Box::new(|_, _| None),
            ..usvg::ImageHrefResolver::default()
        },
        ..usvg::Options::default()
    };
    let tree =
        usvg::Tree::from_data(bytes, &options).map_err(|e| ParseError::Malformed(e.to_string()))?;
    let bytes = estimate(&tree);
    if bytes > budget / SVG_SHARE {
        return Err(ParseError::TooLarge { bytes });
    }
    Ok(Svg { tree, bytes })
}

/// The memory a parsed tree holds, estimated: each node's struct, each
/// path's points and verbs, each embedded raster image's decoded pixels,
/// each embedded SVG's own tree and each text character's outlines (shared
/// resources such as gradients and clip paths count as one node each). Walks
/// the tree with an explicit stack.
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
                usvg::Node::Image(image) => bytes += node + image_bytes(image.kind()),
                usvg::Node::Text(text) => {
                    let chars: usize = text
                        .chunks()
                        .iter()
                        .map(|chunk| chunk.text().chars().count())
                        .sum();
                    bytes += node + chars * TEXT_CHAR_BYTES;
                }
            }
        }
    }
    bytes
}

/// The memory an embedded image holds: an embedded SVG's parsed tree (usvg
/// parses `data:image/svg+xml` into a nested tree), or the RGBA8 bytes a
/// raster image decodes to, from its header's size; zero for data whose size
/// cannot be read (usvg draws nothing for those either).
fn image_bytes(kind: &usvg::ImageKind) -> usize {
    let data = match kind {
        usvg::ImageKind::JPEG(data)
        | usvg::ImageKind::PNG(data)
        | usvg::ImageKind::GIF(data)
        | usvg::ImageKind::WEBP(data) => data,
        usvg::ImageKind::SVG(tree) => return estimate(tree),
    };
    image::ImageReader::new(Cursor::new(data.as_slice()))
        .with_guessed_format()
        .ok()
        .and_then(|reader| reader.into_dimensions().ok())
        .map_or(0, |(width, height)| width as usize * height as usize * 4)
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
            width: f64::from(size.width()),
            height: f64::from(size.height()),
        }
    }

    /// The drawing scaled to exactly `size` device pixels (both edges
    /// non-zero), only the part `crop` of it when given, as RGBA.
    pub(crate) fn rasterize(&self, size: PixelSize, crop: Option<Crop>) -> Pixels {
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
        Pixels {
            width: area.width,
            height: area.height,
            rgba: rgba.into(),
        }
    }
}
