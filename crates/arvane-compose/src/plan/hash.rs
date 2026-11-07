//! Role: FNV-1a 64-bit hashes of the plan's values: commands, paints, effects and layer parameters.
//!
//! Floats hash by their bits. Allocations shared by `Arc` hash by address
//! and length, and the record keeps them alive through [`Retained`].

use std::sync::Arc;

use kivact_types::color::Rgba;
use kivact_types::effect::ShaderUse;
use kivact_types::geometry::{Point, Rect, Transform};
use kivact_types::geometry3d::{Matrix4, Projective};
use kivact_types::paint::{BackfaceVisibility, Extend, MaskComposite, MaskMode};
use kivact_types::pixel::Pixels;
use kivact_types::render::{ExternalItem, Group};
use kivact_types::render::{
    Gradient, GradientShape, LayerEffects, LayerFilter, LayerKey, Paint, PatternSource,
    RenderBackdrop, RenderCmd, RenderMask, RenderScene, ShaderItem,
};
use kivact_types::scene::{FillRule, Shape, Stroke};

use crate::cache::Retained;

/// The FNV-1a offset basis.
const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// The FNV-1a prime.
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// An FNV-1a 64-bit accumulator.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Fnv(u64);

impl Fnv {
    /// An empty accumulator.
    pub(crate) fn new() -> Self {
        Fnv(OFFSET)
    }

    /// The hash so far.
    pub(crate) fn finish(self) -> u64 {
        self.0
    }

    /// Fold in raw bytes.
    pub(crate) fn bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(PRIME);
        }
    }

    /// Fold in a tag or a small discriminant.
    pub(crate) fn tag(&mut self, value: u8) {
        self.bytes(&[value]);
    }

    /// Fold in a 64-bit integer.
    pub(crate) fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }

    /// Fold in a length.
    pub(crate) fn len(&mut self, value: usize) {
        self.u64(value as u64);
    }

    /// Fold in a float by its bits.
    pub(crate) fn f64(&mut self, value: f64) {
        self.u64(value.to_bits());
    }

    /// Fold in a boolean.
    pub(crate) fn bool(&mut self, value: bool) {
        self.tag(u8::from(value));
    }

    /// Fold in the address and length of a shared allocation.
    pub(crate) fn arc<T: ?Sized>(&mut self, value: &Arc<T>) {
        self.u64(Arc::as_ptr(value).cast::<()>() as usize as u64);
    }

    /// Fold in a point.
    pub(crate) fn point(&mut self, p: Point) {
        self.f64(p.x);
        self.f64(p.y);
    }

    /// Fold in a rectangle.
    pub(crate) fn rect(&mut self, r: Rect) {
        self.f64(r.x);
        self.f64(r.y);
        self.f64(r.width);
        self.f64(r.height);
    }

    /// Fold in an optional rectangle.
    pub(crate) fn opt_rect(&mut self, r: Option<Rect>) {
        match r {
            Some(r) => {
                self.tag(1);
                self.rect(r);
            }
            None => self.tag(0),
        }
    }

    /// Fold in a 2D affine transform.
    pub(crate) fn transform(&mut self, t: Transform) {
        for v in [t.a, t.b, t.c, t.d, t.e, t.f] {
            self.f64(v);
        }
    }

    /// Fold in a 4×4 matrix.
    pub(crate) fn matrix4(&mut self, m: &Matrix4) {
        for row in m.m {
            for v in row {
                self.f64(v);
            }
        }
    }

    /// Fold in a projective map.
    pub(crate) fn projective(&mut self, p: &Projective) {
        for row in p.m {
            for v in row {
                self.f64(v);
            }
        }
    }

    /// Fold in a color.
    pub(crate) fn rgba(&mut self, c: Rgba) {
        self.f64(c.r);
        self.f64(c.g);
        self.f64(c.b);
        self.f64(c.a);
    }

    /// Fold in pixels by address and size, keeping the storage alive.
    pub(crate) fn pixels(&mut self, p: &Pixels, retain: &mut Vec<Retained>) {
        self.u64(u64::from(p.width));
        self.u64(u64::from(p.height));
        self.arc(p.rgba.as_shared());
        self.len(p.rgba.len());
        retain.push(Retained::Bytes(p.rgba.clone()));
    }

    /// Fold in a shaped glyph run by address, keeping it alive.
    pub(crate) fn glyphs(
        &mut self,
        run: &kivact_types::render::GlyphRunSnapshot,
        retain: &mut Vec<Retained>,
    ) {
        self.arc(run.as_shared());
        self.len(run.glyphs.len());
        retain.push(Retained::Glyphs(run.clone()));
    }
}

