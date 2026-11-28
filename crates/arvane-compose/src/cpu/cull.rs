//! Role: Which commands of a chunk can touch a region.

use arvane_protocol::{PixelRect, RenderCmd, RenderScene};

use crate::cache::{Indexed, Reach};

/// The commands of a chunk that a raster of `region` must encode: every push
/// and pop (so the painter's group stack stays balanced), every command
/// inside a blurred group (its blur reaches past the group's own bounds), and
/// each draw whose reach overlaps `region`.
pub(crate) fn keep_mask(scene: &RenderScene, index: &[Indexed], region: PixelRect) -> Vec<bool> {
    // One entry per open push: whether it opened a blurred group.
    let mut open: Vec<bool> = Vec::new();
    // The blurred groups among `open`.
    let mut blurred = 0usize;
    scene
        .commands
        .iter()
        .zip(index)
        .map(|(command, indexed)| match command {
            RenderCmd::PushGroup(group) => {
                let blur = group.blur.is_some();
                open.push(blur);
                blurred += usize::from(blur);
                true
            }
            RenderCmd::PushClip(_) | RenderCmd::PushTransform(_) => {
                open.push(false);
                true
            }
            RenderCmd::PopGroup | RenderCmd::PopClip | RenderCmd::PopTransform => {
                if open.pop() == Some(true) {
                    blurred -= 1;
                }
                true
            }
            _ => {
                blurred > 0 || matches!(indexed.reach, Reach::Draws(rect) if overlaps(rect, region))
            }
        })
        .collect()
}

