//! Role: Closure-scoped canvas groups and the `Draw::layer` command that records them.
use super::recorder::Draw;
use crate::effect::{BlendMode, Filter, Shadow, Shape, basic, filter_ops};
use crate::geometry::Rect;
use crate::paint::Mask;
use crate::transform::Transform;
use kivact_types as p;
use kivact_types::render::Composite;
use kivact_types::{BackfaceVisibility, TransformStyle};

/// What [`Draw::layer`](super::Draw::layer) applies to the commands its closure draws: a
/// transform, a clip, group opacity, blending, a filter, a mask and a
/// shadow. Effects use the element style types.
///
/// The parts apply from the outside in: the transform first (so the clip,
/// the area and the mask are in the transformed coordinates), then the
/// clip, then the group (opacity, blend or composite, filter, mask, shadow),
/// which is composited as one image. A clip outside a transform is a layer
/// around a layer. A transform with a z component or a perspective is a 3D
/// matrix that the group carries (its `transform_style` and
/// `backface_visibility` apply to it); a 2D one maps the drawing directly.
///
/// ```
/// use kivact::draw::{Layer, Shape, Transform};
/// use kivact::prelude::*;
/// use kivact::draw::{Draw};
/// use kivact::geometry::{Size};
/// use kivact::style::{pct};
/// fn badge(d: &mut Draw, size: Size) {
///     let tilted = Layer::new()
///         .transform(Transform::new().rotate(-8))
///         .clip(&Shape::circle(pct(50)))
///         .opacity(0.8);
///     d.layer(tilted, |d| d.fill_rect(size.to_rect(), ColorToken::Accent));
/// }
/// # let _ = badge;
/// ```
#[derive(Clone, Debug)]
pub struct Layer {
    // xtask-allow(state-fields): option: set by Layer::area; absent means the group, its clip shape and its mask use the whole canvas (Draw::size)
    pub(super) area: Option<Rect>,
    // xtask-allow(state-fields): option: an authored layer property; absent means the layer has none
    pub(super) transform: Option<Transform>,
    pub(super) transform_style: TransformStyle,
    pub(super) backface: BackfaceVisibility,
    // xtask-allow(state-fields): option: an authored layer property; absent means the layer has none
    pub(super) clip_rect: Option<Rect>,
    // xtask-allow(state-fields): option: an authored layer property; absent means the layer has none
    pub(super) clip: Option<Shape>,
    pub(super) opacity: f64,
    pub(super) blend: BlendMode,
    pub(super) composite: Composite,
    pub(super) filter: Filter,
    // xtask-allow(state-fields): option: an authored layer property; absent means the layer has none
    pub(super) mask: Option<Mask>,
    // xtask-allow(state-fields): option: an authored layer property; absent means the layer has none
    pub(super) shadow: Option<Shadow>,
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

impl Layer {
    /// A layer that changes nothing until its parts are set.
    pub fn new() -> Self {
        Self {
            area: None,
            transform: None,
            transform_style: TransformStyle::Flat,
            backface: BackfaceVisibility::Visible,
            clip_rect: None,
            clip: None,
            opacity: 1.0,
            blend: BlendMode::Normal,
            composite: Composite::SourceOver,
            filter: Filter::new(),
            mask: None,
            shadow: None,
        }
    }
    /// The rect a clip shape's percentages resolve against and a mask's
    /// paint fills (default: the whole area drawn in, [`Draw::size`](super::Draw::size)).
    pub fn area(mut self, area: Rect) -> Self {
        self.area = Some(area);
        self
    }
    /// Map every primitive drawn inside, including text, images, clips and
    /// regions, by the element styles' transform (percentages are of
    /// [`Draw::size`](super::Draw::size)).
    pub fn transform(mut self, t: Transform) -> Self {
        self.transform = Some(t);
        self
    }
    /// How the children of a 3D transform share one 3D space (CSS `transform-style`).
    pub fn transform_style(mut self, style: TransformStyle) -> Self {
        self.transform_style = style;
        self
    }
    /// Whether the back face of a 3D transform is shown (CSS `backface-visibility`).
    pub fn backface_visibility(mut self, backface: BackfaceVisibility) -> Self {
        self.backface = backface;
        self
    }
    /// Clip to a shape placed in the [`area`](Self::area).
    pub fn clip(mut self, shape: &Shape) -> Self {
        self.clip = Some(shape.clone());
        self
    }
    /// Clip to a rectangle (cheaper than a clip shape: no group).
    pub fn clip_rect(mut self, rect: Rect) -> Self {
        self.clip_rect = Some(rect);
        self
    }
    /// Group opacity: what is drawn inside is composited once.
    pub fn opacity(mut self, opacity: impl Into<f64>) -> Self {
        self.opacity = opacity.into();
        self
    }
    /// The blend mode with what is beneath (CSS `mix-blend-mode`). It sets
    /// the composite to source-over, as a blend does.
    pub fn blend(mut self, blend: BlendMode) -> Self {
        self.blend = blend;
        self.composite = Composite::SourceOver;
        self
    }
    /// The Porter–Duff composite with what is beneath (Canvas
    /// `globalCompositeOperation`). It sets the blend to normal, so the
    /// last of `blend` and `composite` wins.
    pub fn composite(mut self, composite: Composite) -> Self {
        self.composite = composite;
        self.blend = BlendMode::Normal;
        self
    }
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }
    /// Mask with `mask` (its layers) painted over the [`area`](Self::area).
    pub fn mask(mut self, mask: impl Into<Mask>) -> Self {
        self.mask = Some(mask.into());
        self
    }
    /// A canvas shadow, following the painted silhouette (after the
    /// layer's filter). Spread and inset belong to box shadows: they are
    /// dropped and reported as unsupported.
    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    /// Whether the group part (anything but the transform and the
    /// rectangle clip) is set.
    pub(super) fn groups(&self) -> bool {
        self.clip.is_some()
            || self.opacity != 1.0
            || self.blend != BlendMode::Normal
            || self.composite != Composite::SourceOver
            || self.filter != Filter::new()
            || self.mask.is_some()
            || self.shadow.is_some()
            || self.transform.is_some_and(|t| !is_2d(&t))
    }
}