/// Fold in a fill rule.
fn rule(h: &mut Fnv, rule: FillRule) {
    h.tag(match rule {
        FillRule::NonZero => 0,
        _ => 1,
    });
}

/// Fold in a paint.
pub(crate) fn paint(h: &mut Fnv, p: &Paint, retain: &mut Vec<Retained>) {
    match p {
        Paint::Solid(c) => {
            h.tag(0);
            h.rgba(*c);
        }
        Paint::Gradient(g) => {
            h.tag(1);
            gradient(h, g);
        }
        Paint::Pattern(pattern) => {
            h.tag(2);
            h.rect(pattern.tile);
            h.bool(pattern.repeat_x);
            h.bool(pattern.repeat_y);
            match &pattern.source {
                PatternSource::Image { image } => {
                    h.tag(0);
                    h.pixels(image, retain);
                }
                PatternSource::Gradient(g) => {
                    h.tag(1);
                    gradient(h, g);
                }
            }
        }
    }
}

/// Fold in a gradient's geometry, stops and extension.
fn gradient(h: &mut Fnv, g: &Gradient) {
    match g.shape {
        GradientShape::Linear { start, end } => {
            h.tag(0);
            h.point(start);
            h.point(end);
        }
        GradientShape::Radial {
            center,
            radius_x,
            radius_y,
        } => {
            h.tag(1);
            h.point(center);
            h.f64(radius_x);
            h.f64(radius_y);
        }
        GradientShape::RadialTwoCircle {
            start_center,
            start_radius,
            end_center,
            end_radius,
        } => {
            h.tag(2);
            h.point(start_center);
            h.f64(start_radius);
            h.point(end_center);
            h.f64(end_radius);
        }
        GradientShape::Conic {
            center,
            start_deg,
            end_deg,
        } => {
            h.tag(3);
            h.point(center);
            h.f64(start_deg);
            h.f64(end_deg);
        }
    }
    h.len(g.stops.len());
    for (offset, color) in &g.stops {
        h.f64(*offset);
        h.rgba(*color);
    }
    h.tag(match g.extend {
        Extend::Pad => 0,
        Extend::Repeat => 1,
        Extend::Reflect => 2,
    });
}

/// Fold in a shape.
pub(crate) fn shape(h: &mut Fnv, s: &Shape) {
    match s {
        Shape::Rect(r) => {
            h.tag(0);
            h.rect(*r);
        }
        Shape::RoundedRect(r) => {
            h.tag(1);
            h.rect(r.rect);
            for radius in r.radii {
                h.f64(radius.x);
                h.f64(radius.y);
            }
        }
        Shape::Polygon { points, rule: r } => {
            h.tag(2);
            h.len(points.len());
            for p in points {
                h.point(*p);
            }
            rule(h, *r);
        }
        Shape::Path { cmds, rule: r } => {
            h.tag(3);
            h.len(cmds.len());
            for cmd in cmds {
                // Path segments are rare and small; their debug form is exact
                // (float Debug prints every distinct value).
                h.bytes(format!("{cmd:?}").as_bytes());
            }
            rule(h, *r);
        }
    }
}

/// Fold in a stroke.
fn stroke(h: &mut Fnv, s: &Stroke) {
    h.f64(s.width);
    h.tag(s.cap as u8);
    h.tag(s.join as u8);
    h.f64(s.miter_limit);
    h.len(s.dash.len());
    for d in &s.dash {
        h.f64(*d);
    }
    h.f64(s.dash_offset);
}

/// Fold in a group's clip, opacity, blend and blur.
fn group(h: &mut Fnv, g: &Group) {
    match &g.clip {
        Some(clip) => {
            h.tag(1);
            shape(h, clip);
        }
        None => h.tag(0),
    }
    h.f64(g.opacity);
    h.tag(g.blend as u8);
    match g.blur {
        Some(b) => {
            h.tag(1);
            h.f64(b);
        }
        None => h.tag(0),
    }
}

