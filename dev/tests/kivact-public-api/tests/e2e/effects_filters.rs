//! Role: Tests for filters, filter graphs, masks and backdrop effects.
//!
//! Filters act on the composite of their element (CSS, in sRGB), masks
//! on its alpha. Pixels of a filter or mask op are compared within ±2/255
//! per channel (§3.3, tier 2); a test that compares against a gradient
//! also allows the gradient's own rounding and says so.

use kivact::draw::filter::Channel;
use kivact::draw::linear_gradient;
use kivact::draw::{Filter, FilterGraph, FilterInput, MaskComposite, MaskLayer, MaskMode};
use kivact::prelude::*;
use kivact::test::{Screenshot, render};

use crate::paint::{BLACK, BLUE, RED, WHITE, at, canvas_of, distance, expect, pixel};

/// The stripes-and-gradient box the graph tests displace.
fn gradient_box(filter: Option<Filter>) -> impl IntoView {
    let base = at(40, 40, 200, 160).background(linear_gradient(90).colors([rgb(RED), rgb(BLUE)]));
    let base = match filter {
        Some(filter) => base.filter(filter),
        None => base,
    };
    canvas_of(node().style(base))
}

/// A displacement by noise: the glitch of the graph tests.
fn glitch() -> Filter {
    let mut graph = FilterGraph::new();
    let noise = graph.turbulence(0.05, 2, 3.0);
    graph.displacement_map(FilterInput::SOURCE, noise, 12.0, Channel::R, Channel::G);
    Filter::new().graph(graph)
}

#[test]
fn contrast_applies_to_the_composite() {
    // Two translucent children (red and blue, alpha 0.5) over white in an
    // opaque parent. Where they overlap, the composite is (128, 64, 191)
    // (blue over the red over white). CSS contrast(2) maps each channel
    // to 2c − 0.5: (0.5, 0, 1), the colour (128, 0, 255). Tier 2.
    let screen = render(|| {
        canvas_of(
            node()
                .style(
                    at(40, 40, 120, 80)
                        .background(rgb(WHITE))
                        .filter(Filter::new().contrast(2)),
                )
                .child((
                    node().style(at(0, 0, 60, 80).background(rgb(RED)).opacity(0.5)),
                    node().style(at(30, 0, 60, 80).background(rgb(BLUE)).opacity(0.5)),
                )),
        )
    });
    let shot = screen.screenshot();
    expect(
        &shot,
        85,
        60,
        0x8000ff,
        2,
        "the overlap, contrast of the composite",
    );
    // Red over white alone: (255, 128, 128) and contrast keeps it.
    expect(
        &shot,
        50,
        60,
        0xff8080,
        2,
        "the red alone, contrast of the composite",
    );
}

#[test]
fn brightness_clamps_after_compositing() {
    // An opaque grey 0.8 with a grey 0.2 at alpha 0.5 on it composites to
    // 0.5; brightness(2) on the parent gives 1.0. Multiplying each child
    // first would give 0.7 instead.
    let screen = render(|| {
        canvas_of(
            node()
                .style(at(40, 40, 120, 80).filter(Filter::new().brightness(2)))
                .child((
                    node().style(at(0, 0, 60, 80).background(rgb(0xcccccc))),
                    node().style(at(0, 0, 60, 80).background(rgb(0x333333)).opacity(0.5)),
                )),
        )
    });
    let shot = screen.screenshot();
    expect(&shot, 50, 60, WHITE, 2, "the overlap is clamped at 1.0");
}

#[test]
fn filter_graph_displaces_the_source() {
    // Tier 2 for the same input: the same screenshot each time; the
    // displaced picture differs from the source.
    let plain = render(|| gradient_box(None)).screenshot();
    let first = render(|| gradient_box(Some(glitch()))).screenshot();
    let second = render(|| gradient_box(Some(glitch()))).screenshot();
    assert_ne!(
        first.hash(),
        plain.hash(),
        "the displacement changes the picture"
    );
    assert_eq!(
        first.hash(),
        second.hash(),
        "the same input gives the same output"
    );
}

