# Use effects

[日本語](../../jp/how-to-guide/effects.md)

Use this guide to change how an element is composed with what is around it:
blur and recolor it with filters, fade its edges with masks, mix it with the
pixels behind it, frame it with a picture cut into nine pieces, and fill the
letters of a text with a gradient. These are CSS properties with the same
names and the same results in Kivact. They are set on a style, and on a
canvas they are the arguments of `Layer` and the painter's `border_image`.

## Filter an element

`filter` changes everything the element and its children draw, after they are
drawn. Its functions are CSS's: `brightness`, `contrast`, `saturate`,
`grayscale`, `sepia`, `hue_rotate`, `invert`, `opacity`, `blur` and
`drop_shadow`. Amounts are CSS's too: `1` leaves the picture as it is for
brightness, contrast, saturate and opacity, and `0` for grayscale, sepia and
invert. Angles are in degrees and lengths in logical pixels.

```rust,no_run
use kivact::draw::Filter;
use kivact::prelude::*;
use kivact::style::rgba;

fn lit_badge() -> impl IntoView {
    column(text("Online"))
        .style(
            style()
                .padding(12)
                .radius(8)
                .background(ColorToken::Accent)
                .filter(
                    Filter::new()
                        .brightness(1.12)
                        .drop_shadow(0.0, 0.0, 4.0, rgba(0x5aa9ff, 0.5)),
                ),
        )
}
# fn main() -> kivact::Result<()> { App::new("Badge").run(lit_badge) }
```

- A `drop_shadow` in a filter follows the drawn shape, including text and
  transparent parts of an image. A box `shadow` follows the box.