/// Fold in a command's own values and return its hash. Allocations it
/// hashes by address are pushed to `retain`.
pub(crate) fn command(cmd: &RenderCmd, retain: &mut Vec<Retained>) -> u64 {
    let mut h = Fnv::new();
    match cmd {
        RenderCmd::ImageSampled { .. }
        | RenderCmd::Image { .. }
        | RenderCmd::Glyphs { .. }
        | RenderCmd::BoxShadow(_)
        | RenderCmd::Shadow { .. } => media_command(&mut h, cmd, retain),
        RenderCmd::Rect { .. }
        | RenderCmd::RoundedRect { .. }
        | RenderCmd::RoundedRectCorners { .. }
        | RenderCmd::StrokeRect { .. }
        | RenderCmd::StrokeRoundedRectCorners { .. }
        | RenderCmd::Line { .. }
        | RenderCmd::Path { .. } => shape_command(&mut h, cmd, retain),
        _ => state_command(&mut h, cmd, retain),
    }
    h.finish()
}

/// Fold in an image, text, shadow command's own values.
fn media_command(h: &mut Fnv, cmd: &RenderCmd, retain: &mut Vec<Retained>) {
    match cmd {
        RenderCmd::Glyphs { .. } => glyphs_command(h, cmd, retain),
        RenderCmd::ImageSampled {
            image,
            rect,
            source,
            opacity,
            pixelated,
        } => {
            h.tag(0);
            h.pixels(image, retain);
            h.rect(*rect);
            h.opt_rect(*source);
            h.f64(*opacity);
            h.bool(*pixelated);
        }
        RenderCmd::Shadow {
            rect,
            radii,
            std_dev,
            color,
        } => {
            h.tag(8);
            h.rect(*rect);
            for r in radii {
                h.f64(*r);
            }
            h.f64(*std_dev);
            h.rgba(*color);
        }

        RenderCmd::Image {
            image,
            rect,
            opacity,
        } => {
            h.tag(10);
            h.pixels(image, retain);
            h.rect(*rect);
            h.f64(*opacity);
        }
        RenderCmd::BoxShadow(b) => {
            h.tag(15);
            h.rect(b.shape.rect);
            for r in b.shape.radii {
                h.f64(r.x);
                h.f64(r.y);
            }
            h.point(b.offset);
            h.f64(b.spread);
            h.f64(b.std_dev);
            h.rgba(b.color);
            h.bool(b.inset);
            h.bool(b.knockout);
        }
        _ => {}
    }
}

/// Fold in a shaped glyph run's own values.
fn glyphs_command(h: &mut Fnv, cmd: &RenderCmd, retain: &mut Vec<Retained>) {
    if let RenderCmd::Glyphs {
        run,
        origin,
        fill,
        stroke: outline,
        order,
    } = cmd
    {
        h.tag(9);
        h.glyphs(run, retain);
        h.point(*origin);
        match fill {
            Some(p) => {
                h.tag(1);
                paint(h, p, retain);
            }
            None => h.tag(0),
        }
        match outline {
            Some(o) => {
                h.tag(1);
                stroke(h, &o.stroke);
                paint(h, &o.paint, retain);
            }
            None => h.tag(0),
        }
        h.tag(*order as u8);
    }
}

