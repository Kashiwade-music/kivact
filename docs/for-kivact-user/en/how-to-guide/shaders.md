# Write shaders

[日本語](../../jp/how-to-guide/shaders.md)

Use a shader when a picture is easier to describe as a formula per pixel than
as shapes: a scanline pattern, a noise texture, a glow that follows a
parameter, a filter that distorts what is below it. You write the per-pixel
part in WGSL, the shader language of WebGPU, and Kivact runs it on the GPU.
This guide covers the two kinds of shader (a paint, which fills an area, and
a filter, which changes an element's pixels), the rules your WGSL must follow,
how values reach the shader, and what the CPU renderer shows instead.

## Paint with a shader

A paint shader is a WGSL function `paint` that returns a color for each pixel.
Kivact checks the source when you build the shader, so a mistake is reported
at setup with its line and column, not on the first frame. Pass the source to
`PaintShader::wgsl` as a string.

```rust,no_run
use kivact::draw::{PaintShader};
use kivact::geometry::Rect;
use kivact::motion::FrameRate;
use kivact::prelude::*;

const SCANLINES: &str = r#"
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let uv = position / context.size;
    let wave = 0.5 + 0.5 * sin(uv.x * 12.0 + context.time * params.speed);
    return vec4<f32>(params.tint.rgb * wave, 1.0);
}
"#;

fn scanline_background() -> impl IntoView {
    // Built once, at setup: a shader that does not validate is an error here.
    let scanlines = PaintShader::wgsl(SCANLINES).expect("the scanline shader is valid");

    canvas(move |draw, size| {
        let paint = scanlines
            .paint()
            .param("speed", 2.0_f64)
            .param("tint", rgb(0xffb020))
            .animate(FrameRate::Display)
            .fallback(rgb(0x101820));
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), paint);
    })
    .style(style().size(240, 96))
    .name("Scanline background")
}
# fn main() -> kivact::Result<()> { App::new("Scanlines").run(scanline_background) }
```

- A `ShaderPaint` is a paint. Use it wherever a paint is accepted: a canvas
  fill or stroke, or a style's background with `style().background(Paint::from(..))`.
- `position` is in logical pixels from the top-left of the shape being filled,
  and `context.size` is its size. Divide one by the other to get coordinates
  from 0 to 1.
- The shader returns a straight-alpha sRGB color. Kivact does the blending.
- `fallback` is drawn instead on the CPU, and when the GPU cannot build the
  pipeline. Without one, nothing is drawn in those cases.

## Filter an element with a shader

A filter shader takes the element's own pixels and returns new ones. It is
written with an `effect` function, and it reads the element's pixels with
`content(position)`.

```rust,no_run
use kivact::draw::{Filter, FilterShader};
use kivact::prelude::*;

const SCAN_FILTER: &str = r#"
fn effect(position: vec2<f32>, context: Context) -> vec4<f32> {
    let c = content(position);
    let scan = 0.85 + 0.15 * step(0.5, fract(position.y * 0.5));
    return vec4<f32>(c.rgb * scan, c.a);
}
"#;

fn status_panel() -> impl IntoView {
    let scan = FilterShader::wgsl(SCAN_FILTER).expect("the scan filter is valid");

    column(text("Status: online"))
        .style(
            style()
                .padding(16)
                .radius(8)
                .background(ColorToken::Surface)
                .filter(Filter::new().shader(scan.filter())),
        )
}
# fn main() -> kivact::Result<()> { App::new("Status").run(status_panel) }
```

- `content(position)` is the element's pixels, straight alpha, and transparent
  outside the element. The shader may return pixels outside the element's box,
  such as a glow: give the shader a `margin(px)` to allow the extra room.
- A filter combines with the other filter functions in the same list, in order.
  Its output replaces the element's pixels.

## Pass values to the shader

Declare the values as fields of a WGSL struct named `Params`. Set them from
Rust with `param(name, value)`, by field name. A field you do not set is zero.

| WGSL field type | Rust value |
|---|---|
| `f32` | `f64` (e.g. `2.0_f64`) |
| `vec2<f32>` | `Point` or `(f64, f64)` |
| `vec3<f32>` | `(f64, f64, f64)` |
| `vec4<f32>` | `Color` (e.g. `rgb(0xffb020)`) or `(f64, f64, f64, f64)` |
| `i32` | `i32` |
| `u32` | `u32` |

- A name the struct does not have, or a value of the wrong type, is dropped and
  reported once.
- A `Color` fits a `vec4` field, and no other type.
- Fields are not nested and not arrays.

Images come in through slots 0 to 3. Bind one with `image(slot, source)` and
sample it in WGSL with `image(slot, uv)`. The sample is straight alpha, bilinear
and clamped at the edge.

```rust,no_run
use kivact::draw::{PaintShader};
use kivact::geometry::Rect;
use kivact::prelude::*;

static PHOTO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#888"/></svg>"##;

const TINT_PHOTO: &str = r#"
fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {
    let uv = position / context.size;
    let photo = image(0u, uv);
    return vec4<f32>(photo.rgb * vec3<f32>(1.0, 0.8, 0.6), photo.a);
}
"#;

fn tinted_photo() -> impl IntoView {
    let shader = PaintShader::wgsl(TINT_PHOTO).expect("the photo shader is valid");

    canvas(move |draw, size| {
        let paint = shader.paint().image(0, PHOTO);
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), paint);
    })
    .style(style().size(160, 80))
    .name("Tinted photo")
}
# fn main() -> kivact::Result<()> { App::new("Tinted photo").run(tinted_photo) }
```

- An image is decoded once, like any `Asset`: keep the `Asset` in setup rather
  than building it in the paint closure.
- A slot with no image reads a transparent pixel.

## Animate with the clock

The shader's `context.time` is in seconds. Turn it on with `animate` and a
`FrameRate`: `FrameRate::Display` runs on every display refresh, and
`FrameRate::Hz(30.0)` runs at a fixed rate. Without `animate`, `time` stays at
zero, so a shader without a clock is a still picture.

- Under reduced motion, `time` stays at zero, and the picture does not move.
  Call `essential()` on a shader whose motion carries meaning (for example a
  progress indicator) to keep its clock running.
- The clock pauses while the window is not visible.
- A node with several shaders runs at the fastest of their rates.

## CPU rendering

The CPU renderer does not run WGSL. It draws what the shader says to draw
instead:

- A paint shader draws its `fallback`, or nothing when there is none. Give
  every paint shader a fallback that looks reasonable, so a machine without a
  GPU still shows a sensible picture.
- A filter shader is not run on the CPU, so the element is drawn unfiltered.
  Do not rely on a filter shader for something the user must see.
- A GPU that fails to build a shader's pipeline is treated the same way as the
  CPU, and the failure is reported once.

By default a window draws on the CPU until it first needs the GPU (`GpuStart::OnDemand`): the first frames of a shader show its fallback, and the shader appears once the device is ready. When the first frame must already run the shader, choose `GpuStart::AtLaunch` with `App::gpu_preference` (`kivact::launch`).

Shaders affect only drawing. Hit testing, focus and screen readers see the same
element as before.

## Rules for the WGSL

Your source is the paint or the effect only. Kivact supplies `Context`, the
bindings and the helpers, so this is a complete paint shader:

```wgsl
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let uv = position / context.size;
    let wave = 0.5 + 0.5 * sin(uv.x * 12.0 + context.time * params.speed);
    return vec4<f32>(params.tint.rgb * wave, 1.0);
}
```

Kivact adds its own definitions around your source, so your source must not
define the names it adds or use the forms it reserves. `wgsl(..)` checks the
source and returns an error with the line and column of the problem.

- **Entry points.** A paint shader defines `fn paint(position: vec2<f32>, context: Context) -> vec4<f32>`, or the same with a third argument `params: Params`. A filter shader defines `fn effect(...)` with the same forms. The name `filter` is a WGSL keyword, so the filter's entry is `effect`.
- **`Params`.** Name the struct `Params`. Its fields have the types in the table above.
- **Helpers you can use.** `image(slot: u32, uv: vec2<f32>)`, `content(position: vec2<f32>)` (filter shaders only), `srgb_to_linear(c: vec3<f32>)` and `linear_to_srgb(c: vec3<f32>)`.
- **Not allowed.** `@group` or `@binding` globals, the entry-point attributes `@vertex`, `@fragment` and `@compute`, `override`, `enable`, any name that starts with `kivact_`, and redefinitions of `Context`, `image`, `content`, `srgb_to_linear` or `linear_to_srgb`.
- **Everything else in WGSL works:** functions, loops, arrays inside a function, and `var<private>`.

`Context` has four fields: `size` (the shape's size in logical pixels), `scale`
(device pixels per logical pixel), `time` (seconds, from `animate`) and, through
`position`, where each pixel is.

## Test shaders on the GPU

A shader's picture depends on the GPU, so its tests run on the GPU lane. A test
opts in with `RenderOptions::from_env().gpu()`, and `KIVACT_TEST_GPU=1` turns the
lane on for every test that reads the environment. A test that cannot get a GPU
skips itself and says why. Set `KIVACT_TEST_MIRROR=off` to run the window tests
without a display. The [testing guide](testing.md) describes both.

## See also

- [Draw with a canvas](canvas.md): fills, strokes and the other paints.
- [Use effects](effects.md): the built-in filters, masks and gradients.
- [Animate values and elements](animation.md): values that drive `Params`.
- [Show images and use custom fonts](images-and-fonts.md): loading the pictures you bind to a slot.
- [Test your app](testing.md): the GPU lane and CPU fallbacks in tests.