#[test]
fn filter_graph_rejects_handles_of_other_graphs() {
    // A handle of graph A used in graph B is replaced by the source and
    // reported once. SOURCE is valid in every graph.
    let screen = render(|| {
        let mut a = FilterGraph::new();
        let from_a = a.flood(rgb(RED));
        let mut b = FilterGraph::new();
        b.blur(from_a, 2.0);
        let mut c = FilterGraph::new();
        c.blur(FilterInput::SOURCE, 2.0);
        canvas_of((
            node().style(at(0, 0, 40, 40).filter(Filter::new().graph(b))),
            node().style(at(60, 0, 40, 40).filter(Filter::new().graph(c))),
        ))
    });
    assert_eq!(
        screen.errors().len(),
        1,
        "the foreign handle is reported once: {:?}",
        screen.errors()
    );
}

#[test]
fn backdrop_drop_shadow_is_applied() {
    // A Backdrop Root (CONTRACT-BACKDROP-ROOT: a layer with an effect; here
    // opacity 0.999, which is invisible in the output) with a transparent
    // background. Its child, the glass, reads that root's content: the red
    // box, and nothing where the box is not. The drop shadow (offset 10 px
    // right, blue) therefore shows right of the box. Tier 2.
    let screen = render(|| {
        node().style(style().size(800, 600)).child(
            node().style(style().size(800, 600).opacity(0.999)).child((
                node().style(at(40, 40, 60, 60).background(rgb(RED))),
                node().style(
                    at(40, 40, 120, 120).backdrop_filter(Filter::new().drop_shadow(
                        10,
                        0,
                        0,
                        rgb(BLUE),
                    )),
                ),
            )),
        )
    });
    let shot = screen.screenshot();
    expect(&shot, 105, 70, BLUE, 2, "the backdrop's drop shadow");
    expect(&shot, 60, 70, RED, 2, "the red box beneath");
}

#[test]
fn luminance_mask_uses_brightness() {
    // A white-to-black gradient as a luminance mask: the white side keeps
    // the red opaque, the black side makes it transparent. The gradient's
    // own rounding allows a few levels.
    let screen = render(|| {
        canvas_of(
            node().style(
                at(40, 40, 200, 100).background(rgb(RED)).mask(
                    MaskLayer::new(linear_gradient(90).colors([rgb(WHITE), rgb(BLACK)]))
                        .mode(MaskMode::Luminance),
                ),
            ),
        )
    });
    let shot = screen.screenshot();
    let opaque = pixel(&shot, 45, 90);
    assert!(
        distance(opaque, RED) <= 16,
        "the white side is opaque: {opaque:06x}"
    );
    let clear = pixel(&shot, 235, 90);
    assert!(
        distance(clear, WHITE) <= 8,
        "the black side is transparent: {clear:06x}"
    );
}

#[test]
fn mask_composite_subtract() {
    // The top layer is opaque white with subtract (CSS source-out over the
    // layer beneath, the white-to-transparent gradient whose alpha is 1 - t):
    // the result alpha is t. At the middle (t = 0.5) the red is at 50%:
    // (255, 128, 128). Tier 2, with the gradient rounding.
    let screen = render(|| {
        canvas_of(node().style(at(40, 40, 200, 40).background(rgb(RED)).mask((
            MaskLayer::new(rgb(WHITE)).composite(MaskComposite::Subtract),
            MaskLayer::new(linear_gradient(90).colors([rgb(WHITE), Color::TRANSPARENT])),
        ))))
    });
    let shot: Screenshot = screen.screenshot();
    expect(&shot, 140, 60, 0xff8080, 4, "the middle is half red");
    expect(&shot, 236, 60, RED, 8, "the right end is opaque red");
    expect(&shot, 44, 60, WHITE, 8, "the left end is clear");
}