- `backdrop_filter` takes the same `Filter`, applied to what is behind the
  element. See [Read what is behind an element](#read-what-is-behind-an-element).

### Filter with a graph

For the SVG filter primitives, build a `FilterGraph`. Each method adds one
primitive and returns a `FilterInput`, a handle to its result. Pass the handle
of an earlier primitive as an input of a later one. `FilterInput::SOURCE` is the
element's own pixels. `Filter::graph` consumes the graph.

```rust,no_run
use kivact::draw::filter::Channel;
use kivact::draw::{Filter, FilterGraph, FilterInput};
use kivact::prelude::*;

fn glitch_title() -> impl IntoView {
    // Displace the element's pixels by noise: a glitch.
    let mut graph = FilterGraph::new();
    let noise = graph.turbulence(0.02, 2, 3.0);
    graph.displacement_map(FilterInput::SOURCE, noise, 12.0, Channel::R, Channel::G);

    text("GLITCH")
        .style(style().font_size(32).weight(700).filter(Filter::new().graph(graph)))
}
# fn main() -> kivact::Result<()> { App::new("Glitch").run(glitch_title) }
```

- Graph primitives include `blur`, `offset`, `blend`, `composite`,
  `color_matrix`, `saturate`, `hue_rotate`, `component_transfer`, `merge`,
  `erode`, `dilate`, `convolve`, `drop_shadow`, `flood`, `image`, `tile`,
  `turbulence`, `fractal_noise`, `displacement_map` and the lighting pair
  `diffuse_lighting` and `specular_lighting`.
- Colors are theme-dependent: `flood` and `drop_shadow` take a `ThemeColor`, so
  a graph follows the active theme.
- The filter region reaches 10% of the element's size beyond its box by
  default. `graph.margin(..)` sets another extent, which matters for glows.
- A graph computes in linear RGB, as SVG does by default. Call `graph.srgb()`
  to compute in sRGB instead.
- A handle from another graph is an error: it is reported when the style is
  compiled, and the input is replaced by the element.

## Fade with a mask

A `mask` keeps the alpha of its layers: where the mask is transparent, the
element and its children disappear. A gradient is the usual mask.

```rust,no_run
use kivact::draw::{MaskComposite, MaskLayer};
use kivact::prelude::*;
use kivact::draw::linear_gradient;

fn fading_list() -> impl IntoView {
    let fade = linear_gradient(180).colors([Color::BLACK, Color::TRANSPARENT]);
    column((text("First"), text("Second"), text("Third")))
        .style(style().gap(8).height(160).mask(fade))
}
# fn main() -> kivact::Result<()> { App::new("Fade").run(fading_list) }
```

- Use `kivact::draw::MaskLayer` for more control: `MaskLayer::new(paint)`,
  `.mode(MaskMode::Luminance)` to mask by brightness instead of alpha, and
  `.composite(MaskComposite::Intersect)` to combine it with the layers below.
  Several layers are a tuple: `style().mask((first, second))`, with the first
  layer on top.
- A mask's paint is positioned in the element's border box by default.
  `Paint::origin(..)` and `Paint::clip(..)` change that, as for backgrounds.
- A mask whose source is a shader is not drawn; it is reported as unsupported.

## Blend with what is behind

`mix_blend_mode` combines an element with the pixels behind it, as CSS's
`mix-blend-mode` does. `Multiply` darkens and `Screen` lightens. `Isolation`
decides which pixels an element blends with.

```rust,no_run
use kivact::draw::BlendMode;
use kivact::prelude::*;
use kivact::style::Isolation;

fn duotone_card() -> impl IntoView {
    stack((
        column(text("Image"))
            .style(style().size(200, 120).padding(12).background(ColorToken::Accent)),
        column(text("Tint"))
            .style(
                style()
                    .size(200, 120)
                    .padding(12)
                    .background(rgb(0xffd166))
                    .mix_blend_mode(BlendMode::Multiply),
            ),
    ))
    // Blend only inside this card, not with the rest of the window.
    .style(style().isolation(Isolation::Isolate))
}
# fn main() -> kivact::Result<()> { App::new("Duotone").run(duotone_card) }
```

- An element blends with the pixels of its isolated group. An element with
  `isolation(Isolation::Isolate)` is one. An element with an `opacity` below
  1, a `filter`, a `mask` or a `clip_path` also forms a group of its own, as in
  CSS.
- Set `Paint::blend(..)` on a background layer to blend that layer with the
  background colors below it (CSS `background-blend-mode`).

## Read what is behind an element

`backdrop_filter` blurs or recolors the pixels behind an element, so a
translucent panel looks like glass. `backdrop_blur(px)` is the short form of
`backdrop_filter(Filter::new().blur(px))`.

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgba;

fn glass_panel() -> impl IntoView {
    stack((
        column(text("Behind the glass"))
            .style(style().size(260, 160).padding(16).background(rgb(0x3e63dd))),
        column(text("Glass"))
            .style(
                style()
                    .size(180, 80)
                    .padding(12)
                    .radius(12)
                    .background(rgba(0xffffff, 0.2))
                    .backdrop_blur(12.0),
            ),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Glass").run(glass_panel) }
```

- The backdrop is what is painted behind the element, up to the nearest
  ancestor that is a group of its own (an `opacity` below 1, a filter, a mask or
  a backdrop filter). Without such an ancestor, it is the whole window.
- Paint the element's own background translucent, as above. An opaque
  background hides the blurred backdrop.

## Frame a box with a picture

`border_image` cuts a picture (or a gradient) into nine pieces: four corners,
four edges and a middle. The corners keep their size, and the edges stretch or
repeat along the border. Use it for frames and rounded decoration that should
scale without distortion.

```rust,no_run
use kivact::draw::{BorderImage, BorderImageRepeat};
use kivact::prelude::*;

fn framed_panel() -> impl IntoView {
    // A 36 px picture with a 12 px frame. The border is 12 px wide.
    let frame = BorderImage::new(br##"<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 36 36"><rect x="2" y="2" width="32" height="32" rx="6" fill="none" stroke="#3e63dd" stroke-width="4"/></svg>"##)
        .slice(12)
        .repeat(BorderImageRepeat::Round);

    column(text("Framed"))
        .style(style().padding(16).border(12, ColorToken::Border).border_image(frame))
}
# fn main() -> kivact::Result<()> { App::new("Frame").run(framed_panel) }
```

- `slice` gives the width of the four cut lines in the picture, in pixels
  (`slice_pct` takes percentages). The corners are that size in the box.
- `width(..)` sets the width of the frame, and defaults to the border widths.
  `outset(..)` moves the frame beyond the border box.
- `repeat`, with `Stretch` (the default), `Repeat`, `Round` or `Space`, sets how
  the edge pieces fill a side that is longer than one piece.
- `fill()` also paints the middle of the picture behind the content.
- A gradient works as the source too: `BorderImage::new(linear_gradient(90).colors([...]))`.
  Its pieces are drawn with the box's size.

On a canvas, `Draw::border_image(rect, &frame)` draws the same nine pieces in
a rectangle. A canvas has no border, so set the frame's `width` yourself:

```rust,no_run
use kivact::draw::{BorderImage, BorderImageRepeat};
use kivact::geometry::Rect;
use kivact::prelude::*;

fn framed_canvas() -> impl IntoView {
    let frame = BorderImage::new(br##"<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 36 36"><rect x="2" y="2" width="32" height="32" rx="6" fill="none" stroke="#3e63dd" stroke-width="4"/></svg>"##)
        .slice(12)
        .width(12)
        .repeat(BorderImageRepeat::Stretch);

    canvas(move |draw, size| {
        draw.border_image(Rect::new(0.0, 0.0, size.width, size.height), &frame);
    })
    .style(style().size(240, 120))
    .name("Framed picture")
}
# fn main() -> kivact::Result<()> { App::new("Framed canvas").run(framed_canvas) }
```

## Choose the area a background is painted in

A background has two settings. `Paint::origin` chooses the box the picture or
gradient is positioned and sized from. `Paint::clip` chooses the box it is
painted in. Both default to the padding box, and the clip defaults to the border
box, as in CSS.

```rust,no_run
use kivact::draw::{BackgroundClip, BackgroundOrigin, Paint};
use kivact::prelude::*;
use kivact::style::rgba;
use kivact::draw::linear_gradient;

fn padded_tint() -> impl IntoView {
    let tint = linear_gradient(90).colors([rgb(0x3e63dd), rgb(0x30a46c)]);

    column(text("Tinted content box"))
        .style(
            style()
                .padding(24)
                .border(8, rgba(0xffffff, 0.5))
                .background(
                    Paint::from(tint)
                        .origin(BackgroundOrigin::ContentBox)
                        .clip(BackgroundClip::ContentBox),
                ),
        )
}
# fn main() -> kivact::Result<()> { App::new("Tint").run(padded_tint) }
```

- `BackgroundClip::BorderBox`, `PaddingBox` and `ContentBox` clip to the
  matching box, with rounded corners that follow the border radius.
- `BackgroundClip::Text` paints only through the glyphs of the element's own
  text. See the next section.

## Fill text with a gradient

Clip a background to `BackgroundClip::Text` and make the text itself
transparent, so the gradient shows through the letters. This works on the
element that holds the text. A container with `BackgroundClip::Text` has no
text of its own, so nothing is painted.

```rust,no_run
use kivact::draw::{BackgroundClip, Paint};
use kivact::prelude::*;
use kivact::draw::linear_gradient;

fn gradient_title() -> impl IntoView {
    let fill = linear_gradient(90).colors([rgb(0xe5484d), rgb(0xffb020), rgb(0x30a46c)]);

    text("Gradient text").style(
        style()
            .font_size(40)
            .weight(800)
            .color(Color::TRANSPARENT)
            .background(Paint::from(fill).clip(BackgroundClip::Text)),
    )
}
# fn main() -> kivact::Result<()> { App::new("Gradient text").run(gradient_title) }
```

- `hint(..)` on a gradient sets where, between two stops, the color is halfway,
  as in CSS: `linear_gradient(90).colors([...]).hint(30)`.
- `interpolate(..)` on a gradient blends the stops in another color space. The
  result is shown in sRGB.

## Things to know

- **Scope.** A filter, a mask and a blend apply to the element and everything
  inside it, as one picture.
- **Costs.** Filters, masks, backdrops and blends each need the pixels of a
  group, so they are drawn as layers. A few of them on many elements add up;
  prefer a single effect on a container over the same effect on every child.
- **Where effects stop.** A shader filter and a shader mask follow their own
  rules: see [Write shaders](shaders.md).
- **Animation.** Filters, opacity and solid backgrounds can be animated with a
  style transition. See [Animate values and elements](animation.md).

## See also

- [Use 3D transforms and depth](3d-and-depth.md): the transforms these effects sit on.
- [Write shaders](shaders.md): WGSL filters and paints.
- [Draw with a canvas](canvas.md): the same effects on a `Layer`.
- [Change the theme and the look of components](theming.md): colors and gradients from tokens.
- API reference: `cargo doc -p kivact --open`, then `kivact::draw::Filter`, `FilterGraph`, `Mask`, `BorderImage` and `BackgroundClip`.
