//! Role: Compositor layers: a node's look and the facts that decide them.
//!
//! Invariants:
//! - A node is a layer for its own reasons: it is a 3D or a flat preserve-3d
//!   member, it has a filter, a mask or a backdrop filter, its `will-change`
//!   is set, it moves (a running transition or a layout motion), or it is an
//!   overlay or fixed box.
//! - An inline group that holds a layer or an item is a layer; so is a
//!   backdrop root that holds a backdrop filter, and an isolated node that
//!   holds a blend mode and a layer. The inline groups it replaces never
//!   span an item.
//! - The facts come from one bottom-up pass before the walk, from the looks
//!   the walk uses, so no node's layer depends on its layout location.
//! - The paint list and the hit test read the same facts, so a 3D context
//!   has the same planes when drawn and when hit.

use crate::lower::effects;
use crate::lower::sanitize::{extent, offset, unit};
use kivact_types::paint::{BackfaceVisibility, Fill, FillImage, TransformStyle};
use kivact_types::pixel::BlendMode;
use kivact_types::{Edges, ElementTransform, FilterOp, Isolation, PaintProps, WillChange};

/// The extent of what reaches everywhere (finite, so unions keep it).
pub(crate) const EVERYWHERE: f64 = f64::MAX / 4.0;

/// What a node's paint says about where it draws, sanitized like
/// `paint::sanitize_paint` (read in place: most nodes have no state
/// styles, and their paint is not copied).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayerTraits {
    pub(crate) opacity: f64,
    /// How far its outer shadows and focus ring reach outside its box.
    pub(crate) overflow: f64,
    /// The widths of its visible border sides (canvas scenes start inside
    /// them).
    pub(crate) border: Edges<f64>,
    /// How far its layer filters draw beyond what they filter.
    pub(crate) fx: f64,
    /// How many of a background, a border, shadows and a ring it draws.
    pub(crate) items: u8,
    /// Its `will-change` value.
    pub(crate) will_change: WillChange,
    /// Its yes/no facts (see the methods named after them).
    pub(crate) flags: LookFlags,
}

/// The yes/no facts of a look, one bit each (the old field names are its methods).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LookFlags(u16);

impl LookFlags {
    pub(crate) const GROUPED: Self = Self(1);
    pub(crate) const CLIP: Self = Self(1 << 1);
    pub(crate) const ROUNDED: Self = Self(1 << 2);
    pub(crate) const FILTERED: Self = Self(1 << 3);
    pub(crate) const MASKED: Self = Self(1 << 4);
    pub(crate) const BACKDROPPED: Self = Self(1 << 5);
    pub(crate) const BLENDED: Self = Self(1 << 6);
    pub(crate) const ISOLATED: Self = Self(1 << 7);
    pub(crate) const PRESERVES: Self = Self(1 << 8);
    pub(crate) const FLAT: Self = Self(1 << 9);
    pub(crate) const TRANSFORMED: Self = Self(1 << 10);
    pub(crate) const PERSPECTIVE: Self = Self(1 << 11);
    pub(crate) const BACKFACE_HIDDEN: Self = Self(1 << 12);
    pub(crate) const SHADER: Self = Self(1 << 13);

    /// Whether `flag` is set.
    pub(crate) const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }

    /// These flags with `flag` set when `on`.
    pub(crate) const fn with(self, flag: Self, on: bool) -> Self {
        if on { Self(self.0 | flag.0) } else { self }
    }
}

const _: () = assert!(
    std::mem::size_of::<LayerTraits>() <= 64,
    "LayerTraits grew: box the new rare field"
);

/// What a subtree holds that decides its layers, one bit each (the old
/// field names are its methods): whether the node itself is a layer, and
/// what its subtree contains.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SubtreeFacts(u8);

impl SubtreeFacts {
    /// The node is a compositor layer.
    pub(crate) const LAYER: Self = Self(1);
    /// A layer is in the subtree (the node's own included).
    pub(crate) const HAS_LAYER: Self = Self(1 << 1);
    /// An external frame or a shader is in the subtree.
    pub(crate) const HAS_ITEM: Self = Self(1 << 2);
    /// A backdrop filter is in the subtree.
    pub(crate) const HAS_BACKDROP: Self = Self(1 << 3);
    /// A non-normal blend mode is in the subtree.
    pub(crate) const HAS_BLEND: Self = Self(1 << 4);

    /// These facts with `fact` set when `on`.
    pub(crate) const fn with(self, fact: Self, on: bool) -> Self {
        if on { Self(self.0 | fact.0) } else { self }
    }

    /// The node is a compositor layer.
    pub(crate) const fn layer(self) -> bool {
        self.0 & Self::LAYER.0 != 0
    }

    /// A layer is in the subtree (the node's own included).
    pub(crate) const fn has_layer(self) -> bool {
        self.0 & Self::HAS_LAYER.0 != 0
    }

    /// An external frame or a shader is in the subtree.
    pub(crate) const fn has_item(self) -> bool {
        self.0 & Self::HAS_ITEM.0 != 0
    }

    /// A backdrop filter is in the subtree.
    pub(crate) const fn has_backdrop(self) -> bool {
        self.0 & Self::HAS_BACKDROP.0 != 0
    }

    /// A non-normal blend mode is in the subtree.
    pub(crate) const fn has_blend(self) -> bool {
        self.0 & Self::HAS_BLEND.0 != 0
    }
}

const _: () = assert!(
    std::mem::size_of::<SubtreeFacts>() <= 1,
    "SubtreeFacts grew: it is a set of bits"
);

impl LayerTraits {
    /// Its effects need a group (clip path, blend), whatever its opacity.
    pub(crate) fn grouped(&self) -> bool {
        self.flags.contains(LookFlags::GROUPED)
    }

