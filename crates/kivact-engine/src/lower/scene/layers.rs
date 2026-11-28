//! Role: Lowering of local scene layers commands.
//!
//! Invariants:
//! - A `PushLayer` opens a child layer of the output (a surface) when it
//!   needs its own compositor layer: a 3D transform, a filter, a mask, a
//!   Porter–Duff operator other than source-over, or an item (an external
//!   frame, a shader, a drawable) inside it. Otherwise it opens a group
//!   (clip, opacity, blend) in the chunk.
//! - A surface's transform is about the recording origin; its content is
//!   lowered in the scene's own coordinates (the canvas origin added to each
//!   rect), so the surface's space is those coordinates.

use super::state::{Frame, State};
use crate::lower::LayerProps;
use crate::lower::context::Context;
use crate::lower::sanitize;
use arvane_protocol::{Group, LayerEffects, Plane, RasterGrid, RenderCmd as D};
use kivact_types::Composite;
use kivact_types::geometry3d::Matrix4;
use kivact_types::paint::{Fill, FilterOp, TransformStyle};
use kivact_types::scene::{Outline, RoundedRect};
use kivact_types::{DrawCmd, DrawList, DrawingLayer, Mix, Point, Rect, Rgba};

/// Whether `cmd` is an item of a layer: an external frame, a drawable or a
/// shader fill (its own item, which a group cannot span).
pub(crate) fn is_item(cmd: &DrawCmd) -> bool {
    match cmd {
        DrawCmd::External { .. } | DrawCmd::Drawable { .. } => true,
        DrawCmd::Rect { fill, .. } | DrawCmd::RoundedRect { fill, .. } => {
            super::layer_effects::has_shader(fill)
        }
        DrawCmd::Shape(shape) => super::layer_effects::has_shader(&shape.paint),
        _ => false,
    }
}

/// The box shadow a drawing layer stands for.
struct CastShadow {
    /// The shadowed shape, in the drawing's coordinates.
    shape: RoundedRect,
    /// The shadow's offset.
    offset: Point,
    /// The shadow's blur as a standard deviation.
    std_dev: f64,
    /// The shadow's colour, its alpha times the alpha of the shape's fill.
    color: Rgba,
}

/// The box shadow `layer` stands for when its only effect is one drop shadow and it draws one
/// solid rectangle, rounded rectangle, circle or ellipse (`rest`: the commands after its push).
fn cast_shadow(layer: &DrawingLayer, rest: &[DrawCmd]) -> Option<CastShadow> {
    if layer.transform.is_some()
        || layer.clip.is_some()
        || !layer.masks.is_empty()
        || layer.opacity != 1.0
        || layer.mix != Mix::Normal
    {
        return None;
    }
    let [
        FilterOp::DropShadow {
            offset,
            blur,
            color,
        },
    ] = layer.filter.as_slice()
    else {
        return None;
    };
    let [shape, DrawCmd::PopLayer, ..] = rest else {
        return None;
    };
    let (shape, fill) = solid_shape(shape)?;
    Some(CastShadow {
        shape,
        offset: Point {
            x: sanitize::offset(offset.x),
            y: sanitize::offset(offset.y),
        },
        std_dev: super::layer_effects::blur_px(*blur) / 2.0,
        color: Rgba {
            a: color.a * fill.a,
            ..*color
        },
    })
}

/// The shape and the solid colour of `cmd` when it draws one solid rectangle, rounded rectangle,
/// circle or ellipse without a stroke (`None` for any other command or a shape that is not finite).
fn solid_shape(cmd: &DrawCmd) -> Option<(RoundedRect, Rgba)> {
    let (shape, fill) = match cmd {
        DrawCmd::Rect { rect, fill } => (RoundedRect::new(*rect, 0.0), fill),
        DrawCmd::RoundedRect { rect, radius, fill } => (RoundedRect::new(*rect, *radius), fill),
        DrawCmd::Shape(sp) if sp.stroke.is_none() => {
            let shape = match sp.shape.to_shape() {
                Outline::Rect(r) => RoundedRect::new(r, 0.0),
                Outline::RoundedRect(r) => r,
                _ => return None,
            };
            (shape, &sp.paint)
        }
        _ => return None,
    };
    let Fill::Solid(color) = fill else {
        return None;
    };
    finite_box(&shape).then_some((shape, *color))
}

