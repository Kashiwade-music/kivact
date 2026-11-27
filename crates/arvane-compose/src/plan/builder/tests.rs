//! Role: Tests of the plan builder's hashing of raster scenes across frames.

use arvane_protocol::{
    CompositeLayer, Composition, LayerEffects, LayerItem, LayerKey, Plane, RasterGrid, RasterPaint,
    RenderCmd, RenderScene,
};
use kivact_types::Matrix4;
use kivact_types::Rgba;
use kivact_types::geometry::Rect;
use kivact_types::paint::BackfaceVisibility;
use kivact_types::scene::Outline;

use super::build;
use crate::cache::CacheIndex;

/// The commands of the raster scene of the test.
const N: usize = 500;
/// The window size in px.
const SIZE: (u32, u32) = (1024, 1024);

/// A composition of one root layer whose only item is a raster scene of `N`
/// squares; the square `recolored` (an index) is drawn in another colour.
fn root_only(content_version: u64, recolored: Option<usize>) -> Composition {
    let commands = (0..N)
        .map(|i| {
            let (col, row) = (i % 25, i / 25);
            let red = if recolored == Some(i) { 0.9 } else { 0.2 };
            RenderCmd::FillShape {
                shape: Outline::Rect(Rect::new(col as f64 * 40.0, row as f64 * 40.0, 30.0, 30.0)),
                paint: RasterPaint::Solid(Rgba {
                    r: red,
                    g: 0.5,
                    b: 0.3,
                    a: 1.0,
                }),
            }
        })
        .collect();
    let root = CompositeLayer {
        key: LayerKey::Root,
        transform: Matrix4::IDENTITY,
        bounds: Rect::new(0.0, 0.0, f64::from(SIZE.0), f64::from(SIZE.1)),
        plane: Plane::Flat,
        backface: BackfaceVisibility::Visible,
        grid: RasterGrid::Target,
        effects: LayerEffects::default(),
        content_version,
        items: vec![LayerItem::Raster(RenderScene { commands })],
    };
    Composition {
        layers: vec![root],
        exports: Vec::new(),
    }
}

/// Plan one frame of `composition` against `cache` and store its records.
/// Returns the commands hashed by the plan and the root layer's signature.
fn frame(cache: &mut CacheIndex, composition: &Composition) -> (usize, u64) {
    let (hashed, frame, chunks, layers, forgets) = {
        let plan = build(composition, SIZE, 1.0, cache, false).expect("the frame plans");
        (
            plan.hashed_commands,
            plan.frame,
            plan.chunk_records,
            plan.layer_records,
            plan.chunk_forgets,
        )
    };
    cache.store(frame, chunks, layers, &forgets);
    let signature = cache
        .layer(&LayerKey::Root)
        .expect("the root layer is recorded")
        .signature;
    (hashed, signature)
}

/// Each raster command is hashed once in a frame that draws it, and a layer
/// whose content version is kept is not hashed at all.
#[test]
fn each_scene_is_hashed_once_and_a_kept_layer_never() {
    let mut cache = CacheIndex::default();
    let (hashed, first) = frame(&mut cache, &root_only(1, None));
    assert_eq!(hashed, N, "frame 1 hashes each command once");
    let (hashed, kept) = frame(&mut cache, &root_only(1, None));
    assert_eq!(hashed, 0, "the same content_version hashes nothing");
    assert_eq!(kept, first, "the same content keeps the signature");
    let (hashed, changed) = frame(&mut cache, &root_only(2, Some(7)));
    assert_eq!(hashed, N, "a new version hashes each command once");
    assert_ne!(changed, first, "a new version changes the signature");
}
