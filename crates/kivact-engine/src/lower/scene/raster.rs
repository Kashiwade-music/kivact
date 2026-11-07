//! Role: Lowering of local scene raster commands.
use super::state::State;
use crate::lower::context::Context;
use crate::lower::sanitize;
use kivact_types::render::{ExternalItem, LayerItem, RenderCmd as D};
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
                    image: kivact_types::Pixels {
                        width: image.width,
                        height: image.height,
                        rgba: image.rgba.clone(),
                    },
                    rect: rect(*r),
                    source: *source,
                    opacity: opacity.clamp(0.0, 1.0) * tint.alpha,
                    pixelated: *pixelated,
                });
                true
            }
            DrawCmd::External {
                frame,
                cpu_pixels,
                rect: r,
                source,
                opacity,
                pixelated,
                ..
            } => {
                // A frame of an external engine is a layer item; the CPU
                // compositor draws its `cpu` pixels when it has them.
                let item = ExternalItem {
                    frame: frame.clone(),
                    rect: rect(*r),
                    source: *source,
                    transform: state.out.space(),
                    clips: state.out.clips(),
                    opacity: opacity.clamp(0.0, 1.0) * tint.alpha,
                    pixelated: *pixelated,
                    cpu: cpu_pixels.clone(),
                };
                state.out.item(LayerItem::External(item));
                true
            }
            DrawCmd::ImagePart { .. } => {
                self.paint_image_part(cmd, state);
                true
            }
            DrawCmd::Image { .. } => {
                self.paint_image(cmd, state);
                true
            }

            _ => false,
        }
    }
    fn paint_image(&self, cmd: &DrawCmd, state: &mut State<'_>) {
        let DrawCmd::Image { source, rect: r } = cmd else {
            unreachable!("invariant: matching image command");
        };
        let r = Rect {
            x: state.origin.x + r.x,
            y: state.origin.y + r.y,
            ..*r
        };
        let tint = &state.current_tint;
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
            });
        }
    }
    fn paint_image_part(&self, cmd: &DrawCmd, state: &mut State<'_>) {
        let DrawCmd::ImagePart {
            source,
            rect: r,
            crop,
            opacity,
            pixelated,
        } = cmd
        else {
            unreachable!("invariant: matching command family");
        };
        let origin = state.origin;
        let rect = |r: Rect| Rect {
            x: origin.x + r.x,
            y: origin.y + r.y,
            width: r.width,
            height: r.height,
        };
        let tint = &state.current_tint;

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
                pixelated: *pixelated,
            });
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
