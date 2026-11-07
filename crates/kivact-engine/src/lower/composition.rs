//! Role: The composition a lowered scene is handed to Arvane in.
//!
//! The paint model draws a whole frame as one raster scene; layerization
//! into compositor layers is a later step. Until then every frame is one
//! root layer holding that scene.

use kivact_types::geometry3d::Matrix4;
use kivact_types::paint::BackfaceVisibility;
use kivact_types::render::{
    CompositeLayer, Composition, LayerEffects, LayerItem, LayerKey, Plane, RasterGrid, RenderScene,
};
use kivact_types::{Rect, Size};

/// The composition of one root layer of `size` logical pixels (identity
/// transform, no effects, no exports) drawing `scene`.
pub(crate) fn root_composition(scene: RenderScene, size: Size) -> Composition {
    Composition {
        layers: vec![CompositeLayer {
            key: LayerKey::Root,
            transform: Matrix4::IDENTITY,
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            },
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects::default(),
            items: vec![LayerItem::Raster(scene)],
        }],
        exports: Vec::new(),
    }
}