/// Fold in a filled or stroked geometry command's own values.
fn shape_command(h: &mut Fnv, cmd: &RenderCmd, retain: &mut Vec<Retained>) {
    match cmd {
        RenderCmd::Path { .. } => path_command(h, cmd),
        RenderCmd::Rect { rect, fill } => {
            h.tag(1);
            h.rect(*rect);
            paint(h, fill, retain);
        }
        RenderCmd::RoundedRect { rect, radius, fill } => {
            h.tag(2);
            h.rect(*rect);
            h.f64(*radius);
            paint(h, fill, retain);
        }
        RenderCmd::RoundedRectCorners { rect, radii, fill } => {
            h.tag(3);
            h.rect(*rect);
            for r in radii {
                h.f64(*r);
            }
            paint(h, fill, retain);
        }
        RenderCmd::StrokeRect {
            rect,
            width,
            color,
            radius,
        } => {
            h.tag(4);
            h.rect(*rect);
            h.f64(*width);
            h.rgba(*color);
            h.f64(*radius);
        }
        RenderCmd::StrokeRoundedRectCorners {
            rect,
            width,
            color,
            radii,
        } => {
            h.tag(5);
            h.rect(*rect);
            h.f64(*width);
            h.rgba(*color);
            for r in radii {
                h.f64(*r);
            }
        }
        RenderCmd::Line {
            from,
            to,
            width,
            color,
        } => {
            h.tag(6);
            h.point(*from);
            h.point(*to);
            h.f64(*width);
            h.rgba(*color);
        }

        _ => {}
    }
}

/// Fold in a path command's own values.
fn path_command(h: &mut Fnv, cmd: &RenderCmd) {
    if let RenderCmd::Path {
        cmds,
        fill,
        stroke: line,
    } = cmd
    {
        h.tag(7);
        h.len(cmds.len());
        for cmd in cmds {
            h.bytes(format!("{cmd:?}").as_bytes());
        }
        match fill {
            Some(c) => {
                h.tag(1);
                h.rgba(*c);
            }
            None => h.tag(0),
        }
        match line {
            Some((w, c)) => {
                h.tag(1);
                h.f64(*w);
                h.rgba(*c);
            }
            None => h.tag(0),
        }
    }
}

/// Fold in a clip, transform, group or shape-filling command's own values.
fn state_command(h: &mut Fnv, cmd: &RenderCmd, retain: &mut Vec<Retained>) {
    match cmd {
        RenderCmd::PushClip(r) => {
            h.tag(11);
            h.rect(*r);
        }
        RenderCmd::PopClip => h.tag(12),
        RenderCmd::FillShape { shape: s, paint: p } => {
            h.tag(13);
            shape(h, s);
            paint(h, p, retain);
        }
        RenderCmd::StrokeShape {
            shape: s,
            stroke: line,
            paint: p,
        } => {
            h.tag(14);
            shape(h, s);
            stroke(h, line);
            paint(h, p, retain);
        }
        RenderCmd::PushTransform(t) => {
            h.tag(16);
            h.transform(*t);
        }
        RenderCmd::PopTransform => h.tag(17),
        RenderCmd::PushGroup(g) => {
            h.tag(18);
            group(h, g);
        }
        RenderCmd::PopGroup => h.tag(19),
        _ => {}
    }
}

/// The hash of each command of a scene, in order.
pub(crate) fn scene_hashes(scene: &RenderScene, retain: &mut Vec<Retained>) -> Vec<u64> {
    scene.commands.iter().map(|c| command(c, retain)).collect()
}

/// Fold in a layer key.
pub(crate) fn layer_key(h: &mut Fnv, key: LayerKey) {
    match key {
        LayerKey::Root => h.tag(0),
        LayerKey::Node(id) => {
            h.tag(1);
            h.u64(id.0);
        }
        LayerKey::Scene {
            node,
            slot,
            ordinal,
        } => {
            h.tag(2);
            h.u64(node.0);
            h.tag(slot);
            h.u64(u64::from(ordinal));
        }
        LayerKey::Drawable(id) => {
            h.tag(3);
            h.u64(id.0);
        }
    }
}

/// Fold in a backface visibility.
pub(crate) fn backface(h: &mut Fnv, b: BackfaceVisibility) {
    h.tag(match b {
        BackfaceVisibility::Visible => 0,
        BackfaceVisibility::Hidden => 1,
    });
}

/// Fold in a shader use: program hash, parameters, images, clock and margin.
pub(crate) fn shader_use(h: &mut Fnv, s: &ShaderUse, retain: &mut Vec<Retained>) {
    h.u64(s.program.hash);
    h.bytes(&s.params);
    for image in &s.images {
        match image {
            Some(p) => {
                h.tag(1);
                h.pixels(p, retain);
            }
            None => h.tag(0),
        }
    }
    h.f64(s.time);
    h.f64(s.margin);
}

