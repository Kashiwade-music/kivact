//! Role: Verify that path segments hash by their values, as cheaply as other commands.

use super::scene_hashed;
use arvane_protocol::{RasterPaint, RenderCmd, RenderScene};
use kivact_types::color::Rgba;
use kivact_types::geometry::{Point, Rect};
use kivact_types::scene::{FillRule, Outline, PathCmd, Stroke};
use std::time::Instant;

/// Opaque black.
const BLACK: Rgba = Rgba {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// A scene of one stroked path of `n` cubic segments, and a scene of `n` rects.
fn scenes(n: usize) -> (RenderScene, RenderScene) {
    let p = |i: usize, k: f64| Point::new(i as f64 + k, (i as f64 * 0.37).sin() * 50.0 + k);
    let mut cmds = vec![PathCmd::MoveTo(p(0, 0.0))];
    cmds.extend((0..n).map(|i| PathCmd::CubicTo(p(i, 0.25), p(i, 0.5), p(i, 1.0))));
    let path = RenderScene {
        commands: vec![RenderCmd::StrokeShape {
            shape: Outline::Path {
                cmds,
                rule: FillRule::NonZero,
            },
            stroke: Stroke::new(2.0),
            paint: RasterPaint::Solid(BLACK),
        }],
    };
    let rects = RenderScene {
        commands: (0..n)
            .map(|i| RenderCmd::Rect {
                rect: Rect::new(i as f64, 0.0, 4.0, 4.0),
                fill: RasterPaint::Solid(BLACK),
            })
            .collect(),
    };
    (path, rects)
}

/// Hashing a path of 4 000 segments costs at most ten times hashing 4 000 rects (best of 5).
#[test]
fn a_long_path_hashes_as_fast_as_its_segments() {
    let (path, rects) = scenes(4000);
    let best = |scene: &RenderScene| {
        (0..5)
            .map(|_| {
                let t = Instant::now();
                std::hint::black_box(scene_hashed(scene));
                t.elapsed()
            })
            .min()
            .expect("five runs")
    };
    let (p, r) = (best(&path), best(&rects));
    assert!(
        p <= r * 10,
        "a path segment costs {:.1} rects",
        p.as_secs_f64() / r.as_secs_f64()
    );
}

/// Two paths that differ in one segment hash differently; equal ones hash equally.
#[test]
fn path_hashes_follow_the_values() {
    let (a, _) = scenes(10);
    let mut b = a.clone();
    if let RenderCmd::StrokeShape {
        shape: Outline::Path { cmds, .. },
        ..
    } = &mut b.commands[0]
    {
        cmds[3] = PathCmd::CubicTo(
            Point::new(1.0, 2.0),
            Point::new(3.0, 4.0),
            Point::new(5.0, 6.0),
        );
    }
    assert_eq!(scene_hashed(&a).hashes, scene_hashed(&a.clone()).hashes);
    assert_ne!(scene_hashed(&a).hashes, scene_hashed(&b).hashes);
}