/// Whether two px rectangles share a pixel.
fn overlaps(a: PixelRect, b: PixelRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

#[cfg(test)]
mod tests {
    //! Role: Tests of which commands of a chunk a raster culls.

    use arvane_protocol::{Group, PixelRect, RasterPaint, RenderCmd, RenderScene};
    use arvane_render::CpuRaster;
    use kivact_types::geometry::{Point, Rect};
    use kivact_types::pixel::BlendMode;
    use kivact_types::shaped::{FontSource, Glyph, GlyphRun, GlyphRunSnapshot};
    use kivact_types::{PaintOrder, Rgba};

    use super::keep_mask;
    use crate::cache::{Indexed, Reach};

    /// The test font of the `arvane` crate.
    static FONT: &[u8] = include_bytes!("../../../arvane/assets/fonts/Tuffy.ttf");
    /// The frame's size in px.
    const SIZE: (u16, u16) = (512, 512);
    /// The 8 × 8 px region the raster draws.
    const REGION: PixelRect = PixelRect {
        x: 100,
        y: 100,
        width: 8,
        height: 8,
    };

    /// One command with its reach in px.
    fn push(
        commands: &mut Vec<RenderCmd>,
        index: &mut Vec<Indexed>,
        command: RenderCmd,
        reach: Reach,
    ) {
        commands.push(command);
        index.push(Indexed { hash: 0, reach });
    }

    /// A text run of one glyph at (300, 300) px, and its reach.
    fn text_run() -> (RenderCmd, Reach) {
        let run = GlyphRunSnapshot::new(GlyphRun {
            font: FontSource::from_static(FONT),
            font_index: 0,
            font_size: 24.0,
            glyphs: vec![Glyph {
                id: 40,
                x: 0.0,
                y: 0.0,
                advance: 12.0,
            }],
            color: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            stroke: None,
            normalized_coords: Vec::new(),
        });
        let command = RenderCmd::Glyphs {
            run,
            origin: Point::new(300.0, 300.0),
            fill: None,
            stroke: None,
            order: PaintOrder::FillStroke,
        };
        let reach = Reach::Draws(PixelRect {
            x: 300,
            y: 300,
            width: 60,
            height: 20,
        });
        (command, reach)
    }

    /// A scene of 500 rects in a grid, a blurred group that holds one rect
    /// over the region and 100 rects beside it, and a text run away from the
    /// region, with the reach of each command.
    fn scene_and_index() -> (RenderScene, Vec<Indexed>) {
        let mut commands = Vec::new();
        let mut index = Vec::new();
        let rect = |i: u32| {
            let (x, y) = ((i % 50) * 10, (i / 50) * 20);
            (
                Rect::new(f64::from(x), f64::from(y), 10.0, 10.0),
                PixelRect {
                    x,
                    y,
                    width: 10,
                    height: 10,
                },
            )
        };
        let draw = |commands: &mut Vec<RenderCmd>, index: &mut Vec<Indexed>, i: u32| {
            let (bounds, px) = rect(i);
            push(
                commands,
                index,
                RenderCmd::Rect {
                    rect: bounds,
                    fill: RasterPaint::Solid(Rgba {
                        r: 0.2,
                        g: 0.5,
                        b: 0.3,
                        a: 1.0,
                    }),
                },
                Reach::Draws(px),
            );
        };
        for i in 0..250 {
            draw(&mut commands, &mut index, i);
        }
        push(
            &mut commands,
            &mut index,
            RenderCmd::PushGroup(Group {
                clip: None,
                opacity: 1.0,
                blend: BlendMode::Normal,
                blur: Some(4.0),
            }),
            Reach::Opens(REGION),
        );
        for i in 250..350 {
            draw(&mut commands, &mut index, i);
        }
        push(
            &mut commands,
            &mut index,
            RenderCmd::PopGroup,
            Reach::Nothing,
        );
        for i in 350..500 {
            draw(&mut commands, &mut index, i);
        }
        let (command, reach) = text_run();
        push(&mut commands, &mut index, command, reach);
        (RenderScene { commands }, index)
    }

    /// The pixels of the region drawn from `scene`, with the commands
    /// `keep` leaves out (None draws all of them).
    fn raster(scene: &RenderScene, keep: Option<&[bool]>) -> Vec<u8> {
        let mut out = vec![0u8; usize::from(SIZE.0) * usize::from(SIZE.1) * 4];
        CpuRaster::new()
            .frame(|raster| {
                raster.raster(
                    scene,
                    Point::new(0.0, 0.0),
                    1.0,
                    SIZE,
                    Some(REGION),
                    &mut out,
                    keep,
                )
            })
            .expect("the scene rasters");
        out
    }

    /// Culling keeps every pixel of the region, keeps every push and pop, and
    /// keeps few draws outside the blurred group.
    #[test]
    fn culling_keeps_the_pixels() {
        let (scene, index) = scene_and_index();
        let keep = keep_mask(&scene, &index, REGION);
        let opens = scene
            .commands
            .iter()
            .position(|c| matches!(c, RenderCmd::PushGroup(_)))
            .expect("the scene has a group");
        let closes = scene
            .commands
            .iter()
            .rposition(|c| matches!(c, RenderCmd::PopGroup))
            .expect("the group closes");
        for (i, cmd) in scene.commands.iter().enumerate() {
            if matches!(
                cmd,
                RenderCmd::PushGroup(_)
                    | RenderCmd::PopGroup
                    | RenderCmd::PushClip(_)
                    | RenderCmd::PopClip
                    | RenderCmd::PushTransform(_)
                    | RenderCmd::PopTransform
            ) {
                assert!(keep[i], "push and pop command {i} is kept");
            }
        }
        assert_eq!(
            raster(&scene, Some(&keep)),
            raster(&scene, None),
            "the region's pixels are the same with the mask",
        );
        let outside = scene
            .commands
            .iter()
            .enumerate()
            .filter(|(i, cmd)| {
                keep[*i]
                    && !(opens..=closes).contains(i)
                    && matches!(cmd, RenderCmd::Rect { .. } | RenderCmd::Glyphs { .. })
            })
            .count();
        assert!(
            outside <= 10,
            "{outside} draws outside the blurred group are kept",
        );
    }
}
