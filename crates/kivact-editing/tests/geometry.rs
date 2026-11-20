//! Role: Tests that a caret stop is never inside a shaping cluster.

use kivact_editing::TextGeometry;
use kivact_types::{Point, Rect, ShapedText, TextCaret};

/// `ffix` shaped with `ffi` as one cluster (a ligature): carets 10 px
/// apart at every scalar, stops at scalars 0, 3 and 4.
fn ligature() -> ShapedText {
    ShapedText {
        carets: (0..=4u32)
            .map(|offset| TextCaret {
                offset,
                rect: Rect {
                    x: 10.0 * f64::from(offset),
                    y: 0.0,
                    width: 1.0,
                    height: 20.0,
                },
            })
            .collect(),
        cluster_boundaries: vec![0, 3, 4],
        ..ShapedText::default()
    }
}

#[test]
fn a_hit_or_a_move_never_lands_inside_a_shaping_cluster() {
    let shaped = ligature();
    let geometry = TextGeometry::new(&shaped, "ffix");
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 20.0,
    };
    let hit = |x: f64| geometry.hit_scalar_in_line(Point { x, y: 0.0 }, 0..4, bounds);
    assert_eq!(
        hit(9.0),
        0,
        "the caret at 10 px is inside the cluster: the start is the nearest stop"
    );
    assert_eq!(
        hit(21.0),
        3,
        "the caret at 20 px is inside the cluster: its end is the nearest stop"
    );
    assert_eq!(hit(38.0), 4);
    assert_eq!(
        geometry.next_boundary(0),
        3,
        "a move steps over the whole cluster"
    );
    assert_eq!(geometry.prev_boundary(4), 3);
}
