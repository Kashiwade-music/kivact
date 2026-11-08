//! Role: Tests for WGSL paint and filter shaders.
//!
//! The CPU lane (no GPU) checks validation of the user's WGSL and the
//! fallback a CPU draws. The GPU lane runs where the window has a GPU and
//! is skipped, saying why, where it has none.
#![allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]

use std::time::Duration;

use super::paint::{RED, at, canvas_of, expect, pixel};
use kivact::ErrorKind;
use kivact::draw::{Filter, FilterShader, Paint, PaintShader};
use kivact::geometry::Rect;
use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact::style::rgb;
use kivact::test::{MirrorMode, RenderOptions, Screen, gpu_unavailable, render, render_with};

/// The paint entry every test starts from.
const PAINT: &str = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(0.0, 0.0, 1.0, 1.0);\n}\n";

/// The shader errors a screen has reported.
fn shader_errors(screen: &Screen) -> Vec<UiError> {
    screen
        .errors()
        .into_iter()
        .filter(|e| *e.kind() == ErrorKind::Shader)
        .collect()
}

/// Whether the GPU lane can run here; says why when it cannot.
fn gpu_lane(name: &str) -> bool {
    match gpu_unavailable() {
        Some(reason) => {
            eprintln!("SKIPPED {name}: {reason}");
            false
        }
        None => true,
    }
}

fn gpu_options() -> RenderOptions {
    RenderOptions::from_env().gpu().mirror(MirrorMode::Required)
}

#[test]
fn invalid_wgsl_reports_line_and_column() {
    // CPU: a syntax error on line 2 at column 9 of the user's source (the
    // generated code is after it and the prelude before it).
    let source = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    let = 1.0;\n    return vec4<f32>(0.0, 0.0, 1.0, 1.0);\n}\n";
    let error = PaintShader::wgsl(source).expect_err("a syntax error");
    assert_eq!(*error.kind(), ErrorKind::Shader, "{error:?}");
    assert!(
        error.message().starts_with("line 2, column 9: "),
        "{}",
        error.message()
    );
}

#[test]
fn forbidden_bindings_are_rejected() {
    // CPU: a binding of the user, an entry point and a compute entry are
    // each an error of the shader kind.
    let sources = [
        "@group(0) @binding(0) var<uniform> extra: f32;\nfn paint(position: vec2<f32>, context: Context) -> vec4<f32> { return vec4<f32>(extra); }\n",
        "@fragment fn paint(position: vec2<f32>, context: Context) -> vec4<f32> { return vec4<f32>(1.0); }\n",
        "@compute @workgroup_size(1) fn main() {}\n",
    ];
    for source in sources {
        let error = PaintShader::wgsl(source).expect_err("a forbidden form");
        assert_eq!(*error.kind(), ErrorKind::Shader, "{source}: {error:?}");
    }
    let filter_error = FilterShader::wgsl("@group(0) @binding(0) var<uniform> extra: f32;\nfn effect(position: vec2<f32>, context: Context) -> vec4<f32> { return content(position); }\n")
        .expect_err("a binding in a filter");
    assert_eq!(*filter_error.kind(), ErrorKind::Shader, "{filter_error:?}");
}

#[test]
fn wrong_entry_signature_is_rejected() {
    // CPU: an entry that returns something but a vec4, or takes a vec3
    // position, is an error of the shader kind.
    let sources = [
        "fn paint(position: vec2<f32>, context: Context) -> f32 { return 1.0; }\n",
        "fn paint(position: vec3<f32>, context: Context) -> vec4<f32> { return vec4<f32>(1.0); }\n",
    ];
    for source in sources {
        let error = PaintShader::wgsl(source).expect_err("a wrong signature");
        assert_eq!(*error.kind(), ErrorKind::Shader, "{source}: {error:?}");
    }
    assert!(
        PaintShader::wgsl(PAINT).is_ok(),
        "the signature that is right"
    );
}

#[test]
fn cpu_draws_the_fallback() {
    // CPU: the paint shader is valid, but the CPU draws what its fallback
    // is (red), not the blue the shader returns. Tier 2 (±2/255).
    let shader = PaintShader::wgsl(PAINT).expect("a valid paint shader");
    let screen = render(move || {
        canvas_of(
            canvas(move |d, _| {
                d.fill_rect(
                    Rect::new(0.0, 0.0, 100.0, 100.0),
                    shader.paint().fallback(rgb(RED)),
                );
            })
            .style(at(0, 0, 100, 100)),
        )
    });
    expect(&screen.screenshot(), 50, 50, RED, 2, "the fallback");
}

#[test]
fn param_type_mismatch_is_reported() {
    // CPU: `speed` is an f32, and a vec3 is given for it: the parameter is
    // dropped and reported once.
    let source = "struct Params { speed: f32 }\nfn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {\n    return vec4<f32>(params.speed, 0.0, 0.0, 1.0);\n}\n";
    let shader = PaintShader::wgsl(source).expect("a valid paint shader");
    let screen = render(move || {
        node().style(
            style()
                .size(100, 100)
                .background(Paint::from(shader.paint().param("speed", (1.0, 2.0, 3.0)))),
        )
    });
    let errors = shader_errors(&screen);
    assert_eq!(errors.len(), 1, "{errors:?}");
}

