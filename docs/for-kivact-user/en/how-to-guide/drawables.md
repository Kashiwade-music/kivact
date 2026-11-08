# Draw live views in a canvas (drawables)

[日本語](../../jp/how-to-guide/drawables.md)

A canvas draws shapes and text. A drawable is the other half: a real view
subtree, with buttons, text fields and signals, that a canvas can draw at any
point under any transform. This is the HTML-in-canvas idea. The drawn view is
still a normal view. It can take focus, get keyboard and IME input, be read by
a screen reader and be found by a test. Use drawables for a settings panel on a
3D card, a control turned on a dial, or a view that another renderer shows.

This guide covers mounting a drawable, drawing it under a transform, what comes
for free (input and accessibility), and exporting a drawable's pixels to your
own renderer, such as a 3D engine.

## Mount a view and draw it

`drawable()` creates a handle in setup. The content is mounted once, with
`.drawable(handle, content)` on the canvas, and the canvas draws it with
`draw.drawable(handle, at)`. The `at` point is where the content's top-left
corner goes, in the canvas's coordinates. Wrap the call in `draw.layer` to put
it under a transform.

```rust,no_run
use kivact::draw::{Layer, Transform};
use kivact::geometry::Point;
use kivact::prelude::*;
use kivact::view::drawable;

fn settings_card() -> impl IntoView {
    let card = drawable();
    let angle = signal(20.0_f64);

    column((
        button("Turn").on_press(move || angle.update(|a| *a += 15.0)),
        canvas(move |draw, size| {
            let width = card.size().get().width;
            let at = Point::new((size.width - width) / 2.0, 40.0);
            draw.layer(
                Layer::new().transform(
                    Transform::new().perspective(800.0).rotate_y(angle.get()),
                ),
                |draw| draw.drawable(card, at),
            );
        })
        .drawable(card, || {
            column((text("Settings"), button("Apply").on_press(|| ())))
        })
        .style(style().size(320, 200)),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Settings card").run(settings_card) }
```

- `card.size()` is the content's laid-out size, as a value. It is zero until
  the first layout, so the canvas draws it after that layout runs.
- The content is laid out as an absolutely positioned child of the canvas, at
  its top-left corner. It does not change the canvas's size; give the canvas a
  size of its own.
- The canvas redraws when the signals it reads change, here `angle` and the
  content's size. The content updates itself: its own signals and handlers work
  as they do anywhere else.
- A drawable is `Copy`, so the canvas closure can capture it like a signal.

## Input and accessibility come with it

You do not write any hit testing or accessibility for the drawn view. The engine
maps each point back through the transform the canvas drew it with, so:

- A click on the rotated "Apply" button lands on the button, as the user sees
  it.
- Keyboard focus moves to the content in document order, and Tab and Enter work
  as they do in any view.
- The content appears in the accessibility tree of the canvas, with its bounds
  transformed to the place where the content is drawn. Screen readers announce
  it as they announce any other view.
- A test finds the content by role and name, for example `Role::Button` named
  `Apply`, and clicks it, as in [Test your app](testing.md).

A drawable that the canvas does not draw in the current frame takes no pointer
input.

## Export a drawable to your own renderer

A 3D engine or a custom renderer often needs the content as pictures, not on
screen. Mount the content on a `Viewport` instead: it is laid out and rendered
for export, and never drawn on screen. `share()` returns a `DrawableShare`,
a `Send` and `Sync` handle for your render thread. Ask for pixels, then wait for
frames.

```rust,no_run
use std::time::Duration;
use kivact::draw::Pixels;
use kivact::geometry::Matrix4;
use kivact::prelude::*;
use kivact::view::{DrawableFrame, ShareError, ShareUpdate, drawable, viewport};

fn engine_panel() -> impl IntoView {
    let panel = drawable();
    let share = panel.share();
    share.request_pixels();
    // The engine shows the panel 40 px right and 24 px down in the viewport.
    share.set_geometry(Some(Matrix4::translate(40.0, 24.0, 0.0)));

    std::thread::spawn(move || {
        let mut seen = 0;
        loop {
            match share.wait(seen, Duration::from_millis(100)) {
                Ok(ShareUpdate::Frame(frame)) => {
                    seen = frame.number();
                    if let DrawableFrame::Pixels { pixels, .. } = frame {
                        // Upload `pixels` (straight-alpha RGBA8) to a texture of your engine.
                        let _ = pixels;
                    }
                }
                Ok(_) => {}
                Err(ShareError::Timeout) => {}
                Err(_) => break, // the drawable is gone, or the request failed
            }
        }
    });

    viewport(Pixels::new(1, 1, vec![0, 0, 0, 0]))
        .drawable(panel, || column((text("Settings"), button("Apply").on_press(|| ()))))
        .style(style().size(320, 200))
}
# fn main() -> kivact::Result<()> { App::new("Engine panel").run(engine_panel) }
```

- `wait(after, timeout)` returns the next frame whose number is greater than
  `after`, or `Timeout` when none comes in time. Pass the last number you saw;
  `0` takes any frame. Frame numbers keep growing across requests, so passing
  the last number you saw also works after a new request.
- `latest()` returns the newest frame without waiting. Use it when a render
  loop prefers the last picture it has.
- `request_pixels()` starts a new request. Frames of an older request are
  dropped. `stop()` ends the export and frees the window's buffers.
- Frames are straight-alpha RGBA8, in device pixels: the window's scale unless
  you call `set_scale(..)` on the share.
- `is_closed()` tells you whether the drawable was disposed. The loop above ends
  on `Closed` as well.
- To share frames as GPU textures with your device, call `request_gpu(adapter)`
  with the window's adapter, which needs the `gpu-interop` feature. See
  [Show textures from your own GPU renderer](gpu-texture-sharing.md) for how the import
  hands a texture back to the window.

- `set_geometry(Some(matrix))` tells the view where your engine draws the panel.
  The matrix maps the drawable's box, in its logical pixels from its top-left
  corner, to the logical pixels of the viewport that hosts it. The engine uses
  the x, y and w parts of the matrix and ignores z. A 3D transform therefore has
  to be applied to the panel's plane first, and you pass the 2D map that
  results: the entries of `m` in rows 0, 1 and 3 and columns 0, 1 and 3. With
  it, a click on the panel in your 3D view reaches the panel's buttons.
- `set_geometry(None)` hides the panel from hit testing. Frames keep coming,
  but clicks no longer reach its content.
- The geometry belongs to the current request. `request_pixels()` and
  `request_gpu(..)` start a new one with no geometry, so call `set_geometry`
  again after them.

## Things to know

- **Mount once.** A drawable can be mounted in one place. A second mount is
  reported and leaves that place empty.
- **Draw once per recording.** Drawing the same drawable twice in one canvas
  draws the first and reports the second as an error.
- **Content is a normal view.** It has its own state, handlers and focus, like
  any other view.
- **Drawn pixels and hits follow the drawing.** A drawable turned, scaled or
  lifted in 3D is hit where it appears; see [Use 3D transforms and depth](3d-and-depth.md).
- **Export costs a render.** Each request renders the content; stop exports you
  no longer read.

## See also

- [Draw with a canvas](canvas.md): shapes, paths, text and the hit areas of drawn regions.
- [Use 3D transforms and depth](3d-and-depth.md): the transforms a drawable is drawn under.
- [Show textures from your own GPU renderer](gpu-texture-sharing.md): GPU frames without a read-back.
- [Test your app](testing.md): find and click the content by role and name.
- API reference: `cargo doc -p kivact --open`, then `kivact::view::drawable` and `kivact::view::DrawableShare`.
