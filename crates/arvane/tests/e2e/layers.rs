//! Role: Layer rendering contract tests.
//!
//! Layers: group opacity, clips to any shape, blend modes, blur and drop
//! shadow filters, gradient masks, and how groups must nest. A group is a
//! compositor layer (`LayerEffects`) that the root layer draws as a child.

use kivact_types::render::{Composition, LayerEffects, LayerFilter, LayerItem, Paint, RenderCmd};

use crate::support::{
    RED, WHITE, assert_near, color, color_alpha, fill, gray, layer, node_key, open, pixel, pt,
    raster, rect, root_layer, show_composition,
};

/// The root item drawing child layer `index`, with no clip.
fn child(index: u32) -> LayerItem {
    LayerItem::Layer {
        index,
        clips: Vec::new(),
    }
}

#[test]
fn group_opacity_fades_the_group_not_each_command() {
    let mut window = open(100.0, 40.0);
    // The same two rects, each half transparent, drawn after the group.
    let mut loose = Vec::new();
    for x in [50.0, 70.0] {
        loose.push(RenderCmd::Rect {
            rect: rect(x, 0.0, 30.0, 40.0),
            fill: Paint::Solid(color_alpha(RED, 0.5)),
        });
    }
    let group_effects = LayerEffects {
        opacity: 0.5,
        ..LayerEffects::default()
    };
    let group = layer(
        node_key(1),
        rect(0.0, 0.0, 100.0, 40.0),
        group_effects,
        vec![raster(vec![
            fill(0.0, 0.0, 30.0, 40.0, RED),
            fill(20.0, 0.0, 30.0, 40.0, RED),
        ])],
    );
    show_composition(&mut window, move |size| Composition {
        layers: vec![root_layer(size, vec![child(1), raster(loose)]), group],
        exports: Vec::new(),
    });
    assert_near(pixel(&window, 10, 20), 0xff8080, 2, "one rect in the group");
    assert_near(
        pixel(&window, 25, 20),
        0xff8080,
        2,
        "the overlap is not darker",
    );
    assert_near(pixel(&window, 60, 20), 0xff8080, 2, "one faded rect");
    assert_near(
        pixel(&window, 75, 20),
        0xff4040,
        2,
        "faded rects darken where they overlap",
    );
}

#[test]
fn blur_and_drop_shadow_layers_filter_their_content() {
    let mut window = open(200.0, 80.0);
    let blurred = layer(
        node_key(1),
        rect(0.0, 0.0, 200.0, 80.0),
        LayerEffects {
            filters: vec![LayerFilter::Blur { std_dev: 3.0 }],
            ..LayerEffects::default()
        },
        vec![raster(vec![fill(20.0, 20.0, 40.0, 40.0, 0x000000)])],
    );
    let shadowed = layer(
        node_key(2),
        rect(0.0, 0.0, 200.0, 80.0),
        LayerEffects {
            filters: vec![LayerFilter::DropShadow {
                offset: pt(10.0, 10.0),
                std_dev: 2.0,
                color: color(0x000000),
            }],
            ..LayerEffects::default()
        },
        vec![raster(vec![fill(110.0, 10.0, 40.0, 40.0, RED)])],
    );
    show_composition(&mut window, move |size| Composition {
        layers: vec![
            root_layer(size, vec![child(1), child(2)]),
            blurred,
            shadowed,
        ],
        exports: Vec::new(),
    });
    assert!(
        gray(pixel(&window, 40, 40)) < 8,
        "the blurred square's middle"
    );
    let edge = gray(pixel(&window, 20, 40));
    assert!((96..=160).contains(&edge), "half way at its edge: {edge}");
    let near = gray(pixel(&window, 16, 40));
    assert!(near > edge && near < 250, "the blur reaches beyond: {near}");
    assert_eq!(pixel(&window, 6, 40), WHITE, "and fades within its reach");
    assert_eq!(pixel(&window, 130, 30), RED, "the shadowed square");
    assert!(gray(pixel(&window, 155, 45)) < 40, "its shadow, offset");
    assert_eq!(pixel(&window, 105, 5), WHITE, "no shadow up and left");
}
