# Draw with a canvas

[日本語](../../jp/how-to-guide/canvas.md)

Use a canvas when the standard layout elements cannot express what you want to
show: charts, diagrams, game boards, custom meters, image editors. You draw
shapes, paths, text and images with plain function calls, and Kivact redraws
when the signals you read change. This guide also covers making drawn parts
clickable and readable by screen readers, showing frames from your own
renderer, and exporting pictures without a window.

## Draw shapes

`canvas` takes a closure that receives a `Draw` and the node's current `Size`.
Coordinates are in logical pixels, measured from the canvas's own top-left
corner.

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{Point, Rect};
use kivact::prelude::*;

fn badge() -> impl IntoView {
    canvas(|draw, size| {
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x20242c));
        draw.circle(Point::new(48.0, 48.0), 20, rgb(0x6699ff));
        draw.rounded_rect(Rect::new(96.0, 28.0, 120.0, 40.0), 8.0, ColorToken::Accent);
        draw.line(
            Point::new(12.0, 100.0),
            Point::new(size.width - 12.0, 100.0),
            2,
            rgb(0xffffff),
        );
    })
    .style(style().size(240, 120))
    .name("A blue circle and an accent bar on a dark background")
}

fn main() -> kivact::Result<()> {
    App::new("Canvas").run(badge)
}
```

- A canvas has no size of its own. Give it one with `.style(style().size(w, h))`
  or let layout stretch it, and read the real size from the closure's second
  argument. The first draw happens at size zero, then again after layout, so
  always draw relative to `size`.
- Give the canvas a `.name(..)`. It is the text a screen reader announces for
  the picture.
- Colors are `rgb(0xrrggbb)`, `rgba(0xrrggbb, alpha)`, or theme tokens such as
  `ColorToken::Accent`, which follow the active theme (light, dark, a custom
  theme). Anything that converts to a `Paint` works wherever a paint is
  expected: a color, a token or a gradient.
- Strokes take a width (`2` is a solid line two pixels wide) or a full `Stroke`
  for caps, joins and dashes.

## Draw paths

A `Path` is a chain of moves and segments. `fill_path` fills it,
`stroke_path` outlines it.

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{LineCap, LineJoin, Path, Point, Stroke};
use kivact::prelude::*;

fn wave() -> impl IntoView {
    canvas(|draw, size| {
        let mid = size.height / 2.0;
        let wave = Path::new()
            .move_to(Point::new(8.0, mid))
            .cubic_to(
                Point::new(size.width * 0.3, mid - 40.0),
                Point::new(size.width * 0.6, mid + 40.0),
                Point::new(size.width - 8.0, mid),
            );
        let stroke = Stroke::new(4).cap(LineCap::Round).join(LineJoin::Round);
        draw.stroke_path(&wave, &stroke, ColorToken::Accent);

        let triangle = Path::new()
            .move_to(Point::new(20.0, 20.0))
            .line_to(Point::new(60.0, 20.0))
            .line_to(Point::new(40.0, 50.0))
            .close();
        draw.fill_path(&triangle, rgb(0xe5484d));
        draw.stroke_path(&triangle, &Stroke::new(2).dash([4.0, 3.0]), rgb(0xffffff));
    })
    .style(style().size(240, 120).background(rgb(0x20242c)))
    .name("A wave and a triangle")
}
# fn main() -> kivact::Result<()> { App::new("Paths").run(wave) }
```

