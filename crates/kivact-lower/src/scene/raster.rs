//! Role: Lowering of local scene raster commands.
use super::state::State;
use crate::context::Context;
use crate::sanitize;
use kivact_types::DrawCommand as D;
use kivact_wire::*;
impl Context<'_> {
    pub(super) fn paint_raster(&self, cmd: &SceneCmd, state: &mut State<'_>) -> bool {
        let origin = state.origin;
        let rect = |r: Rect| Rect {
            x: origin.x + r.x,
            y: origin.y + r.y,
            width: r.width,
            height: r.height,
        };
        let tint = &state.current_tint;
        match cmd {
            SceneCmd::Pixels {
                image,
                rect: r,
                source,
                opacity,
                pixelated,
            } => {
                state.out.commands.push(D::ImageSampled {
                    image: kivact_types::ImageData {
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
            #[cfg(feature = "gpu-interop")]
            SceneCmd::External {
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
                    state.out.commands.push(command);
                } else {
                    self.drop_command();
                }
                true
            }
            // An external frame draws only where it was imported.
            #[cfg(not(feature = "gpu-interop"))]
            SceneCmd::External { .. } => true,
            SceneCmd::ImagePart {
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
                            width: image.width as f32,
                            height: image.height as f32,
                        });
                        Rect {
                            x: crop.x / n.width * image.width as f32,
                            y: crop.y / n.height * image.height as f32,
                            width: crop.width / n.width * image.width as f32,
                            height: crop.height / n.height * image.height as f32,
                        }
                    });
                    state.out.commands.push(D::ImageSampled {
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
            SceneCmd::Image { source, rect: r } => {
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
                    state.out.commands.push(D::Image {
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