/// Whether the bounds of `shape` are finite and its size and radii are not negative.
fn finite_box(shape: &RoundedRect) -> bool {
    let r = shape.rect;
    [r.x, r.y, r.width, r.height].iter().all(|v| v.is_finite())
        && r.width >= 0.0
        && r.height >= 0.0
        && shape
            .radii
            .iter()
            .all(|c| c.x.is_finite() && c.y.is_finite() && c.x >= 0.0 && c.y >= 0.0)
}

/// Whether the `PushLayer` of `layer` opens a surface, given the commands
/// that follow it (up to its pop).
fn needs_surface(layer: &DrawingLayer, rest: &[DrawCmd]) -> bool {
    if cast_shadow(layer, rest).is_some() {
        return false;
    }
    layer.transform.is_some()
        || !layer.filter.is_empty()
        || !layer.masks.is_empty()
        || layer.mix.composite() != Composite::SourceOver
        || span_has_item(rest)
}

/// Whether the commands up to the pop that closes the layer of `rest`
/// contain an item.
fn span_has_item(rest: &[DrawCmd]) -> bool {
    let mut depth = 1usize;
    for cmd in rest {
        if is_item(cmd) {
            return true;
        }
        match cmd {
            DrawCmd::PushLayer(_) => depth += 1,
            DrawCmd::PopLayer => {
                depth -= 1;
                if depth == 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    false
}

/// Whether a scene holds a child layer and whether it holds an item (the
/// paint list's facts; a scene with neither is cacheable).
pub(crate) fn scene_layers(scene: &DrawList) -> (bool, bool) {
    let mut layer = false;
    let mut item = false;
    for (index, cmd) in scene.cmds.iter().enumerate() {
        item |= is_item(cmd);
        if let DrawCmd::PushLayer(l) = cmd {
            layer |= needs_surface(l, &scene.cmds[index + 1..]);
        }
    }
    (layer, item)
}

/// `m` about the point `origin`: the recording's transform in the scene's
/// coordinates.
fn about(m: &Matrix4, origin: Rect) -> Matrix4 {
    Matrix4::translate(-origin.x, -origin.y, 0.0)
        .then(*m)
        .then(Matrix4::translate(origin.x, origin.y, 0.0))
}

/// `r` moved by the canvas origin `origin`.
fn moved(r: Rect, origin: Rect) -> Rect {
    Rect {
        x: origin.x + r.x,
        y: origin.y + r.y,
        width: r.width,
        height: r.height,
    }
}

impl Context<'_> {
    /// Lower the layer command `cmd`; `rest` are the commands after it.
    pub(super) fn paint_layers(
        &self,
        cmd: &DrawCmd,
        rest: &[DrawCmd],
        state: &mut State<'_>,
    ) -> bool {
        let origin = state.origin;
        let rect = |r: Rect| moved(r, origin);
        match cmd {
            DrawCmd::PushLayer(layer) => {
                self.push_layer(layer, rest, state);
                true
            }
            DrawCmd::PopLayer => {
                match state.frames.last() {
                    Some(Frame::Group) => {
                        state.frames.pop();
                        state.out.push(D::PopGroup);
                    }
                    Some(Frame::Surface) => {
                        state.frames.pop();
                        state.out.close_layer();
                    }
                    Some(Frame::Flat) => {
                        state.frames.pop();
                    }
                    _ => self.drop_command(),
                }
                true
            }
            DrawCmd::PushTransform(m) => {
                let transform = m.about(Point {
                    x: origin.x,
                    y: origin.y,
                });
                state.frames.push(Frame::Transform {
                    raster: self.raster.get(),
                });
                self.raster.set(
                    self.raster.get() * crate::lower::scene::raster_zoom(transform.max_scale()),
                );
                state.out.push(D::PushTransform(transform));
                true
            }
            DrawCmd::PopTransform => {
                if let Some(&Frame::Transform { raster }) = state.frames.last() {
                    state.frames.pop();
                    self.raster.set(raster);
                    state.out.push(D::PopTransform);
                } else {
                    self.drop_command();
                }
                true
            }
            DrawCmd::PushClip(r) => {
                let command = D::PushClip(rect(*r));
                let drawn = sanitize::drawable(&command);
                if drawn {
                    state.out.push(command);
                } else {
                    self.drop_command();
                }
                state.frames.push(Frame::Clip { drawn });
                true
            }
            DrawCmd::PopClip => {
                if let Some(&Frame::Clip { drawn }) = state.frames.last() {
                    state.frames.pop();
                    if drawn {
                        state.out.push(D::PopClip);
                    }
                } else {
                    self.drop_command();
                }
                true
            }
            _ => false,
        }
    }

    /// Open the layer `layer`: a surface when it needs one, else a group.
    fn push_layer(&self, layer: &DrawingLayer, rest: &[DrawCmd], state: &mut State<'_>) {
        if let Some(shadow) = cast_shadow(layer, rest) {
            let color = state.current_tint.color(shadow.color);
            if color.a > 0.0 {
                state.out.push(D::BoxShadow(arvane_protocol::BoxShadow {
                    shape: RoundedRect {
                        rect: moved(shadow.shape.rect, state.origin),
                        radii: shadow.shape.radii,
                    },
                    offset: shadow.offset,
                    spread: 0.0,
                    std_dev: shadow.std_dev,
                    color,
                    inset: false,
                    knockout: false,
                }));
            }
            state.frames.push(Frame::Flat);
            return;
        }
        if needs_surface(layer, rest) {
            let props = self.surface(layer, state);
            state.out.open_layer(props);
            state.frames.push(Frame::Surface);
            return;
        }
        let origin = state.origin;
        let clip = layer.clip.as_ref().map(|shape| {
            shape.to_shape().translated(Point {
                x: origin.x,
                y: origin.y,
            })
        });
        state.frames.push(Frame::Group);
        state.out.push(D::PushGroup(Group {
            clip,
            opacity: layer.opacity.clamp(0.0, 1.0),
            blend: layer.mix.blend_mode(),
            blur: None,
        }));
    }

    /// The properties of the surface `layer` opens in `state`.
    fn surface(&self, layer: &DrawingLayer, state: &mut State<'_>) -> LayerProps {
        let origin = state.origin;
        let area = moved(layer.area, origin);
        let clip: Option<Outline> = layer.clip.as_ref().map(|shape| {
            shape.to_shape().translated(Point {
                x: origin.x,
                y: origin.y,
            })
        });
        let plane = if layer.transform_style == TransformStyle::Preserve3d {
            Plane::Preserve3d
        } else {
            Plane::Flat
        };
        LayerProps {
            key: state.out.next_scene_key(),
            transform: layer
                .transform
                .as_ref()
                .map_or(Matrix4::IDENTITY, |m| about(m, origin)),
            bounds: area,
            plane,
            backface: layer.backface,
            grid: RasterGrid::Target,
            effects: LayerEffects {
                opacity: layer.opacity.clamp(0.0, 1.0),
                blend: layer.mix.blend_mode(),
                composite: layer.mix.composite(),
                filters: self.layer_filters(&layer.filter),
                clip,
                masks: self.layer_masks(&layer.masks, area),
                backdrop: None,
            },
        }
    }
}
