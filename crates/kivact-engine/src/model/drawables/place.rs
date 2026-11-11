//! Role: Where a canvas draws each drawable: its map, clips and backface.
//!
//! The host's two scenes are read in order (slot 0, then slot 1). The
//! transforms, layers and clips open around a `Drawable` command make the
//! placement: its map folds the open transforms from the innermost out
//! (`Projective::then` applies its first argument first), a clip keeps the
//! map from the host canvas into the space it was declared in, and the
//! backface is decided by the enclosing layers' matrices.
//!
//! Invariants:
//! - The first `Drawable` command of an id wins (slot 0 before slot 1, then
//!   command order); later ones are reported as duplicates.
//! - The scopes are balanced in a recording; a pop removes the innermost.

use std::collections::BTreeMap;

use kivact_types::geometry3d::{Matrix4, Projective};
use kivact_types::scene::Shape;
use kivact_types::{
    BackfaceVisibility, DrawCmd, DrawList, DrawableId, DrawingLayer, Point, Transform,
    TransformStyle,
};

/// A drawable's placement in the host canvas.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DrawnPlacement {
    /// Drawable box space → host canvas (padding box) space.
    pub(crate) map: Projective,
    /// The scene slot the drawable is drawn in (0: `draw`, 1: `draw_over`).
    pub(crate) slot: u8,
    /// The command index in its scene (a larger one is drawn later).
    pub(crate) order: u32,
    /// Clips open where it was drawn, each with the map from host canvas
    /// space to the space the clip was declared in; all apply.
    pub(crate) clips: Vec<(Projective, Shape)>,
    /// An enclosing layer with backface hidden shows its back.
    pub(crate) back_hidden: bool,
}

/// The placements of one host's scenes.
#[derive(Default)]
pub(crate) struct Placements {
    /// The placement of each drawable that is drawn (its first drawing).
    pub(crate) by_id: BTreeMap<DrawableId, DrawnPlacement>,
}

/// One open scope of the walk of a scene.
enum Frame {
    /// A transform (affine, as a homography), and whether the nearest open
    /// layer preserves 3D, so the transform is in its 3D context.
    Transform { map: Projective, in_3d: bool },
    /// A layer: its matrix, its 3D context and its clip.
    Layer(LayerFrame),
    /// A clip declared at this depth, in the host canvas to clip space map.
    Clip(Projective, Shape),
}

/// What a layer contributes to the walk.
struct LayerFrame {
    /// The layer's own 4×4 transform (identity without one).
    own: Matrix4,
    /// The accumulated 4×4 of its 3D context (`own` in a flat parent).
    context: Matrix4,
    // xtask-allow(state-fields): independent: push_layer reads it from the nearest open layer's preserves (inside_3d); preserves is this layer's own transform_style, so a flat or preserving layer sits in a flat or 3D parent
    /// Whether the layer is inside a `Preserve3d` parent.
    in_3d: bool,
    /// The layer's backface visibility.
    backface: BackfaceVisibility,
    // xtask-allow(state-fields): independent: push_layer takes it from this layer's transform_style; in_3d is the parent layer's, so a flat or preserving layer sits in a flat or 3D parent
    /// Whether the layer's children share a 3D context.
    preserves: bool,
}

/// The placements of the drawables of `scenes` (slot 0 and slot 1).
pub(crate) fn drawable_placements(scenes: [Option<&DrawList>; 2]) -> Placements {
    let mut out = Placements::default();
    for (slot, scene) in scenes.into_iter().enumerate() {
        let Some(scene) = scene else { continue };
        let mut frames: Vec<Frame> = Vec::new();
        for (index, cmd) in scene.cmds.iter().enumerate() {
            walk_cmd(cmd, slot as u8, index as u32, &mut frames, &mut out);
        }
    }
    out
}