/// A dropped parameter of a fallback shader is reported like one of the
/// primary shader.
#[test]
fn fallback_shader_problems_are_reported() {
    let source = "struct Params { speed: f32 }\nfn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {\n    return vec4<f32>(params.speed, 0.0, 0.0, 1.0);\n}\n";
    let fallback = PaintShader::wgsl(source).expect("a valid paint shader");
    let primary = PaintShader::wgsl(PAINT).expect("a valid paint shader");
    let screen = render(move || {
        node().style(
            style().size(100, 100).background(Paint::from(
                primary
                    .paint()
                    .fallback(fallback.paint().param("missing", 1.0)),
            )),
        )
    });
    let errors = shader_errors(&screen);
    assert_eq!(errors.len(), 1, "{errors:?}");
}

#[test]
fn gpu_paint_shader_output() {
    // GPU, tier 2 (the paint of a shader in the compositor, ±2/255): the
    // output is position / size per sampled pixel center.
    if !gpu_lane("gpu_paint_shader_output") {
        return;
    }
    let shader = PaintShader::wgsl("fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(position / context.size, 0.0, 1.0);\n}\n")
        .expect("a valid paint shader");
    let screen = render_with(gpu_options(), move || {
        canvas_of(
            canvas(move |d, _| {
                d.fill_rect(Rect::new(0.0, 0.0, 100.0, 100.0), shader.paint());
            })
            .style(at(0, 0, 100, 100)),
        )
    });
    let shot = screen.screenshot();
    for (x, y) in [(10u32, 10u32), (50, 30), (90, 70)] {
        let channel = |v: u32| ((f64::from(v) + 0.5) / 100.0 * 255.0).round() as u32;
        let want = (channel(x) << 16) | (channel(y) << 8);
        expect(&shot, x, y, want, 2, "the paint shader's output");
    }
}

#[test]
fn gpu_filter_shader_inverts() {
    // GPU, tier 2 (a filter in the compositor, ±2/255): `content()` is the
    // element's own pixels, so a red box comes out cyan.
    if !gpu_lane("gpu_filter_shader_inverts") {
        return;
    }
    let filter = FilterShader::wgsl("fn effect(position: vec2<f32>, context: Context) -> vec4<f32> {\n    let c = content(position);\n    return vec4<f32>(1.0 - c.rgb, c.a);\n}\n")
        .expect("a valid filter shader");
    let screen = render_with(gpu_options(), move || {
        canvas_of(
            node().test_id("filtered").style(
                at(0, 0, 100, 100)
                    .background(rgb(RED))
                    .filter(Filter::new().shader(filter.filter())),
            ),
        )
    });
    expect(
        &screen.screenshot(),
        50,
        50,
        0x00ffff,
        2,
        "the inverted content",
    );
}

#[test]
fn gpu_shader_time_advances() {
    // GPU: `animate(FrameRate::Display)` runs the clock, so the output at
    // the same point changes after the screen advances.
    if !gpu_lane("gpu_shader_time_advances") {
        return;
    }
    let shader = PaintShader::wgsl("fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(fract(context.time), 0.0, 0.0, 1.0);\n}\n")
        .expect("a valid paint shader");
    let screen = render_with(gpu_options(), move || {
        canvas_of(
            canvas(move |d, _| {
                d.fill_rect(
                    Rect::new(0.0, 0.0, 100.0, 100.0),
                    shader.paint().animate(FrameRate::Display),
                );
            })
            .style(at(0, 0, 100, 100)),
        )
    });
    let before = pixel(&screen.screenshot(), 50, 50);
    screen.advance(Duration::from_millis(250));
    let after = pixel(&screen.screenshot(), 50, 50);
    assert_ne!(before, after, "the shader's time did not advance");
    assert!(shader_errors(&screen).is_empty(), "{:?}", screen.errors());
}

#[test]
fn gpu_params_follow_member_attributes() {
    // GPU: `@align(16)` puts `tint` at byte 16 of `Params`, so the host must
    // write it there. Written at the natural offset (4), it would read as 0.
    if !gpu_lane("gpu_params_follow_member_attributes") {
        return;
    }
    let source = "struct Params { speed: f32, @align(16) tint: f32 }\nfn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {\n    return vec4<f32>(params.speed, params.tint, 0.0, 1.0);\n}\n";
    let shader = PaintShader::wgsl(source).expect("a valid paint shader");
    let screen = render_with(gpu_options(), move || {
        canvas_of(
            canvas(move |d, _| {
                d.fill_rect(
                    Rect::new(0.0, 0.0, 100.0, 100.0),
                    shader
                        .paint()
                        .param("speed", 0.25_f64)
                        .param("tint", 0.5_f64),
                );
            })
            .style(at(0, 0, 100, 100)),
        )
    });
    expect(
        &screen.screenshot(),
        50,
        50,
        0x408000,
        2,
        "the members at their attribute offsets",
    );
    assert!(shader_errors(&screen).is_empty(), "{:?}", screen.errors());
}