`Path` also has `quad_to`, `arc`, `ellipse`, `arc_to` (SVG's `A`), `rect` and
`rounded_rect`. Angles are in degrees, and zero points right. By default a path
is filled with the non-zero rule. For holes, use
`path.with_fill_rule(FillRule::EvenOdd)`. For plain
shapes, `draw.circle` and `draw.fill_shape(rect, &Shape, paint)` are shorter than
building a path.

## Fill with a gradient

There are two kinds, and they place color differently.

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{CanvasGradient, Point, Rect, linear_gradient};
use kivact::prelude::*;

fn gradients() -> impl IntoView {
    canvas(|draw, size| {
        // 1. Relative to the shape being filled, as in CSS.
        let sky = linear_gradient(180).colors([rgb(0x1e3a8a), rgb(0x93c5fd)]);
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, 60.0), sky);

        // 2. Placed by coordinates, as in an HTML canvas. Two shapes that
        //    use the same gradient show the same colors at the same spot.
        let ramp = CanvasGradient::linear(Point::new(0.0, 0.0), Point::new(size.width, 0.0))
            .stop(0.0, rgb(0xe5484d))
            .stop(1.0, rgb(0x3e63dd));
        draw.fill_rect(Rect::new(0.0, 70.0, size.width, 20.0), ramp.clone());
        draw.circle(Point::new(size.width / 2.0, 110.0), 16, ramp);
    })
    .style(style().size(240, 130))
    .name("Gradient samples")
}
# fn main() -> kivact::Result<()> { App::new("Gradients").run(gradients) }
```

`linear_gradient`, `radial_gradient` and `conic_gradient` stretch to the shape
they fill. `CanvasGradient::{linear, radial, conic}` are anchored to points of
the canvas. Both can be used wherever a paint is accepted, including strokes,
and a coordinate gradient can be extended past its ends with
`.extend(GradientExtend::Repeat)`.

## Draw and measure text

`draw.text(text, position, style)` draws one line. The position is the
text's top-left corner unless the `TextStyle` changes its anchor with
`align` and `baseline`. To size things around a label, measure it first with
the same shaping the drawing uses:

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{Draw, Point, Rect, TextBaseline, TextStyle};
use kivact::prelude::*;
use kivact::style::TextAlign;

/// A pill exactly as wide as its label plus 12 px of padding.
fn pill(draw: &mut Draw, label: &str, at: Point) {
    let style = TextStyle {
        size: 12.0,
        weight: 600,
        color: rgb(0xffffff),
        ..TextStyle::default()
    };
    let metrics = draw.measure_text(label, &style);
    let rect = Rect::new(at.x, at.y, metrics.size.width + 12.0, metrics.size.height + 8.0);
    draw.rounded_rect(rect, rect.height / 2.0, ColorToken::Accent);
    draw.text(label, Point::new(at.x + 6.0, at.y + 4.0), style);
}

fn labels() -> impl IntoView {
    canvas(|draw, size| {
        pill(draw, "New", Point::new(12.0, 12.0));
        pill(draw, "In progress", Point::new(12.0, 44.0));

        // Centered on a point: use the anchor instead of measuring.
        let centered = TextStyle {
            align: TextAlign::Center,
            baseline: TextBaseline::Middle,
            ..TextStyle::default()
        };
        draw.text("Centered", Point::new(size.width / 2.0, 100.0), centered);
    })
    .style(style().size(240, 120))
    .name("Label samples")
}
# fn main() -> kivact::Result<()> { App::new("Labels").run(labels) }
```

`TextMetrics` gives the text's `size` (`size.width`, `size.height`) and the
`baseline` offset in logical pixels. For wrapped text use `draw.layout_paragraph(text, &style, Some(max_width))`
and draw the result with `draw.paragraph(&paragraph, origin, color)`.
`TextStyle` has the same vocabulary as element text: `family`, `weight`,
`italic`, `letter_spacing`, `decoration` and so on.

## Transform, clip and fade a group

`draw.layer(Layer, |draw| { … })` applies a transform, clip, opacity, blend mode,
filter, mask or shadow to everything drawn inside the closure. The group ends
when the closure returns. A layer's transform maps text, images and clips too.

```rust,no_run
# use kivact::control::canvas;
# use kivact::draw::{Layer, Point, Rect, Transform};
# use kivact::prelude::*;
fn spinner() -> impl IntoView {
    canvas(|draw, size| {
        let center = Transform::new()
            .translate(size.width / 2.0, size.height / 2.0)
            .rotate(30);
        draw.layer(Layer::new().transform(center).opacity(0.8), |draw| {
            draw.fill_rect(Rect::new(-20.0, -4.0, 40.0, 8.0), ColorToken::Accent);
        });
    })
    .style(style().size(80, 80))
    .name("Spinner")
}
# fn main() -> kivact::Result<()> { App::new("Layer").run(spinner) }
```

## Redraw when state changes

Whatever you read from a signal inside the closure becomes a dependency. When
the signal changes, only this canvas is redrawn. You do not write to a
"dirty" flag or call a repaint function.

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{LineJoin, Path, Point, Stroke};
use kivact::prelude::*;

fn sparkline(values: Signal<Vec<f32>>) -> impl IntoView {
    canvas(move |draw, size| {
        let values = values.get(); // reading subscribes this canvas
        let step = size.width / values.len().saturating_sub(1).max(1) as f32;
        let path = values.iter().enumerate().fold(Path::new(), |path, (i, v)| {
            let point = Point::new(i as f32 * step, size.height * (1.0 - v.clamp(0.0, 1.0)));
            if i == 0 { path.move_to(point) } else { path.line_to(point) }
        });
        draw.stroke_path(&path, &Stroke::new(2).join(LineJoin::Round), ColorToken::Accent);
    })
    .style(style().size(240, 60))
    .name("Trend line")
}

