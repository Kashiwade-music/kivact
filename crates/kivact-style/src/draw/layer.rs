//! Role: Closure-scoped canvas groups and the `Draw::layer` command that records them.
use super::recorder::{Affine, Draw};
use crate::effect::{BlendMode, Filter, Shadow, Shape, basic, filter_ops};
use crate::geometry::Rect;
use crate::paint::{Paint, fill_layer};
use crate::transform::Transform;
use kivact_wire as p;

/// What [`Draw::layer`](super::Draw::layer) applies to the commands its closure draws: a
/// transform, a clip, group opacity, blending, a filter, a mask and a
/// shadow. Effects use the element style types.
///
/// The parts apply from the outside in: the transform first (so the clip,
/// the area and the mask are in the transformed coordinates), then the
/// clip, then the group (opacity, blend, filter, mask, shadow), which is
/// composited as one image. A clip outside a transform is a layer around a
/// layer.
///
/// ```
/// use kivact::draw::{Layer, Shape, Transform};
/// use kivact::prelude::*;
/// use kivact::draw::{Draw, Size};
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
    pub(super) area: Option<Rect>,
    pub(super) transform: Option<Transform>,
    pub(super) clip_rect: Option<Rect>,
    pub(super) clip: Option<Shape>,
    pub(super) opacity: f32,
    pub(super) blend: BlendMode,
    pub(super) filter: Filter,
    pub(super) mask: Option<Paint>,
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
            clip_rect: None,
            clip: None,
            opacity: 1.0,
            blend: BlendMode::Normal,
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
        self.opacity = opacity.into() as f32;
        self
    }
    pub fn blend(mut self, blend: BlendMode) -> Self {
        self.blend = blend;
        self
    }
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }
    /// Mask with `mask` painted over the [`area`](Self::area).
    pub fn mask(mut self, mask: impl Into<Paint>) -> Self {
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
            || self.filter != Filter::new()
            || self.mask.is_some()
            || self.shadow.is_some()
    }
}

impl Draw {
    /// Draw what `f` draws through `layer` (its transform, clip and group
    /// effects), which ends with `f`.
    pub fn layer(&mut self, layer: Layer, f: impl FnOnce(&mut Draw)) {
        let saved = self.transform;
        if let Some(t) = layer.transform {
            let m = Affine {
                m: t.matrix_for(self.size),
            };
            self.transform = m.then(self.transform);
            self.cmds.push(p::SceneCmd::PushTransform(m.m));
        }
        if let Some(rect) = layer.clip_rect {
            self.cmds.push(p::SceneCmd::PushClip(rect));
        }
        let group = layer.groups();
        if group {
            self.push_group(&layer);
        }
        f(self);
        if group {
            self.cmds.push(p::SceneCmd::PopLayer);
        }
        if layer.clip_rect.is_some() {
            self.cmds.push(p::SceneCmd::PopClip);
        }
        if layer.transform.is_some() {
            self.transform = saved;
            self.cmds.push(p::SceneCmd::PopTransform);
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
        self.cmds
            .push(p::SceneCmd::PushLayer(Box::new(p::DrawingLayer {
                area,
                clip: layer.clip.as_ref().map(|shape| basic(area, shape)),
                opacity: layer.opacity,
                blend: layer.blend,
                filter: filter_ops(&filter, &self.theme),
                mask: layer.mask.as_ref().map(|p| fill_layer(p, &self.theme)),
            })));
    }
}
