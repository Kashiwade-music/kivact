# Show textures from your own GPU renderer

[日本語](../../jp/how-to-guide/gpu-texture-sharing.md)

> **Experimental.** The `gpu-interop` feature works on Windows (Direct3D 12 and
> Vulkan) and Linux (Vulkan). It is off by default, the API may change without
> notice, and part of it is `unsafe`: you keep a contract with the graphics API.

Use this guide when your app has a render engine of its own, such as a 3D viewport, a
video decoder or a simulation, with its own GPU device, and you want its frames inside
a Kivact window **without reading them back to the CPU**. The engine draws into
textures; the window draws those same textures on its own GPU. They meet through OS
shared handles and two fences.

If a frame is small, or the engine is on the CPU, you do not need any of this:
publish the pixels as `Pixels` and draw them with `Draw::pixels`
([Draw with a canvas](canvas.md#show-frames-from-your-own-renderer)).

## How it works

- Your engine makes a **pool** of textures, three by default, on its own GPU device,
  and exports each texture and two timeline fences once as OS handles (NT handles on
  Windows, file descriptors on Linux).
- The window **imports** the pool into its own GPU device and draws from it.
- A **frame** is a texture slot plus a number. The engine signals the `ready` fence to
  that number when the frame is drawn. The window signals the `release` fence when it
  has finished reading, so the engine knows when a texture is free to write again.
- Kivact runs no thread of your engine. Your thread calls a small handle,
  `ShareLink`, and each call wakes the UI thread.

The texture format is RGBA8 with premultiplied alpha, `sRGB`-encoded, like `Pixels`
and CSS colors. An engine that computes in linear light encodes to sRGB as its last
step.

## Turn it on

In the application:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["gpu-interop"] }
```

`gpu-interop` includes `gpu`. In the engine, depend on `kivact-gpu-share`. It does not
know the UI, so an engine thread never needs `kivact`:

```toml
[dependencies]
kivact-gpu-share = { git = "https://github.com/Kashiwade-music/kivact", features = ["wgpu"] }
```

| Feature of `kivact-gpu-share` | You get |
|---|---|
| none | The shared types and errors. |
| `export` | Pools of textures and fences on raw Vulkan (`vulkan`) and Direct3D 12 (`d3d12`). |
| `wgpu` | `export` plus helpers for an engine on wgpu (`wgpu`). The wgpu version is pinned to `=29.0.4`, the one Kivact uses, so the two types are the same. |

If your engine uses another wgpu version, use the native helpers or follow "An engine on
wgpu of another version" in the `kivact-gpu-share` rustdoc.

Two rules decide whether sharing is possible at all:

- **Same adapter.** The engine's device must be on the adapter the window reports
  (`gpu_interop::adapter()`), not just any GPU.
- **Same graphics API.** A Direct3D 12 window shares with a Direct3D 12 engine, and a
  Vulkan window with a Vulkan engine. To choose, set `App::gpu_preference` (see
  [Choose the adapter and API](#choose-the-adapter-and-api)).

## The window side

`shared_image(width, height)` does the work for the UI. Create it in setup, give its
`link()` to the engine's thread, and draw `frame()` in a canvas. `pixels()` is a
fallback the same image carries for windows that cannot import.

```rust,ignore
use kivact::control::canvas;
use kivact::draw::{ImageOptions, Rect};
use kivact::gpu_interop;
use kivact::prelude::*;

fn viewport() -> impl IntoView {
    let image = gpu_interop::shared_image(1280, 720);

    // `ShareLink` is Send + Sync + Clone: it is the engine thread's handle.
    let link = image.link();
    std::thread::spawn(move || engine_thread(link));

    let status = image.clone();
    let view = image;
    column((
        canvas(move |d, size| {
            let rect = Rect::new(0.0, 0.0, size.width, size.height);
            // Tracked: the canvas redraws for each new frame the engine publishes.
            if let Some(frame) = view.frame() {
                d.gpu_frame(&frame, rect, ImageOptions::default());
            } else if let Some(pixels) = view.pixels() {
                d.pixels(&pixels, rect, ImageOptions::default());
            }
        })
        .style(style().grow(1)),
        text(move || format!("{:?}", status.status())),
    ))
}
```

`SharedImage` is a handle for the UI thread (`Clone`, not `Send`). It belongs to the
owner it was created in: when that component goes away, the link closes, the engine's
waits return at once and the window releases the last frame.

With the creative kit, the viewport component takes the same two sources:

```rust,ignore
use kivact::gpu_interop::shared_image;
use kivact_ui_creative::viewport_surface;

let image = shared_image(1280, 720);
let (frame, pixels, sized) = (image.clone(), image.clone(), image);
viewport_surface(move || pixels.pixels())
    .gpu_frame(move || frame.frame())
    .on_resize(move |size| sized.set_size(size.physical_width, size.physical_height))
```

It needs the kit's own `gpu-interop` feature. The surface zooms and pans the frame and
maps the pointer to image pixels.

### Follow the size of the view

`image.set_size(width, height)` asks the engine for pictures of another size, in
physical pixels. Each new size is a **new request**: the old import is dropped, the engine
makes a new pool and provides it. Do not call it for every pixel of a window drag; wait
until the size settles, or round it to steps.

## The engine side

The engine's thread owns a loop with three jobs: take the newest request, serve it, and
publish frames. A request carries the mode, the size and a token; each new one replaces
the old one.

```rust,ignore
use std::time::Duration;

use kivact_gpu_share::{FrameReady, PoolConfig, wgpu as share};
use kivact::gpu_interop::{ShareError, ShareLink, ShareMode, ShareRequest};

struct Engine {
    request: ShareRequest,
    // Drop order matters: the pool goes first, because it waits for its last frame.
    pool: share::Pool,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn engine_thread(link: ShareLink) {
    let mut engine: Option<Engine> = None;
    while !link.is_closed() {
        // A newer request replaces the pool. Without an engine, wait for one.
        let request = match engine {
            Some(_) => link.poll_request(),
            None => link.wait_request(Duration::from_millis(100)),
        };
        if let Some(request) = request {
            drop(engine.take()); // release the old pool first
            engine = match &request.mode {
                ShareMode::Gpu(_) => Engine::serve(&link, request).ok(),
                ShareMode::Pixels => {
                    serve_pixels(&link, &request); // draw on the CPU
                    None
                }
            };
        }
        if let Some(running) = &mut engine {
            if running.draw_and_publish(&link).is_err() {
                engine = None; // wait for the next request
            }
        }
    }
}
```

Serving a GPU request makes a device on the reported adapter, a pool of the request's
size, and hands the pool to the window:

```rust,ignore
impl Engine {
    fn serve(link: &ShareLink, request: ShareRequest) -> Result<Engine, String> {
        let ShareMode::Gpu(adapter) = &request.mode else {
            return Err("a Pixels request".into());
        };

        // No `InstanceFlags::DEBUG` or `VALIDATION` next to a Direct3D 12 window:
        // turning the debug layer on after a device exists removes that device.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            flags: wgpu::InstanceFlags::empty().with_env(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let (_, device, queue) =
            share::request_device(&instance, adapter, &Default::default()).map_err(|e| e.to_string())?;
        let config = PoolConfig::new(request.width, request.height);
        let (pool, textures) = share::Pool::new(&device, adapter, config).map_err(|e| e.to_string())?;

        // SAFETY: the pool comes from `kivact_gpu_share`, which keeps the contract of
        // `SharedTextures`; this engine draws into a texture only after `wait_ready`
        // and only through `Target::submit`.
        unsafe { link.provide(&request, textures) }.map_err(|e| e.to_string())?;
        link.wait_ready(&request, Duration::from_secs(5)).map_err(|e| e.to_string())?;
        Ok(Engine { request, pool, device, queue })
    }

    fn draw_and_publish(&mut self, link: &ShareLink) -> Result<(), ShareError> {
        let target = self.pool.acquire(Duration::from_secs(1)).map_err(|e| ShareError::Failed(e.to_string()))?;
        let view = target.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        draw_scene(&mut encoder, &view); // the engine's own render pass
        // SAFETY: `queue` is the pool's queue and only this thread uses it.
        let ready: FrameReady = unsafe { target.submit(&self.queue, [encoder.finish()]) }
            .map_err(|e| ShareError::Failed(e.to_string()))?;
        link.publish(&self.request, ready.slot, ready.value);
        Ok(())
    }
}
```

What each step guarantees:

1. **`wait_request` / `poll_request`** return a request only when it is newer than the last
   one this handle returned. A `Gpu` request names the adapter and graphics API; a
   `Pixels` request means the window cannot import right now (see the states below).
2. **`request_device`** opens a device on that adapter with the right extensions. A
   Direct3D 12 device from it shares its `ID3D12Device` with the window when both use
   wgpu, which is fine.
3. **`Pool::new`** makes the textures and fences and returns the handles once. Make a
   new pool for each request. Never provide one twice (`ShareError::Failed`).
4. **`provide`** hands the handles to the UI thread, which imports them. It is `unsafe`
   because the engine promises the contract in the rustdoc of
   `kivact::gpu_interop::SharedTextures`: the formats, the hand-off states, writing only
   after `wait_ready`, and only once the release fence reached the texture's last frame.
   `kivact_gpu_share`'s pools keep nearly all of it for you.
5. **`wait_ready`** returns `Ok` when the window finished importing. Draw before that and
   the textures are not yours yet.
6. **`acquire` / `submit`** hand a texture over (the `Target` that `acquire` returns
   borrows the pool until `submit`), wait for the GPU-side release of its last frame and
   put the hand-off barriers around your commands. The texture holds the frame from
   `slots` frames ago, so `LoadOp::Load` keeps it.
7. **`publish(request, slot, value)`** tells the UI a frame is ready. It wakes the UI thread.
   The newest frame wins: if the UI is slower than the engine, older frames are skipped
   and released together. A publish for a stale request, a request that is not ready or a
   closed link is ignored.

The same steps work on raw Vulkan or Direct3D 12 with `kivact_gpu_share::vulkan::Pool`
and `d3d12::Pool` (`acquire`, then on its target `record_acquire`, your commands,
`record_release`, `submit`). See the crate's rustdoc.

### Pictures on the CPU

When the window cannot import, the engine still has to show something. Serve a `Pixels`
request by drawing on the CPU, or by reading back a texture of your own, and publish the
result:

```rust,ignore
use std::sync::Arc;

use kivact::draw::Pixels;

fn serve_pixels(link: &ShareLink, request: &ShareRequest) {
    let rgba: Vec<u8> = render_on_cpu(request.width, request.height); // straight-alpha RGBA8
    link.publish_pixels(Pixels::new(request.width, request.height, Arc::from(rgba)));
}
```

`publish_pixels` is accepted whatever the request, and the newest picture wins. The UI
draws `pixels()` whenever there is no `frame()`, so a GPU request that the engine could
not serve (no matching adapter, an error in `serve`) can fall back the same way.
`share::read_back` reads a texture of the engine's own back as straight-alpha RGBA8.

## States and failures

Show `image.status()` while you develop. It is tracked, like a signal.

| `ShareStatus` | Meaning | What to do |
|---|---|---|
| `Waiting` | A `Gpu` request was issued and the engine has not provided a pool. | Check that the engine thread runs and serves the request. |
| `Importing` | The pool is provided and the window imports it. | Wait; `wait_ready` returns when it finishes. |
| `Ready` | Frames draw. | Publish frames. |
| `Pixels` | The window cannot import: it draws on the CPU, its GPU device is not up yet, or the platform is not Windows or Linux. | Serve with `publish_pixels`. |
| `Failed(error)` | The window refused the pool. `error.kind()` is `Unsupported` (no importing GPU device) or `Platform` (the textures do not match the adapter, the API or the size they say, or a device failure). The message is Arvane's. | Fix the pool. The state stays until the next request. |
| `Closed` | The image's owner is gone (its `show` turned false, its row went, the App closed). Nothing draws and the link is closed. | Stop the engine thread when `is_closed` says so. |

A **new request** comes by itself in three cases: the window's adapter changes (a new
device after a device loss, or the GPU appears after the first frames), the size changes,
or the import is lost. Each one drops the import in the window and makes your engine's
pool stale. The engine must notice (`poll_request` between frames), drop its pool and
serve the new request. After a new device every earlier import is gone.

The calls on `ShareLink` return `ShareError` values you should handle:

| `ShareError` | Meaning | What to do |
|---|---|---|
| `Stale` | A newer request replaced this one. | Drop what you made and take the new request. |
| `Closed` | The image's owner is gone. | Stop the thread. |
| `Timeout` | A wait ran out. | Treat the pool as failed and wait for a new request. |
| `Failed(message)` | The request cannot be served so: the window refused the pool, the request was `Pixels`, or textures were given twice. | Log it, drop the pool. |

`kivact_gpu_share::Error` covers the engine's own failures: a timeout waiting for the
GPU (usually a window that stopped presenting), `NoAdapter`, `Mismatch` (an argument
against the contract) and a failed call of the graphics API (`Api`). The waits are
always bounded, so a release that never comes is an error one round of the pool later,
not a hang. Dropping a pool waits up to five seconds for its last frame, then leaks the
raw objects instead of freeing them under the GPU.

## Choose the adapter and API

The window makes its GPU device first and tells you which adapter it is on. Read it with
`gpu_interop::adapter()`: `None` until the device is ready, while the window draws on the
CPU (`ARVANE_RENDERER=cpu`, or no GPU) and on other platforms. It is tracked, so a
`memo` can follow it. A `GpuAdapter` names the graphics API, the adapter's LUID or
Vulkan UUIDs, and a `generation`; a new generation means a new device.

To make the window choose a graphics API, a kind of adapter or an exact adapter, which is
useful when the engine needs a specific one, set a wish:

```rust,ignore
use kivact::gpu_interop::{GpuBackend, GpuPower, GpuPreference};
use kivact::prelude::*;

fn main() -> kivact::Result<()> {
    App::new("Viewport")
        .gpu_preference(GpuPreference {
            backend: Some(GpuBackend::Vulkan),
            power: GpuPower::HighPerformance,
            adapter: None,
        })
        .run(app)
}
```

It is a wish, not a requirement: the window falls back to another adapter or API when it
cannot honor it, and the environment variables `WGPU_BACKEND` and `WGPU_POWER_PREF`
override it. Without a wish the window prefers the low-power adapter, Vulkan first and
then Direct3D 12 on Windows. The engine always follows `adapter()`, never its own
preference.

## Limits

- **One window.** An image is drawn only in the App's window, by its GPU. A window that
  draws on the CPU draws nothing for a GPU frame; use the `Pixels` path there.
- **Windows and Linux only.** Direct3D 12 with Direct3D 12 and Vulkan with Vulkan on
  Windows; Vulkan on Linux. No other combination, and no macOS.
- **Format.** One format: RGBA8, premultiplied alpha, sRGB-encoded values.
- **You keep the contract.** `provide`, `submit` and the low-level import are `unsafe`
  because Kivact cannot check that the engine writes only where it may. Breaking the
  contract is undefined behavior of the graphics API, including a broken picture, a lost
  device or a crashed process.
- **In order, one at a time.** A pool hands textures out round robin. The target of a
  frame borrows its pool until it is submitted; drop it to give the frame up.
- **One export per import.** Some drivers (NVIDIA on Vulkan) refuse to import memory whose
  handle was exported before. Make a new pool for each request.
- **Use a pool only after the window imported it.** On Vulkan, its textures start in the
  layout the window's import puts them in; a pool that was never imported is outside
  Vulkan's rules. For `Pixels`, read back a texture of the engine's own, not the pool's.
- **Experimental API.** Names and details may change.

If your app follows the adapter and the status itself, instead of using `shared_image`,
the lower level `gpu_interop::import` imports one pool now and
`ExternalImage::publisher` carries frames from the engine's thread to
`ExternalImage::latest_frame`. `shared_image` is built on it.

## See also

- [Draw with a canvas](canvas.md) for `Draw::pixels`, `Draw::gpu_frame` and regions on the
  frame.
- [Run async work from a component](async-and-tasks.md) for threads and `blocking`.
- [Find out why something failed or updated](diagnostics.md) for `ErrorKind::Unsupported`
  and `ErrorKind::Platform` from a refused import.
- `cargo doc -p kivact --features gpu-interop --open`: `kivact::gpu_interop`.
  `cargo doc -p kivact-gpu-share --features wgpu --open`: the engine side.