fn app() -> impl IntoView {
    let values = signal(vec![0.5_f32]);
    column((
        sparkline(values),
        button("Add a point").on_press(move || {
            values.update(|v| v.push(((v.len() as f32) * 1.7).sin() * 0.5 + 0.5));
        }),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Sparkline").run(app)
}
```

Keep the closure a pure function of its inputs: read signals, call `draw`
methods, nothing else. Do not write to signals inside it. Load images once
outside the closure (`let logo = Asset::from(bytes)` and then
`draw.image(&logo, rect)`): images are cached by the identity of the `Asset`,
so creating a new `Asset` on every draw decodes the image on every draw.

To animate a canvas, drive a signal from `frame_loop` or `animated` and read it
in the closure. See [Animate values and elements](animation.md).

## Make drawn parts clickable and accessible

A canvas is one node. Pixels you draw have no meaning to the pointer, the
keyboard or a screen reader until you declare them as *regions*. A region has a
shape (the hit area), an accessible role and name, and handlers. You do this in
two steps:

1. In setup, declare behavior with `.region(id, part)` on the canvas. The part
   comes from `kivact::primitive::part` and carries the role, name, state and
   handlers, exactly like attaching a part to an ordinary element.
2. In the draw closure, give the same `RegionId` its geometry with
   `draw.region(id, rect)`.

```rust,test_harness
use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::part;

const SWATCHES: [(&str, u32); 3] = [("Red", 0xe5484d), ("Green", 0x30a46c), ("Blue", 0x3e63dd)];

fn swatches() -> impl IntoView {
    let chosen = signal(0usize);

    let mut board = canvas(move |draw, size| {
        for (i, (_, color)) in SWATCHES.iter().enumerate() {
            let rect = Rect::new(i as f32 * 56.0, 0.0, 48.0, size.height);
            draw.fill_rect(rect, rgb(*color));
            if chosen.get() == i {
                draw.stroke_rect(rect, 3, rgb(0xffffff));
            }
            draw.region(RegionId(i as u64), rect);
        }
    });
    for (i, (name, _)) in SWATCHES.iter().enumerate() {
        board = board.region(
            RegionId(i as u64),
            part(|state| state)
                .role(Role::Button)
                .name(*name)
                .selected(move || chosen.get() == i)
                .on_press(move || chosen.set(i)),
        );
    }

    column((
        board.style(style().size(168, 48)).name("Color swatches"),
        text(move || format!("Chosen: {}", SWATCHES[chosen.get()].0)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Swatches").run(swatches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_region_is_a_button_for_tests_and_screen_readers() {
        let screen = kivact::test::render(swatches);
        screen.get_by_text("Chosen: Red");
        screen.get_by_role(Role::Button, "Green").click();
        screen.get_by_text("Chosen: Green");
        assert!(screen.get_by_role(Role::Button, "Green").is_selected());
    }
}
```

The regions get keyboard focus in the order you declared them, respond to
Enter and Space through `on_press`, and show up in the accessibility tree like
buttons, so `kivact::test` can find and click them by role and name.

Things to know:

- **Declare, then draw.** A `RegionId` that is drawn but not declared, or drawn
  twice in the same layer, is reported as an error and ignored. A declared
  region that is never drawn is simply absent.
- **The hit area follows what you draw.** A region keeps the transform of the
  layer it was drawn in, so rotated and scaled shapes are hit where they appear.
  For non-rectangular areas use `draw.region_shape(id, rect, &Shape::circle(r))`
  or `draw.region_path(id, rect, &path)`.
- **Overlap resolves by drawing order**, and Tab follows declaration order.
- For a changing number of items, `canvas.regions(items, key, make)` declares a
  keyed collection and keeps each item's state while it stays in the list.
- To react to the pointer anywhere on the canvas (a paint brush, a drag), use
  `.on_pointer(|event| …)`. `PointerEvent::pos` is in the canvas's coordinates
  and `PointerEvent::region()` says which region, if any, was hit.
- Editable text fields cannot be put inside a canvas region.

## Show frames from your own renderer

If your application renders pictures itself (an emulator, a 3D viewport, a
video decoder), render on a worker thread, send each frame as `Pixels`
(RGBA8, straight alpha) and draw the newest one in a canvas. The channel
passes frames to the UI thread; no polling is needed.

```rust,no_run
use std::sync::Arc;
use std::time::Duration;
use kivact::control::canvas;
use kivact::draw::{ImageOptions, Pixels, Rect};
use kivact::prelude::*;

fn renderer_preview() -> impl IntoView {
    let image = signal(None::<Pixels>);
    let (tx, rx) = async_channel::bounded(2);

    // Frame size in physical pixels: logical size times the display scale.
    let scale = kivact::platform::scale_factor();
    let width = (320.0 * scale).ceil().max(1.0) as u32;
    let height = (180.0 * scale).ceil().max(1.0) as u32;

    std::thread::spawn(move || {
        for red in (0..=255u8).cycle() {
            // Replace this solid color with your renderer's output.
            let rgba: Arc<[u8]> = [red, 64, 128, 255]
                .repeat(width as usize * height as usize)
                .into();
            if tx.send_blocking(Pixels::new(width, height, rgba)).is_err() {
                break; // the window closed
            }
            std::thread::sleep(Duration::from_millis(16));
        }
    });

    spawn(async move {
        while let Ok(mut newest) = rx.recv().await {
            // Skip frames that piled up while the UI was busy.
            for _ in 0..rx.len() {
                if let Ok(frame) = rx.try_recv() {
                    newest = frame;
                }
            }
            image.set(Some(newest));
        }
    });

    canvas(move |draw, size| {
        if let Some(frame) = image.get() {
            let target = Rect::new(0.0, 0.0, size.width, size.height);
            draw.pixels(&frame, target, ImageOptions::default());
        }
    })
    .style(style().size(320, 180))
    .name("Renderer preview")
}

fn main() -> kivact::Result<()> {
    App::new("Renderer").run(renderer_preview)
}
```

- `async-channel` is a dependency you add to your own `Cargo.toml`. Any
  channel that wakes the receiving task works.
- When the window or the component goes away, the receiving task is dropped,
  the channel closes, and `send_blocking` returns `Err`. End your worker loop
  there.
- Keep the channel small and drop old frames, as above, so a slow UI never
  builds up a backlog.
- If the window can be resized or its scale can change, tell the worker the new
  size and render again. `ImageOptions` can crop (`source`), fade (`opacity`)
  or switch to nearest-neighbour sampling (`pixelated`) for pixel art.
- Reuse the same `Arc` for a frame that did not change instead of copying the
  buffer again.
- When reading frames back from the GPU is too slow, see
  [Share GPU textures with a canvas](gpu-texture-sharing.md).

## Export an image without a window

`Offscreen` renders the same `Draw` vocabulary into an RGBA image of an exact
size. It needs no `App` and opens no window, so use it for exporting artwork,
thumbnails and frame sequences, or from a command-line tool. Write the picture
as one function and use it for both the screen and the export:

```rust
use kivact::draw::{Draw, Offscreen, Point, Rect, Size};
use kivact::prelude::*;

fn artwork(draw: &mut Draw, size: Size) {
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x101820));
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    draw.circle(center, size.height / 3.0, rgb(0xf2aa4c));
}

fn main() -> kivact::Result<()> {
    let mut renderer = Offscreen::new();
    let image = renderer.render(640, 360, artwork)?;

    // Tightly packed RGBA, top row first, straight (not premultiplied) alpha.
    assert_eq!(image.rgba.len(), 640 * 360 * 4);
    assert_eq!(&image.rgba[..4], &[0x10, 0x18, 0x20, 0xff]);
    // Encode `image.rgba` with the PNG or image crate of your choice.
    Ok(())
}
```

In the same app, `canvas(artwork)` shows the exact function you export.

- The image starts transparent. Paint a background to get an opaque picture.
- Coordinates are output pixels at scale 1. A size of zero, or beyond what the
  renderer accepts, is an error and `paint` is not called.
- Drawing commands that cannot be drawn (non-finite coordinates, negative
  radii, `Pixels` whose buffer length does not match) and images or fonts
  that fail to load are returned as errors, where a canvas reports them and
  draws the rest. Check the `Result` before saving.
- Text uses the system's fonts. For identical output on every machine, register
  the files you ship with `renderer.font(include_bytes!("…"))` and give the
  renderer your app's theme with `Offscreen::theme(theme)`.
- An `Offscreen` stays on the thread that made it. To export in the background,
  create it inside `blocking`:

```rust,no_run
use kivact::draw::{Draw, Offscreen, Rect, Size};
use kivact::prelude::*;

fn artwork(draw: &mut Draw, size: Size) {
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x101820));
}

fn exporter() -> impl IntoView {
    let export = action(|(width, height): (u32, u32)| async move {
        let image = blocking(move || Offscreen::new().render(width, height, artwork)).await?;
        // Write `image` to disk here, still off the UI thread.
        Ok(image.rgba.len())
    });
    column((
        button("Export 1920 × 1080").on_press(move || export.run((1920, 1080))),
        dynamic(move || export.state(), |state| match state {
            AsyncState::Idle => text(""),
            AsyncState::Pending => text("Exporting…"),
            AsyncState::Ready(bytes) => text(format!("Rendered {bytes} bytes")),
            AsyncState::Failed(error) => text(format!("Failed: {error}")),
        }),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Export").run(exporter) }
```

## See also

- [Animate values and elements](animation.md): drive a canvas from a frame loop.
- [Run async work from a component](async-and-tasks.md): channels, `spawn` and `blocking`.
- [Share GPU textures with a canvas](gpu-texture-sharing.md): frames from a GPU renderer without a read-back.
- [Test your app](testing.md): find regions by role, compare screenshots.
- API reference: `cargo doc -p kivact --open`, then `kivact::draw` and `kivact::control::canvas`.
