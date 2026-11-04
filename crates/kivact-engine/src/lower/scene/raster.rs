//! Role: Lowering of local scene raster commands.
use super::state::State;
use crate::lower::context::Context;
use crate::lower::sanitize;
use kivact_types::render::RenderCmd as D;
use kivact_types::{DrawCmd, Rect, Size};
impl Context<'_> {
    pub(super) fn paint_raster(&self, cmd: &DrawCmd, state: &mut State<'_>) -> bool {
        let origin = state.origin;
        let rect = |r: Rect| Rect {
            x: origin.x + r.x,
            y: origin.y + r.y,
            width: r.width,
            height: r.height,
        };
        let tint = &state.current_tint;
        match cmd {
            DrawCmd::Pixels {
                image,
                rect: r,
                source,
                opacity,
                pixelated,
            } => {
                state.out.push(D::ImageSampled {
                    image: kivact_types::scene::Pixels {
                        width: image.width,
                        height: image.height,
                        rgba: image.rgba.clone(),
                    },
                    rect: rect(*r),
                    source: *source,
                    opacity: opacity.clamp(0.0, 1.0) * tint.alpha,
                    color_matrix: tint.matrix,
                    pixelated: *pixelated,
                });
                true
            }
            DrawCmd::External {
                frame,
                rect: r,
                source,
                opacity,
                pixelated,
            } => {
                let command = D::External {
                    frame: *frame,
                    rect: rect(*r),
                    source: *source,
                    opacity: opacity.clamp(0.0, 1.0) * tint.alpha,
                    pixelated: *pixelated,
                };
                if sanitize::drawable(&command) {
                    state.out.push(command);
                } else {
                    self.drop_command();
                }
                true
            }
            DrawCmd::ImagePart {
                source,
                rect: r,
                crop,
                opacity,
                pixelated,
            } => {
                let natural = self.natural(source);
                // The source portion needs the destination's device resolution.
                let drawn = match (crop, natural) {
                    (Some(crop), Some(n)) if crop.width > 0.0 && crop.height > 0.0 => Size {
                        width: r.width * n.width / crop.width,
                        height: r.height * n.height / crop.height,
                    },
                    _ => Size {
                        width: r.width,
                        height: r.height,
                    },
                };
                let size = arvane::asset::PixelSize::of(drawn, self.raster.get());
                let pixels = self.pixels(source, size);
                if let Some((image, _)) = pixels {
                    let source = crop.map(|crop| {
                        let n = natural.unwrap_or(Size {
                            width: image.width as f64,
                            height: image.height as f64,
                        });
                        Rect {
                            x: crop.x / n.width * image.width as f64,
                            y: crop.y / n.height * image.height as f64,
                            width: crop.width / n.width * image.width as f64,
                            height: crop.height / n.height * image.height as f64,
                        }
                    });
                    state.out.push(D::ImageSampled {
                        image,
                        rect: rect(*r),
                        source,
                        opacity: opacity.clamp(0.0, 1.0) * tint.alpha,
                        color_matrix: tint.matrix,
                        pixelated: *pixelated,
                    });
                }
                true
            }
            DrawCmd::Image { source, rect: r } => {
                let r = rect(*r);
                let device = arvane::asset::PixelSize::of(
                    Size {
                        width: r.width,
                        height: r.height,
                    },
                    self.raster.get(),
                );
                if !sanitize::drawable(&D::PushClip(r)) {
                    self.drop_command();
                } else if let Some((image, _)) = self.pixels(source, device) {
                    state.out.push(D::Image {
                        image,
                        rect: r,
                        opacity: tint.alpha,
                        color_matrix: tint.matrix,
                    });
                }
                true
            }

            _ => false,
        }
    }
}

/// The zoom images are rasterized for under transforms enlarging them by
/// `z`: `z` rounded up to a quarter step of powers of two (exactly 1
/// without a transform), so a zooming subtree rasterizes its SVGs again
/// only every 19 %, never blurrier than drawn.
pub(crate) fn raster_zoom(z: f64) -> f64 {
    if z == 1.0 || !z.is_finite() || z <= 0.0 {
        return 1.0;
    }
    ((z.log2() * 4.0).ceil() / 4.0).exp2()
}