    /// It clips its children to its padding box.
    pub(crate) fn clip(&self) -> bool {
        self.flags.contains(LookFlags::CLIP)
    }

    /// Its corners are rounded.
    pub(crate) fn rounded(&self) -> bool {
        self.flags.contains(LookFlags::ROUNDED)
    }

    /// It has a CSS filter (it needs a layer).
    pub(crate) fn filtered(&self) -> bool {
        self.flags.contains(LookFlags::FILTERED)
    }

    /// It has a mask.
    pub(crate) fn masked(&self) -> bool {
        self.flags.contains(LookFlags::MASKED)
    }

    /// It has a backdrop filter.
    pub(crate) fn backdropped(&self) -> bool {
        self.flags.contains(LookFlags::BACKDROPPED)
    }

    /// Its blend mode is not normal.
    pub(crate) fn blended(&self) -> bool {
        self.flags.contains(LookFlags::BLENDED)
    }

    /// It isolates its group (`isolation: isolate`).
    pub(crate) fn isolated(&self) -> bool {
        self.flags.contains(LookFlags::ISOLATED)
    }

    /// Its `transform-style` is `preserve-3d`.
    pub(crate) fn preserves(&self) -> bool {
        self.flags.contains(LookFlags::PRESERVES)
    }

    /// Its element transform has no z component and no perspective.
    pub(crate) fn flat(&self) -> bool {
        self.flags.contains(LookFlags::FLAT)
    }

    /// Its element transform is not the identity.
    pub(crate) fn transformed(&self) -> bool {
        self.flags.contains(LookFlags::TRANSFORMED)
    }

    /// Its children are in a perspective.
    pub(crate) fn perspective(&self) -> bool {
        self.flags.contains(LookFlags::PERSPECTIVE)
    }

    /// Its back face is hidden.
    pub(crate) fn backface_hidden(&self) -> bool {
        self.flags.contains(LookFlags::BACKFACE_HIDDEN)
    }

    /// One of its background layers is a shader.
    pub(crate) fn shader(&self) -> bool {
        self.flags.contains(LookFlags::SHADER)
    }
}

impl LayerTraits {
    /// The look of the paint `p`.
    pub(crate) fn of(p: &PaintProps) -> Self {
        let shadows = p
            .shadows
            .iter()
            .filter(|s| !s.inset)
            .map(|s| {
                // The blur's gaussian (standard deviation blur / 2) is
                // drawn at most arvane_protocol::SHADOW_REACH deviations out.
                let blur = extent(s.blur) / 2.0 * arvane_protocol::SHADOW_REACH;
                offset(s.offset.x).abs().max(offset(s.offset.y).abs())
                    + offset(s.spread).max(0.0)
                    + blur
            })
            .fold(0.0, f64::max);
        let ring = p.ring.map_or(0.0, |r| extent(r.width));
        let filter: &[FilterOp] = p.effects.as_ref().map_or(&[], |e| &e.filter);
        let border = crate::lower::shape::border_widths(p.border.as_ref());
        let items = [
            p.background.is_some(),
            border != Edges::all(0.0),
            !p.shadows.is_empty(),
            p.ring.is_some(),
        ]
        .iter()
        .filter(|x| **x)
        .count() as u8;
        let effects = p.effects.as_deref();
        LayerTraits {
            opacity: unit(p.opacity, 1.0),
            overflow: shadows.max(ring),
            border: Edges {
                top: extent(border.top),
                right: extent(border.right),
                bottom: extent(border.bottom),
                left: extent(border.left),
            },
            fx: effects::filter_reach(filter).min(EVERYWHERE),
            items,
            will_change: p.will_change,
            flags: LookFlags::default()
                .with(LookFlags::GROUPED, effects::grouped(p))
                .with(LookFlags::CLIP, p.clip)
                .with(LookFlags::ROUNDED, !p.radius.is_zero())
                .with(LookFlags::FILTERED, effects.is_some_and(|e| !e.filter.is_empty()))
                .with(LookFlags::MASKED, effects.is_some_and(|e| !e.masks.is_empty()))
                .with(LookFlags::BACKDROPPED, effects.is_some_and(|e| !e.backdrop.is_empty()))
                .with(LookFlags::BLENDED, effects.is_some_and(|e| e.blend != BlendMode::Normal))
                .with(LookFlags::ISOLATED, p.isolation == Isolation::Isolate)
                .with(LookFlags::PRESERVES, p.transform_style == TransformStyle::Preserve3d)
                .with(LookFlags::FLAT, p.transform.is_2d())
                .with(LookFlags::TRANSFORMED, p.transform != ElementTransform::NONE)
                .with(LookFlags::PERSPECTIVE, p.perspective.is_some())
                .with(LookFlags::BACKFACE_HIDDEN, p.backface == BackfaceVisibility::Hidden)
                .with(
                    LookFlags::SHADER,
                    p.background.as_ref().is_some_and(|fill| matches!(fill, Fill::Layers(layers) if layers.iter().any(|l| matches!(l.image, FillImage::Shader(_))))),
                ),
        }
    }

    /// Whether the node's effects group its subtree (CSS: the grouping
    /// properties that flatten a preserve-3d context).
    pub(crate) fn grouping(&self) -> bool {
        self.clip()
            || self.opacity < 1.0
            || self.filtered()
            || self.masked()
            || self.backdropped()
            || self.grouped()
            || self.isolated()
    }

    /// Whether the node is a flat member of a preserve-3d context: its
    /// own plane is shared with its parent.
    pub(crate) fn plane_3d(&self) -> bool {
        self.preserves() && !self.grouping()
    }
}