/// Apply one command of the walk: open or close a scope, or place a drawable.
fn walk_cmd(cmd: &DrawCmd, slot: u8, order: u32, frames: &mut Vec<Frame>, out: &mut Placements) {
    match cmd {
        DrawCmd::PushTransform(t) => frames.push(Frame::Transform {
            map: Projective::from_affine(*t),
            in_3d: inside_3d(frames),
        }),
        DrawCmd::PopTransform => pop(frames, |f| matches!(f, Frame::Transform { .. })),
        DrawCmd::PushClip(rect) => {
            let map = fold(frames).invert();
            if let Some(map) = map {
                frames.push(Frame::Clip(map, Shape::Rect(*rect)));
            }
        }
        DrawCmd::PopClip => pop(frames, |f| matches!(f, Frame::Clip(..))),
        DrawCmd::PushLayer(layer) => push_layer(layer, frames),
        DrawCmd::PopLayer => pop(frames, |f| matches!(f, Frame::Layer(_))),
        DrawCmd::Drawable { id, at } => place(*id, *at, slot, order, frames, out),
        _ => {}
    }
}

/// Remove the innermost frame of the kind `is`; the scopes are balanced, so
/// a mismatch removes nothing.
fn pop(frames: &mut Vec<Frame>, is: impl Fn(&Frame) -> bool) {
    if frames.last().is_some_and(is) {
        frames.pop();
    }
}

/// Open a layer: its transform and its clip (the clip is in the layer's space).
fn push_layer(layer: &DrawingLayer, frames: &mut Vec<Frame>) {
    let own = layer.transform.unwrap_or(Matrix4::IDENTITY);
    let in_3d = inside_3d(frames);
    let context = if in_3d {
        frames
            .iter()
            .rev()
            .find_map(|f| match f {
                Frame::Layer(l) => Some(l.context),
                _ => None,
            })
            .unwrap_or(Matrix4::IDENTITY)
            .then(own)
    } else {
        own
    };
    frames.push(Frame::Layer(LayerFrame {
        own,
        context,
        in_3d,
        backface: layer.backface,
        preserves: layer.transform_style == TransformStyle::Preserve3d,
    }));
    if let Some(clip) = &layer.clip
        && let Some(map) = fold(frames).invert()
    {
        frames.push(Frame::Clip(map, crate::lower::scene_shape::shape(clip)));
    }
}

/// Whether a frame pushed now is inside a 3D context: the nearest open layer
/// preserves 3D.
fn inside_3d(frames: &[Frame]) -> bool {
    frames
        .iter()
        .rev()
        .find_map(|f| match f {
            Frame::Layer(l) => Some(l.preserves),
            _ => None,
        })
        .unwrap_or(false)
}

/// The map from the innermost space out to the host canvas of the open frames
/// (`then` applies the inner frame first). A run of frames in a 3D context
/// accumulates its 4 × 4 matrices and is flattened where the context ends;
/// the others are flattened at once.
fn fold(frames: &[Frame]) -> Projective {
    let mut flat = Projective::IDENTITY;
    let mut deep = Matrix4::IDENTITY;
    for frame in frames.iter().rev() {
        let (matrix, in_3d) = match frame {
            Frame::Transform { map, in_3d } => (Matrix4::from_projective(*map), *in_3d),
            Frame::Layer(l) => (l.own, l.in_3d),
            Frame::Clip(..) => continue,
        };
        if in_3d {
            deep = deep.then(matrix);
        } else {
            flat = flat.then(deep.then(matrix).flatten());
            deep = Matrix4::IDENTITY;
        }
    }
    flat.then(deep.flatten())
}

/// Place the drawable `id` drawn at `at` under the open frames.
fn place(id: DrawableId, at: Point, slot: u8, order: u32, frames: &[Frame], out: &mut Placements) {
    if out.by_id.contains_key(&id) {
        // A drawable drawn again: only its first drawing places it.
        return;
    }
    let translate = Projective::from_affine(Transform {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: at.x,
        f: at.y,
    });
    let map = translate.then(fold(frames));
    let clips = frames
        .iter()
        .filter_map(|f| match f {
            Frame::Clip(map, shape) => Some((*map, shape.clone())),
            _ => None,
        })
        .collect();
    let back_hidden = frames.iter().any(|f| match f {
        Frame::Layer(l) => {
            let matrix = if l.in_3d { l.context } else { l.own };
            l.backface == BackfaceVisibility::Hidden && matrix.shows_back()
        }
        _ => false,
    });
    out.by_id.insert(
        id,
        DrawnPlacement {
            map,
            slot,
            order,
            clips,
            back_hidden,
        },
    );
}