/// Whether `t` is a 2D affine transform on a box of `size` (drawn as a
/// canvas transform, not a 3D group).
fn is_2d(t: &Transform) -> bool {
    t.to_wire([p::Len::ZERO, p::Len::ZERO], 0.0).is_2d()
}

impl Draw {
    /// Draw what `f` draws through `layer` (its transform, clip and group
    /// effects), which ends with `f`.
    pub fn layer(&mut self, layer: Layer, f: impl FnOnce(&mut Draw)) {
        let saved = self.transform;
        let transform_2d = layer.transform.filter(is_2d);
        if let Some(t) = transform_2d {
            let [a, b, c, d, e, f] = t.matrix_for(self.size);
            let m = p::Transform { a, b, c, d, e, f };
            self.transform = m.then(self.transform);
            self.cmds.push(p::DrawCmd::PushTransform(m));
        }
        if let Some(rect) = layer.clip_rect {
            self.cmds.push(p::DrawCmd::PushClip(rect));
        }
        let group = layer.groups();
        if group {
            self.push_group(&layer);
        }
        f(self);
        if group {
            self.cmds.push(p::DrawCmd::PopLayer);
        }
        if layer.clip_rect.is_some() {
            self.cmds.push(p::DrawCmd::PopClip);
        }
        if transform_2d.is_some() {
            self.transform = saved;
            self.cmds.push(p::DrawCmd::PopTransform);
        }
    }

    pub(super) fn push_group(&mut self, layer: &Layer) {
        let area = layer.area.unwrap_or(self.size.to_rect());
        let mut filter = layer.filter.clone();
        if let Some(shadow) = &layer.shadow {
            if shadow.spread != 0.0 || shadow.inset {
                self.unsupported += 1;
            }
            filter =
                filter.drop_shadow(shadow.offset.x, shadow.offset.y, shadow.blur, shadow.color);
        }
        // A 3D transform is carried by the group: its matrix about the canvas origin.
        let transform = layer.transform.filter(|t| !is_2d(t)).map(|t| {
            t.to_wire([p::Len::ZERO, p::Len::ZERO], 0.0)
                .matrix(self.size)
        });
        self.cmds
            .push(p::DrawCmd::PushLayer(Box::new(p::DrawingLayer {
                area,
                clip: layer.clip.as_ref().map(|shape| basic(area, shape)),
                opacity: layer.opacity,
                blend: layer.blend,
                composite: layer.composite,
                filter: filter_ops(&filter, &self.theme),
                masks: layer
                    .mask
                    .as_ref()
                    .map(|m| m.wire(&self.theme))
                    .unwrap_or_default(),
                transform,
                transform_style: layer.transform_style,
                backface: layer.backface,
            })));
    }
}
