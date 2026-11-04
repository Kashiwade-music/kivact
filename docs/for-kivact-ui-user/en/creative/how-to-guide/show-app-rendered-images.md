# Show frames your application renders

[日本語](../../../jp/creative/how-to-guide/show-app-rendered-images.md)

Goal: show an image your application produces (a 3D viewport, a composite, a histogram, a node preview) in a pannable, zoomable surface, receive pointer positions in image coordinates, and keep heavy rendering off the UI thread.

The kit draws and navigates; it never renders your scene. You give it RGBA pixels, and it handles fitting, zoom, pan, overlays and input. This guide assumes you finished the [tutorial](../getting-started/README.md). Building images and passing them between threads is covered more generally in the core guide [Draw with a canvas](../../../../for-kivact-user/en/how-to-guide/canvas.md#show-frames-from-your-own-renderer).

## Show an image with zoom and pan

```rust,no_run
use kivact::draw::{Pixels, Point, Rect, Size};
use kivact::interaction::ViewTransform;
use kivact::prelude::*;
use kivact::style::{ThemeMode, rgb};
use kivact_ui_creative as ui;
use std::sync::Arc;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

/// Your renderer. Here: a procedural gradient whose blue channel is `tint`.
fn render(tint: f64) -> Pixels {
    let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            rgba.push((x * 255 / WIDTH) as u8);
            rgba.push((y * 255 / HEIGHT) as u8);
            rgba.push((tint.clamp(0.0, 1.0) * 255.0) as u8);
            rgba.push(255);
        }
    }
    Pixels::new(WIDTH, HEIGHT, Arc::from(rgba))
}

fn viewer() -> impl IntoView {
    let tint = signal(0.5);
    let transform = signal(ViewTransform::default());
    let surface = signal(Size::new(0.0, 0.0));
    let pointer = signal(Point::new(0.0, 0.0));

    // Re-renders whenever `tint` changes; `None` would show only the background.
    let frame = memo(move || Some(render(tint.get())));

    column((
        row((
            ui::value_slider("Tint", tint).range(0.0, 1.0).step(0.01).precision(2),
            button("Fit").on_press(move || {
                transform.set(ViewTransform::fit(
                    Size::new(WIDTH as f64, HEIGHT as f64),
                    surface.get_untracked(),
                ));
            }),
            text(move || {
                let p = pointer.get();
                format!("x {:.0}, y {:.0}", p.x, p.y)
            }),
        ))
        .style(style().gap(8)),
        ui::viewport_surface(frame)
            .transform(transform)
            .on_resize(move |size: ui::SurfaceSize| surface.set(size.logical))
            .on_pointer(move |event: ui::SurfacePointer| pointer.set(event.position))
            .overlay(|draw, _transform| {
                draw.stroke_rect(Rect::new(200.0, 100.0, 240.0, 160.0), 1.0, rgb(0xffd25a));
            })
            .name("Viewer")
            .style(style().height(400)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Viewer")
        .theme(ui::theme(ThemeMode::Dark))
        .run(viewer)
}
```

Run it. The image fits the surface at first. Scroll to zoom around the pointer, pan with the middle button (or hold `Space` and drag with the primary button), and press **Fit** to return. The yellow rectangle stays glued to the picture as you zoom, because the overlay is drawn in image coordinates.

### What the surface does for you

- **Without `.transform(..)` the image always fits** the surface. With a `Binding<ViewTransform>` you own the zoom and pan, so you can save it, reset it (as **Fit** does) or share it between two viewers.
- **Coordinates are the image's.** `on_pointer` reports each `SurfacePointer` (position, phase, modifiers; a press's button is in the phase) with `position` in pixels of the *image*, whatever the zoom is. `overlay(|draw, transform| ..)` draws in the same coordinates. Use `transform` when you need the current scale, for example to keep a handle the same size on screen.
- **`regions(..)`** adds accessible, clickable rectangles in image coordinates (`SurfaceRegion::new(id, name, role, bounds)`); the `region` field of a pointer event says which one is under the pointer.
- **`pixelated(true)`** samples the nearest pixel when you zoom in past the native size, which is what you want for pixel inspection.
- **`on_resize`** reports a `SurfaceSize`: the logical size, the physical pixel size and the scale factor. Render at `physical_width` by `physical_height` for a sharp image on a high-density display, and re-render when it changes.
- **Equality decides repainting.** `Pixels` values are compared by identity, so cloning or passing the same `Pixels` again costs nothing, and a new `Pixels` triggers a repaint.
- **3D, compositing and scopes are yours.** The surface does not know what the pixels mean. A histogram or waveform is just another image you render from your data, shown in another `viewport_surface`.

