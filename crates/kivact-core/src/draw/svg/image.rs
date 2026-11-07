//! Role: Embeds images in an SVG export as PNG data URIs.
//!
//! Raw pixels (`DrawCmd::Pixels`) are cropped to their source rectangle and
//! embedded as they are. An asset (`Image`, `ImagePart`) is decoded and
//! scaled by the same renderer `Offscreen` uses, drawn at the size of its
//! rectangle in device pixels (one per logical pixel, rounded up), so its
//! resolution is that of the drawn size, not of the original file.

use super::emit::Emitter;
use super::encode::{number, png_data_uri};
use crate::{Result, UiError};
use kivact_engine::api::{OffscreenError, OffscreenRenderer};
use kivact_runtime::ErrorKind;
use kivact_types::{DrawCmd, DrawList, Pixels, Rect};
use std::sync::Arc;

impl Emitter<'_> {
    /// Raw pixels, cropped to `source` (in pixels), drawn at `rect`.
    pub(super) fn pixels(
        &mut self,
        image: &Pixels,
        rect: Rect,
        source: Option<Rect>,
        opacity: f64,
        pixelated: bool,
    ) -> Result<()> {
        if !(rect.width > 0.0 && rect.height > 0.0) {
            return Ok(());
        }
        let (x0, y0, width, height) = crop_bounds(image.width, image.height, source);
        if width == 0 || height == 0 {
            return Ok(());
        }
        let pixels: &[u8] = &image.rgba;
        if pixels.len() != image.width as usize * image.height as usize * 4 {
            return Err(self.unsupported("pixels whose length does not match their size"));
        }
        let row = image.width as usize * 4;
        let mut cropped = Vec::with_capacity(width as usize * height as usize * 4);
        for y in y0..y0 + height {
            let start = y as usize * row + x0 as usize * 4;
            cropped.extend_from_slice(&pixels[start..start + width as usize * 4]);
        }
        let uri = png_data_uri(width, height, &cropped, self.at)?;
        self.image_element(rect, &uri, opacity, pixelated);
        Ok(())
    }

    /// An asset drawn at `rect` by the renderer: `cmd` is the same command
    /// with its destination at the origin of an image of `rect`'s size.
    pub(super) fn drawn_image(&mut self, cmd: DrawCmd, rect: Rect) -> Result<()> {
        if !(rect.width > 0.0 && rect.height > 0.0) {
            return Ok(());
        }
        let width = pixel_size(rect.width);
        let height = pixel_size(rect.height);
        if !OffscreenRenderer::accepts_size(width, height) {
            return Err(UiError::new(
                ErrorKind::Message,
                format!("an image of {width}x{height} pixels is too large to embed"),
                Some(self.at),
            ));
        }
        let scene = Arc::new(DrawList {
            cmds: vec![cmd],
            regions: Vec::new(),
        });
        let image = self
            .engine
            .render(&scene, (width, height), self.inherited)
            .map_err(|error| match error {
                OffscreenError::Dropped(_) => self.unsupported("an image the renderer drops"),
                OffscreenError::Failed(message) => {
                    UiError::new(ErrorKind::Asset, message.to_string(), Some(self.at))
                }
            })?;
        let uri = png_data_uri(image.width, image.height, &image.rgba, self.at)?;
        // The renderer has applied the opacity and the pixelation already.
        self.image_element(rect, &uri, 1.0, false);
        Ok(())
    }

    fn image_element(&mut self, rect: Rect, uri: &str, opacity: f64, pixelated: bool) {
        let mut attributes = format!(
            "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\"",
            number(rect.x),
            number(rect.y),
            number(rect.width),
            number(rect.height)
        );
        if opacity != 1.0 {
            attributes.push_str(&format!(" opacity=\"{}\"", number(opacity)));
        }
        if pixelated {
            attributes.push_str(" image-rendering=\"pixelated\"");
        }
        self.body
            .push_str(&format!("<image {attributes} href=\"{uri}\"/>\n"));
    }
}

/// The device size of a logical length: whole pixels, at least one.
fn pixel_size(logical: f64) -> u32 {
    logical.ceil().clamp(1.0, f64::from(u32::MAX)) as u32
}

/// The crop of an image of `width` × `height` pixels to `source` (or all of
/// it), as `(x, y, width, height)` in whole pixels, inside the image.
fn crop_bounds(width: u32, height: u32, source: Option<Rect>) -> (u32, u32, u32, u32) {
    let Some(source) = source else {
        return (0, 0, width, height);
    };
    let left = source.x.floor().clamp(0.0, f64::from(width));
    let top = source.y.floor().clamp(0.0, f64::from(height));
    let right = (source.x + source.width)
        .ceil()
        .clamp(0.0, f64::from(width));
    let bottom = (source.y + source.height)
        .ceil()
        .clamp(0.0, f64::from(height));
    let (left, top) = (left as u32, top as u32);
    let right = right as u32;
    let bottom = bottom as u32;
    (
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    )
}