/// Fold in one filter of a layer.
pub(crate) fn filter(h: &mut Fnv, f: &LayerFilter, retain: &mut Vec<Retained>) {
    match f {
        LayerFilter::Blur { std_dev } => {
            h.tag(0);
            h.f64(*std_dev);
        }
        LayerFilter::DropShadow {
            offset,
            std_dev,
            color,
        } => {
            h.tag(1);
            h.point(*offset);
            h.f64(*std_dev);
            h.rgba(*color);
        }
        LayerFilter::ColorMatrix(m) => {
            h.tag(2);
            for v in m.0 {
                h.f64(v);
            }
        }
        LayerFilter::Graph(graph) => {
            // The graph's values, with each image replaced by its address.
            h.tag(3);
            let mut addresses = Vec::new();
            let keyed = graph.map_images(|image| {
                let mut ih = Fnv::new();
                ih.pixels(image, &mut addresses);
                ih.finish()
            });
            retain.extend(addresses);
            h.bytes(format!("{keyed:?}").as_bytes());
        }
        LayerFilter::Shader(s) => {
            h.tag(4);
            shader_use(h, s, retain);
        }
    }
}

/// Fold in a mask.
fn mask(h: &mut Fnv, m: &RenderMask, retain: &mut Vec<Retained>) {
    scene(h, &m.scene, retain);
    h.tag(match m.mode {
        MaskMode::Alpha => 0,
        MaskMode::Luminance => 1,
    });
    h.tag(match m.composite {
        MaskComposite::Add => 0,
        MaskComposite::Subtract => 1,
        MaskComposite::Intersect => 2,
        MaskComposite::Exclude => 3,
    });
}

/// Fold in a whole scene (its commands in order).
fn scene(h: &mut Fnv, s: &RenderScene, retain: &mut Vec<Retained>) {
    h.len(s.commands.len());
    for cmd in &s.commands {
        h.u64(command(cmd, retain));
    }
}

/// Fold in the backdrop of a layer.
fn backdrop(h: &mut Fnv, b: &RenderBackdrop, retain: &mut Vec<Retained>) {
    shape(h, &b.shape);
    h.len(b.filters.len());
    for f in &b.filters {
        filter(h, f, retain);
    }
}

/// Fold in a layer's effects (opacity, blend, composite, filters, clip, masks, backdrop).
pub(crate) fn effects(h: &mut Fnv, e: &LayerEffects, retain: &mut Vec<Retained>) {
    h.f64(e.opacity);
    h.tag(e.blend as u8);
    h.tag(e.composite as u8);
    h.len(e.filters.len());
    for f in &e.filters {
        filter(h, f, retain);
    }
    match &e.clip {
        Some(clip) => {
            h.tag(1);
            shape(h, clip);
        }
        None => h.tag(0),
    }
    h.len(e.masks.len());
    for m in &e.masks {
        mask(h, m, retain);
    }
    match &e.backdrop {
        Some(b) => {
            h.tag(1);
            backdrop(h, b, retain);
        }
        None => h.tag(0),
    }
}

/// Fold in an external item's frame, geometry, clips and pixels.
pub(crate) fn external(h: &mut Fnv, e: &ExternalItem, retain: &mut Vec<Retained>) {
    h.u64(e.frame.image.0);
    h.u64(u64::from(e.frame.slot));
    h.u64(e.frame.value);
    h.rect(e.rect);
    h.opt_rect(e.source);
    h.transform(e.transform);
    h.len(e.clips.len());
    for clip in &e.clips {
        shape(h, clip);
    }
    h.f64(e.opacity);
    h.bool(e.pixelated);
    match &e.cpu {
        Some(p) => {
            h.tag(1);
            h.pixels(p, retain);
        }
        None => h.tag(0),
    }
}

/// Fold in a shader item: shader, geometry, coverage, fallback and opacity.
pub(crate) fn shader_item(h: &mut Fnv, s: &ShaderItem, retain: &mut Vec<Retained>) {
    shader_use(h, &s.shader, retain);
    h.rect(s.rect);
    h.transform(s.transform);
    scene(h, &s.coverage, retain);
    scene(h, &s.fallback, retain);
    h.f64(s.opacity);
}