Touch screens zoom and pan with two fingers. There is no separate trackpad pinch event yet; a pinch that arrives as a wheel event zooms.

## Render on another thread

Rendering that takes longer than a frame must not block the UI. Render on a worker thread and send finished `Pixels` over a channel; a UI task writes the newest one into a signal. Signals belong to the UI thread, so never move a `Signal` into the worker. Send plain data.

```rust,no_run
use kivact::draw::Pixels;
use kivact::prelude::*;
use kivact_ui_creative as ui;
use std::sync::Arc;
use std::time::Duration;

fn render(frame: u32) -> Pixels {
    let shade = (frame % 256) as u8;
    Pixels::new(2, 2, Arc::from([shade, 80, 160, 255].repeat(4)))
}

/// Starts a renderer and returns the signal that always holds its latest frame.
fn start_renderer() -> Signal<Option<Pixels>> {
    let image = signal(None);
    let (sender, receiver) = async_channel::bounded::<Pixels>(2);

    std::thread::spawn(move || {
        for frame in 0.. {
            // Fails once the view is gone and the receiver is dropped: stop.
            if sender.send_blocking(render(frame)).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(33));
        }
    });

    spawn(async move {
        while let Ok(mut newest) = receiver.recv().await {
            // Skip frames the UI was too slow to show.
            while let Ok(later) = receiver.try_recv() {
                newest = later;
            }
            image.set(Some(newest));
        }
    });
    image
}

fn live_view() -> impl IntoView {
    let image = start_renderer();
    ui::viewport_surface(image)
        .pixelated(true)
        .name("Live view")
        .style(style().height(300))
}
```

Add `async-channel` to your `Cargo.toml` for this example. The task started with `spawn` belongs to the component that called `start_renderer` and stops when the component goes away; the channel closing is what tells the worker to exit. The bounded channel (size 2) keeps a slow UI from building a queue of stale frames.

## Show a GPU frame without reading it back (experimental)

If your renderer already draws on the GPU, reading each frame back to the CPU as `Pixels` can cost more than the frame budget. An experimental path shares the GPU image with the window instead. It is limited to Windows and Linux, needs the `gpu-interop` feature of both `kivact` and `kivact-ui-creative`, and is described in full in the core guide [Share GPU textures with a canvas](../../../../for-kivact-user/en/how-to-guide/gpu-texture-sharing.md).

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["gpu-interop"] }
kivact-ui-creative = { git = "https://github.com/Kashiwade-music/kivact", features = ["gpu-interop"] }
```

The surface then has `gpu_frame(..)`, which takes an optional frame:

```text
let image = kivact::gpu_interop::shared_image(1280, 720);
let (frame, pixels, sized) = (image.clone(), image.clone(), image);
ui::viewport_surface(move || pixels.pixels())
    .gpu_frame(move || frame.frame())
    .on_resize(move |size| sized.set_size(size.physical_width, size.physical_height))
```

While `gpu_frame` supplies a frame and the window has a GPU device, the surface shows it. Otherwise (the device is not ready, the window renders on the CPU, or the frame was lost) it shows the `Pixels` you passed to `viewport_surface`, so keep both. The size of whichever image is shown drives fit, zoom, pointer coordinates, overlays and regions. This API may change without notice.

## See also

- [Draw with a canvas](../../../../for-kivact-user/en/how-to-guide/canvas.md#show-frames-from-your-own-renderer): `Pixels`, canvases and passing frames between threads.
- [Share GPU textures with a canvas](../../../../for-kivact-user/en/how-to-guide/gpu-texture-sharing.md)
- [Connect nodes in a node graph](node-graph.md): node previews take `Pixels`, too.
- [Edit colors, easing curves and gradients](color-curves-and-gradients.md)
- Full API: `cargo doc -p kivact-ui-creative --open`
- The catalog's viewport and grading chapters: `cargo run -p kivact-ui-creative-catalog`
