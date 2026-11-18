# Arvane host stack audit (prefix AR)

## 0. Summary

Five biggest problems:
1. **The render caches end their "frame" on every raster call, but the compositor calls the rasterizer once per chunk and per damaged rectangle.** So converted images, derived tiles, GPU image textures and atlas slots are dropped between chunks of the same window frame and rebuilt again and again. One chunk's GPU raster also costs a submission, a damaged rectangle costs a new texture, two submissions and a full re-validation and re-encode of the scene (AR-01, AR-04).
2. **The composition is diffed from scratch every frame.** `plan::build` hashes every command of every chunk byte by byte, allocates a new index per chunk, and re-plans the whole frame in a loop when there are too many targets. The engine already knows what changed (AR-03).
3. **Every window has its own wgpu Instance, Device, 18 eagerly built pipelines, vello_hybrid renderer, glyph atlas, CPU rasterizer and 256 MiB image-conversion budget.** Memory and memory budgets are multiplied the same way on the CPU path (AR-02, AR-09, AR-10).
4. **The `arvane` "facade" is not a layer.** About 60 % of it forwards to `ShellWindow`, `TextEngine`, `CpuCompositor` and `arvane-asset`, and its documented rule ("the only crate outside Arvane may depend on") is false: the engine depends on arvane-native, arvane-shell and arvane-virtual directly. The `ShellWindow` SPI has 35 methods, with dead ones and two ways to do several things (AR-06, AR-07, AR-16).
5. **The embedding added for a downstream plugin framework (≈9.8k of 16.3k native lines) spreads the child's lifecycle over five owners and stores visibility, size and scale two or three times.** It reuses the Android `SurfaceDestroyed`/`SurfaceCreated` events for "hidden", and silently drops injected input while reporting it delivered (AR-05, AR-08, AR-15).

Target shape: six runtime crates with real boundaries. `arvane-shell` is a smaller SPI. `arvane-native` and `arvane-virtual` are backends. `arvane-text` owns all font knowledge. `arvane-asset` decodes to premultiplied pixels. `arvane-render` takes an explicit frame scope across many rasters. `arvane-compose` takes damage from the engine and owns one shared GPU device per process. `arvane` shrinks to the window driver and presenter (frame policy and GPU lifecycle) with no re-exports. Test hooks move behind the `test` feature. SVG text and SVG rasterization stop carrying a second font database, shaper and rasterizer.

## 1. Current structure map

| crate | responsibility (1 line) | src lines | depended on by | boundary |
|---|---|---|---|---|
| `arvane` | presenter (frame policy + GPU lifecycle, ~1.6k) and window driver; the rest forwards (`Engine`, `Window`, `Windows`, `Rasterizer`, `asset`, `embed`, `spi`, `test_font`, `gpu_unavailable`) | 2 859 (+3 347 e2e) | kivact-engine | **Partly fake.** The presenter is real; the facade role is fake because the engine bypasses it (AR-06). |
| `arvane-shell` | window/loop/child SPI, input normalisation (Mouse, Contacts, ClickCounter, WindowInput, pipeline), accessibility acceptance (AcceptedTree, PlatformTree) | 3 461 | arvane, native, virtual, engine | Real, but the SPI is too wide (AR-07). |
| `arvane-native` | winit top-level windows (2.7k), Win32 embed (3.4k), X11 embed (4.9k), shared embed (1.5k), a11y (0.8k), native_input (0.9k), display, services, mirror + test parents (2.6k, `test` only) | 16 278 | arvane (dev), engine, virtual (opt), kivact (feature) | Real. Internally it is three backends; embed is about 60 %. |
| `arvane-virtual` | scripted stepped backend, virtual clock, mirror adapter | 1 875 | engine (`testing`), kivact-test | Real. |
| `arvane-text` | Parley shaping, measuring, caches | 2 917 | arvane | Real, but it does not own all font knowledge (AR-17). |
| `arvane-render` | vello_cpu/vello_hybrid raster of one `RenderScene`, validation, frame caches, stem darkening, glyph outline/ink | 4 526 | arvane, compose | Real, but the cache "frame" contract is broken with its only caller (AR-01). wgpu leaks through it (`GpuRaster::new(&wgpu::Device,..)`), contradicting architecture.md:36. |
| `arvane-compose` | Composition → plan (damage, caches, 3D, inline) → CPU and GPU executors, filters, shaders, external/export textures | 12 874 | arvane, kivact (feature) | Real. |
| `arvane-asset` | image/SVG decode, cache and worker, fit, font file check, **generic family table** | 3 516 | arvane | Real, except for the font-family resolution (AR-17). |
| `kivact-gpu-share` | unsafe D3D12/Vulkan/wgpu texture-share pools, import, device opening | 5 092 | compose, dev producers | Real (external engines use it). Module layout is triplicated (AR-18). |

Missing boundary: none. The GPU **device** belongs to the process, but today it lives per window inside `arvane::presenter` and `arvane-compose::GpuContext` (AR-02).

## 2. Findings

### AR-01 Render frame caches end at every raster call, so the compositor's chunks evict each other
- kind: BUG / INEFF
- where: crates/arvane-render/src/cpu.rs:133-142 (`validate::scene` + `self.images.frame(..)` inside `raster`); crates/arvane-render/src/draw/painter.rs:110 (`derived.frame`); crates/arvane-render/src/gpu/encode.rs:104-124 (`atlas.frame_evicting` + `textures.frame` per raster); callers crates/arvane-compose/src/cpu/compositor.rs:279-316 (one `raster.raster` per chunk and **per damaged rect**), crates/arvane-compose/src/gpu/exec.rs:348-380 (`raster_into` per chunk/rect).
- evidence: `FrameCache::frame` drops every entry "the frame did not use" when its closure returns (frame_cache.rs:52-77). Each `raster()` call is one such frame. A window frame with chunks A and B therefore evicts A's converted images, gradient/pattern tiles, GPU image textures and atlas slots when B is drawn. They are converted, premultiplied, uploaded and destroyed again next frame. The module docs promise "a photo left on screen is not converted again every frame" (lib.rs:12-16); that holds only when the photo's chunk is the last one rastered.
- fix: Move the frame scope to the caller. `CpuRaster::frame<R>(&mut self, f: impl FnOnce(&mut CpuFrameRaster<'_>) -> R) -> R` and `GpuRaster::frame(..)` open the image, derived and atlas scopes once. `CpuFrameRaster::raster(scene, origin, scale, size, clip, out)` draws inside them. `CpuCompositor::frame` and `GpuContext::draw_planned` wrap all chunk tasks of one window frame in one `frame(..)`. Validation runs once per chunk task, not once per damaged rect (see AR-04).
- breaking: no   kpi: K4, K5, K2   size: M
- confidence: high

### AR-02 One GPU device, pipeline set and renderer per window
- kind: STRUCT / INEFF
- where: crates/arvane/src/presenter/frame_policy.rs:81 (`gpu: Gpu::new(env.gpu)` per presenter); crates/arvane/src/presenter/gpu.rs:678-684 (`start_device` per window); crates/arvane-compose/src/gpu/context.rs:47-111 (`wgpu::Instance::new`, `request_device`, `GpuRaster::new`, `Pipelines::new`, `Shaders::new` per `GpuContext`); crates/arvane/src/window.rs:234-249 (`Windows::open` makes a presenter and begins it).
- evidence: Each window's presenter starts its own worker, which creates an Instance, an adapter, a Device, 18 render pipelines (pipelines.rs:401-470) and a vello_hybrid renderer with its own glyph atlas. Multi-window and DevTools windows pay K1 and K2 per window. A device loss is handled per window, with N restart budgets.
- fix: Create one `GpuDevice` per `Driver` (process UI thread) in a new `arvane-compose::gpu::Device` holding instance, adapter, device, queue, `Pipelines`, `Shaders` and `GpuRaster`. `GpuContext` becomes per-window frame state (`CacheIndex`, `chunks`, `kept`, `pool`, `exports`, `external`, `GpuSurface`) borrowing `&Device`. Move `presenter::gpu::State` (Off/Deferred/Background/Failed) from `Presenter` to `Driver`: one start, one restart budget, one `RendererChange` stream per device. Each presenter keeps only `DeviceSurface` and `FrameHealth`. Imported external images belong to the device, so any window may draw them; this deletes the "image belongs to one window" rule (arvane/src/lib.rs:20-24).
- breaking: no (internal; `GpuAdapter::generation` semantics unchanged)   kpi: K1, K2   size: L
- confidence: high

### AR-03 The composition is fully re-hashed and re-planned every frame
- kind: INEFF
- where: crates/arvane-compose/src/plan/damage.rs:171-245 (`hash::scene_hashes` for every command of every chunk, a new `Arc<[Indexed]>` and `retain` Vec per chunk per frame); crates/arvane-compose/src/plan/hash.rs:44-49 (FNV-1a one byte at a time); crates/arvane-compose/src/plan/builder.rs:27-41 (on more than `INLINE_TARGET_LIMIT` targets, re-run the whole `Builder` with `inline.clone()` once per inlined layer); crates/arvane-compose/src/plan/damage.rs:296-322 (`carry_reused` walks every cached chunk).
- evidence: The engine lowers a whole `Composition` each frame. Arvane re-derives what changed by hashing and diffing every `RenderCmd`, which is O(total commands) when one caret blinks. The inline loop is O(layers × plan) in the worst case.
- fix: (1) `kivact_types::render::CompositeLayer` gets `content_version: u64`, bumped by the engine when the layer's items change (the engine already tracks per-node paint dirtiness). `Builder::chunk` reuses the previous `ChunkRecord` and skips hashing when `(key, version, grid)` is unchanged. Command-level diffing runs only for changed chunks. (2) Replace FNV byte hashing with `std::hash::Hasher` (`foldhash`, already in the tree) over `u64` words. (3) Choose the layers to inline in one pass before building: sort candidates by area and inline until `targets <= INLINE_TARGET_LIMIT`, then build once.
- breaking: no (types change is internal between engine and Arvane)   kpi: K4, K5   size: M
- confidence: high

### AR-04 Damaged rectangles cost a full validate + encode each; GPU submits per chunk and per rect
- kind: INEFF
- where: crates/arvane-compose/src/cpu/compositor.rs:291-306 (loop `for rect in rects { raster.raster(scene, .., Some(*rect), ..) }`); crates/arvane-render/src/cpu.rs:85-160 (each call validates the whole scene and records every command); crates/arvane-compose/src/gpu/exec.rs:310-340 (per rect: `targets::target(&self.device, (r.width, r.height))`, which is unpooled, then `raster_into` submit plus a copy submit); exec.rs:348-380 (one encoder + `queue.submit` per chunk).
- evidence: N damaged rects mean N whole-scene validations and N full scene encodes (culling only by `visible`). On the GPU each rect also gets a fresh texture and two queue submissions, and every chunk gets one.
- fix: `CpuFrameRaster::raster_rects(scene, origin, scale, size, rects: &[PixelRect], out)` validates once, encodes once with a clip path that is the union of `rects`, and renders into the scratch once. GPU: `GpuContext::draw_planned` records every chunk raster into one `wgpu::CommandEncoder` and submits once. The per-rect patch draws through a scissor into the chunk texture: render-pass `LoadOp::Load` plus `set_scissor_rect` per rect, with no temp texture.
- breaking: no   kpi: K4, K5   size: M
- confidence: high

### AR-05 Embedded windows drop injected input and still report it delivered
- kind: BUG
- where: crates/arvane-native/src/port/embed/session.rs:69-71 (`fn inject(&mut self, _: WindowId, _: Vec<InjectedInput>, request) { self.injected.push(request) }`), session.rs:204-224 (`deliver_injected` sends `ShellEvent::Injected`).
- evidence: The `inputs` argument is discarded. DevTools input injection (`kivact-core/src/devtools/input.rs:190`, `HostRequest::Inject` → engine pump.rs:210-212) into an embedded plugin editor answers success and nothing happens.
- fix: `EmbedLoop` holds `&RefCell<VecDeque<(Vec<InjectedInput>, RequestId)>>` from `Session`. After the callback, `Session::deliver_injected` feeds each input through the window's `WindowInput` (the same `arvane_shell::WindowInput` the Win32/X11 ports use for native messages; expose `fn input(&mut self) -> &mut WindowInput` on the embed window types), then sends `Injected`.
- breaking: no   kpi: -   size: S
- confidence: high

### AR-06 `arvane` is not a facade; most of it forwards
- kind: STRUCT / CONTRA
- where: crates/arvane/src/lib.rs:4-6 ("the only crate outside Arvane may depend on (via `kivact-engine`)") vs crates/kivact-engine/Cargo.toml:11-18 (engine depends on arvane-native, arvane-shell, arvane-virtual); crates/arvane/src/engine.rs:20-201 (every method forwards to `arvane_text::TextEngine` + `error::text`); crates/arvane/src/rasterizer.rs:9-58 (forwards to `CpuCompositor::render` and `arvane_render::dimensions`); crates/arvane/src/window.rs:24-188 (30 one-line forwards to `ShellWindow`); crates/arvane/src/asset.rs, embed.rs:57-91 (`Embedded` forwards `ShellChild`), spi.rs.
- evidence: Each adapter crate already keeps vendor types out of its own API (each lib.rs states the invariant). The facade adds a third error mapping (`TextError`→`Failure`, `RenderError`→`Failure`) and a second name for every SPI type (`arvane::spi::*`, `arvane::ShellEvent`, ...).
- fix: Delete the facade role and keep `arvane` as **the window driver**: `Driver`, `Presenter` (frame_policy, gpu, health, exports, frame) and `Shell`/`SteppedShell` construction. Delete `engine.rs`, `rasterizer.rs`, `asset.rs`, `spi.rs`, `availability.rs`, `test_font.rs`, `embed.rs` (`driver` moves to `window.rs`; `Embedded` is replaced by `Box<dyn ShellChild>`), and every `pub use` of other crates. arvane-text returns `kivact_types::Failure` (move `error::text_kind` into arvane-text). The engine uses `arvane_text::TextEngine`, `arvane_compose::CpuCompositor::render` + `arvane_render::dimensions`, `arvane_asset::*` and `arvane_shell::*` directly. Update architecture.md's crate table and `dev/xtask/policy/workspace.toml` `visible_to`.
- breaking: no (internal crates)   kpi: K6 (less generic monomorphisation through forwards: minor)   size: M
- confidence: high

### AR-07 `ShellWindow` is 35 methods with dead and duplicate entries
- kind: DEAD / API / DUP
- where: crates/arvane-shell/src/window.rs:44-197.
  - `fn reduced_motion(&self)` (136): no caller through the trait (only `environment()` is read); 4 impls.
  - `fn cpu_format(&self)` (53) and `enum CpuFormat { Xrgb8888 }` (27-30): never called; one variant.
  - `fn focus(&mut self)` (50): only called by the impls' own `perform(PlatformRequest::FocusWindow)` (win32/window.rs:137, winit/window.rs:88).
  - `fn frame_period_ns(&self)` (162) duplicates `Environment::frame_period_ns` (kivact-types env.rs:34; the doc says "consistent with `environment`").
  - `fn request_size` (96) duplicates `perform(PlatformRequest::RequestWindowSize)`: two paths to one request.
  - IME is five setters (`set_ime_purpose/allowed/cursor_area/text`, `set_soft_keyboard`, 148-159) for one fact. The engine syncs them with separate `Sent` fields (kivact-engine host/window.rs:105-110: `soft_keyboard`, `ime_allowed`), which is an implicit state machine at the boundary.
  - Presenter-only methods (`keeps_frames`, `keep_frame`, `shows_cpu_frame`, `release_cpu_surface`, `recover_surface`, `present_with`, `with_gpu_target`, `wait_until`) are mixed with window services.
- fix: Delete `reduced_motion`, `cpu_format`/`CpuFormat`, `focus` (keep it as a private impl fn), `frame_period_ns` (the presenter reads `environment().frame_period`). Delete `PlatformRequest::RequestWindowSize` and keep only `request_size(size, request) -> SizeReply`. Replace the five IME setters with `fn set_ime(&mut self, ime: Option<ImeRequest>)`, where `kivact_types::ImeRequest { purpose, cursor_area, text: Option<ShellImeText>, soft_keyboard: bool }` and `None` means IME disabled. The engine keeps one `Sent<Option<ImeRequest>>`. Split the surface methods into `trait WindowSurface` (`present_with`, `with_gpu_target`, `recover_surface`, `release_cpu_surface`, `shows_cpu_frame`, `drawable()` (AR-08), `physical_size`, `scale`, `wait_until`, `keep_frame: Option<..>` (AR-19)), reached by `fn surface(&mut self) -> &mut dyn WindowSurface`. Only the presenter calls it.
- breaking: no   kpi: -   size: M
- confidence: high

### AR-08 "Can this window draw?" is derived in four places; embed hiding reuses Android surface events
- kind: STATE / CONTRA
- where: crates/arvane-native/src/port/embed/surface.rs:6-60 (`ChildSurface` Shown/Hidden → `SurfaceCreated`/`SurfaceDestroyed`); crates/arvane-virtual/src/surface.rs:6-22 (`TestSurface`, same shape); winit `Surface` (port/winit/window.rs:278); crates/arvane/src/window.rs:124-130 + presenter/frame.rs:77-97 (`no_surface`, `no_pixels`); crates/kivact-engine/src/host/window.rs:120-131 (`surface_of`: generation + `size <= 0`).
- evidence: Three backend types model `Option<generation>`. The facade re-derives "no surface" and "no pixels". The engine re-derives both again. A minimized window comes back as `Presented::Dropped(Failure{kind: Content})` (frame.rs:90-96), but `FailureKind::Content` means "invalid application-supplied data" (kivact-types platform.rs:74). An embed hide destroys and recreates the GPU swapchain on every show (`Gpu::drop_surface`), which costs plugin hosts that toggle editors (K5).
- fix: In arvane-shell: `pub enum Drawable { Yes { generation: NonZeroU64 }, Hidden, NoPixels }` and `WindowSurface::drawable() -> Drawable`. One `SurfacePresence` struct in arvane-shell (generation counter + apply(visible)) replaces `ChildSurface` and `TestSurface`. Add `ShellEvent::Visibility(bool)` for embed show/hide. The presenter keeps the GPU surface while `Hidden` and drops it only on a real `SurfaceDestroyed` (keep that variant only if a platform emits it; no supported platform does today). `Presented::NotDrawable(Drawable)` replaces `Dropped(Content|Unavailable)` for the two non-error cases. The engine stores `Drawable` as given and deletes `surface_of`.
- breaking: no   kpi: K5   size: M
- confidence: high

### AR-09 The CPU path keeps three full frames, keeps them after switching to the GPU, and never pools temp targets
- kind: INEFF
- where: crates/arvane-compose/src/cpu/compositor.rs:107-116 (`output: Pixmap` kept), 279-316 (root raster chunk kept in `stores.chunks`), 324-340 (`Pixmap::new` for every temp target every frame; the GPU has `Pool`), 376 (`Op::Filter` clones the whole input pixmap); crates/arvane/src/presenter/frame_policy.rs:249-251 (invalidate only on not-shown), 412-421 (GPU frame: `release_cpu_surface` but `self.cpu` keeps its state); crates/kivact-engine/src/host/pump.rs:727-748 (engine timer releases it 1 s later, for GPU windows too).
- evidence: A plain one-layer UI on the CPU holds the root chunk (W×H×4), the compositor output (W×H×4) and the softbuffer (W×H×4): about 24 MB at 1080p. After the GPU takes over, the CPU output and chunks stay until the engine's idle wake. The engine schedules that wake after every shown frame, even on the GPU (K3).
- fix: (1) For `CompositeLayer` root content with default effects (the `chunk_shortcut` case at layers.rs:66), let the CPU executor raster that chunk **directly into `output`** with damage rects, and keep no chunk copy. (2) `CpuCompositor` gets `pool: Vec<Pixmap>` reused by size, like `gpu::targets::Pool`. (3) `Op::Filter` borrows the input (`filters::apply(filter, &targets[input], ..)` writing into a pooled pixmap; use `split_at_mut` when output ≠ input). (4) `Presenter::draw_native` calls `self.cpu.invalidate()` when the GPU draws its first frame. The engine's `release_idle_frames` runs only when `Presented::Shown { renderer: Renderer::Cpu, .. }`.
- breaking: no   kpi: K2, K4, K3   size: M
- confidence: high (1: medium; the root chunk is drawn via `Op::Draw` onto target 0)

### AR-10 Every image is held up to three times
- kind: INEFF
- where: crates/arvane-asset/src/lib.rs:44-46 ("AssetImage are straight-alpha RGBA8"), store/asset_store.rs:25 (256 MiB budget); crates/arvane-render/src/images.rs:24-26 (`MAX_BYTES` 256 MiB of premultiplied copies **per `CpuRaster`**, so per window plus the offscreen `Rasterizer`), cpu.rs:174-188 (premultiplied copy via `ImageAlphaType::Alpha`); GPU textures in gpu/images.rs.
- evidence: The asset store decodes to straight alpha, so vello_cpu premultiplies a second copy and caches it under a second 256 MiB budget per window. The GPU uploads a third copy. Budgets multiply with windows.
- fix: `arvane-asset` produces premultiplied RGBA8 (`kivact_types::Pixels` gets `alpha: Alpha::Premultiplied` or the type is documented premultiplied, since all decoders write through `pixels.rs`). arvane-render passes it as `ImageAlphaType::AlphaPremultiplied` without conversion and deletes `images.rs`' CPU conversion cache (keep only the GPU texture cache, scoped per frame per AR-01). Straight alpha is produced only where a consumer needs it (`kivact-svg` PNG export, CPU `unpremultiplied` exports).
- breaking: yes if `Pixels` is public (it is re-exported via kivact `draw`); state the alpha mode on the type   kpi: K2, K4   size: M
- confidence: high

### AR-11 SVG carries a second font database, shaper and rasterizer
- kind: INEFF / DUP
- where: crates/arvane-asset/Cargo.toml (`resvg` with `text`, `system-fonts`, `raster-images`); src/svg/fonts.rs:27-34 (`with_font` drops the whole database on every app font registration), :46-52 (`fontdb::Database::load_system_fonts()` parses every system font file a second time, after fontique's scan); src/svg/document.rs:143-154 (`tiny_skia::Pixmap` + `resvg::render`); Cargo.lock: `tiny-skia`, `rustybuzz`, `fontdb`, `png 0.17.16` (tiny-skia) next to `png 0.18.1`, `roxmltree` ×2, `miniz_oxide` ×2.
- evidence: The binary ships two rasterizers (vello_cpu, tiny-skia), two shapers (harfrust via parley, rustybuzz via usvg), two font databases (fontique, fontdb) and two PNG codecs. The first SVG with text rescans every system font on the worker. SVGs are rasterized to bitmaps at one device size instead of drawn as vectors.
- fix: Keep `usvg` (parse + text-to-path) and delete `resvg` rendering and `tiny-skia`. Convert the `usvg::Tree` to a `RenderScene` (paths, gradients, patterns, images, groups with opacity, clips and masks map to existing `RenderCmd`s) in a new `arvane-asset/src/svg/scene.rs`. The asset store then caches a `RenderScene` per SVG, which vello draws at any scale. Supply SVG text through `usvg::Options::font_resolver`, backed by font bytes from arvane-text's collection (make the fontique `Collection` `shared: true` and pass a `SharedFontSource` handle to the asset store). Delete `fontdb::load_system_fonts`, `SvgFonts::with_font` and the `system-fonts` feature.
- breaking: no   kpi: K6, K2, K1 (first SVG)   size: L
- confidence: medium (usvg 0.46 `font_resolver` API shape to be checked during implementation)

### AR-12 Startup work on the UI thread: system font scan, asset worker, per-window CPU rasterizer
- kind: INEFF
- where: crates/arvane-text/src/text_engine.rs:19-31 (`#[derive(Default)] TextEngine` → `FontContext::default()` → fontique `CollectionOptions { system_fonts: true }` → `System::new()` scans system fonts eagerly; fontique-0.11.1 collection/mod.rs:55-61, 251-252); crates/arvane-asset/src/store/asset_store.rs:88-90 (worker thread with 32 MiB stack spawned at construction).
- evidence: The system font enumeration (DirectWrite collection or fontconfig `FcFontList`) runs on the UI thread before the first measure. The asset worker exists even when the app has no image.
- fix: `TextEngine::new()` builds the collection with `system_fonts: false, shared: true` and starts the system scan on a thread (`std::thread::spawn(|| SystemFonts::new())`) at `Shell::run`. The first `shape`/`measure` that needs a system family joins it (`OnceCell<Result<..>>`). Text that only uses registered fonts never waits. `AssetStore::new` stores the waker and spawns the worker on the first `Job`.
- breaking: no   kpi: K1   size: S
- confidence: medium (exact fontique API for deferred system load: `Collection::load_system_fonts` exists at mod.rs:95)

### AR-13 All 18 compositor pipelines are compiled at device start
- kind: INEFF
- where: crates/arvane-compose/src/gpu/pipelines.rs:401-470 (`Pipelines::new`: 12 draw pipelines for 11 Porter–Duff composites plus the shader blend, the filter-graph pipeline with lighting and turbulence WGSL, filter, mask, two apply, clear).
- evidence: Almost every frame uses only `SourceOver`, `apply` and `clear`. The graph shader (`graph_ops`/`graph_light`/`graph_turb`) is the largest WGSL module and is used only by SVG-filter-like layer filters. This delays "GPU ready" (the switch from CPU) on every launch and, today, for every window (AR-02).
- fix: `Pipelines { draw: [OnceCell<wgpu::RenderPipeline>; 12], graph: OnceCell<..>, filter: OnceCell<..>, mask_step: OnceCell<..>, .. }` with `fn draw(&self, device, slot) -> &RenderPipeline`, the same pattern as the existing `present_for` (pipelines.rs:473-494). Build `SourceOver`, `apply_*` and `clear` eagerly; everything else on first use.
- breaking: no   kpi: K1 (time to GPU), K2   size: S
- confidence: high

### AR-14 Exports: two change-detection rules and a third copy of the pool list
- kind: DUP / INEFF / CONTRA
- where: crates/arvane-compose/src/cpu/compositor.rs:430-466 (CPU `ExportRecord.content = content_hash(px)`: SipHash over every pixel byte of every export each frame); crates/arvane-compose/src/gpu/export_pool.rs:29-41 (GPU `ExportRecord.signature`: the layer's content signature); crates/arvane/src/presenter/exports.rs:20-58 (`ExportRegistry.pools: HashMap<DrawableId, u64>`, a copy of `GpuContext.exports.pools`).
- evidence: The same "did this drawable change" question has two answers, and the CPU one is O(pixels) per frame. The presenter mirrors the GPU pool list so it can report "lost" after the context is gone.
- fix: The plan returns `PlannedExport { sink, target, signature }` for both executors, and the CPU `ExportRecord` uses `signature` (delete `content_hash`). With AR-02 the device outlives windows, so `GpuContext` (device-level `Exports`) reports `ExportNotice::Lost` itself from `Device::lost()`; delete `ExportRegistry.pools` and keep only `notices`.
- breaking: no   kpi: K4   size: S
- confidence: high

### AR-15 The embedded child's lifecycle has five owners and duplicated facts
- kind: STATE
- where: crates/arvane-native/src/port/embed/child.rs:37-79 (`Inner { state: ChildStatus{Running|Stopped{cause: Stop, window: NativeWindow}}, applied: Applied{width,height,scale,visibility}, host: HostSize, commands }`); session.rs:13-38 (`ChildLifecycle {Opening|Live|Disposed|Leaked}` × `HandlerSlot {Unstarted|Running}`); win32/state.rs:14-19, 101-136 (`StartSlot`, `surface: Cell<ChildSurface>`, `in_callback`, `draining`); arvane-shell child.rs:69-113 (`PanicSlot` message, also implied by `Stop::Panicked`); x11/window.rs:29-115 (`width/height/scale` fields plus `shared.update(..)`).
- evidence: Visibility is stored in `Applied.visibility` and in `ChildSurface`, synced in `apply_visibility` (win32/window.rs:580-625; x11/window.rs:108-115). Size and scale are stored in the X11 `Window` fields and in `Applied`. "Panicked" is stored in `PanicSlot` and in `Stop::Panicked`. Whether the child runs App code is derived from `ChildStatus`, `ChildLifecycle`, `HandlerSlot` and `StartSlot` together. `Session::opening()/live()/active()` are projections of the same thing.
- fix: One UI-thread owner `ChildMachine` in `port/embed/session.rs`: `enum Child { Opening { make: MakeHandler }, Live { handler: Box<dyn ShellHandler>, control: ShellControl }, Stopped { cause: ChildStop, handler: Option<..> }, Leaked }`. That deletes `StartSlot`, `HandlerSlot`, `ChildLifecycle` and `ChildStatus.window` (`NativeWindow` Live/Gone becomes the `Stopped` transition argument). The cross-thread `Shared` holds only `commands`, `HostSize`, and an `Applied` snapshot written **only** by `ChildMachine::publish()`. UI-thread code reads its own `Applied` (delete the X11 `width/height/scale` fields and `ChildSurface`'s visibility; derive `Drawable` from `Applied.visibility` per AR-08). `PanicSlot` keeps the message only; `Stop::Panicked` has no payload.
- breaking: no   kpi: -   size: M
- confidence: medium (read child.rs, session.rs, state.rs, resize.rs; not every wndproc reentry path)

### AR-16 Three configuration carriers and two handler traits for one driver
- kind: DUP / API
- where: crates/arvane/src/window.rs:191-207 (`trait Handler`, the same as `arvane_shell::ShellHandler` with `Windows` instead of `ShellLoop`), 294-353 (`Shell { prefer_gpu, gpu_preference, observer }` with three builders), 355-371 (`ShellSetup`, the same three fields), 64-69 and 281-286 (`capabilities()` overriding `gpu` twice); crates/arvane/src/embed.rs:15-55 (`Embedder { handler, observer }` + `EmbedOptions { prefer_gpu, gpu_preference }`); `Windows` clones `observer` on every event (window.rs:435-500).
- fix: One `pub struct DriverOptions { gpu: GpuPreference /* prefer_gpu folded in as GpuStart::Never */, observer: Option<Observer> }`, consumed by `Shell::run(backend, options, handler)`, `Shell::start_stepped(backend, options, handler)` and `driver(options, make)` for embed. Add `GpuStart::Never` to `kivact_types::GpuStart` and delete `prefer_gpu` everywhere (`EmbedOptions.prefer_gpu`, `Shell::prefer_gpu`, `ShellSetup.prefer_gpu`, `RendererStart::resolve`'s `prefer_gpu` arm). Delete `ShellSetup` and `Embedder`. Keep `Handler` (it carries presenters) but delete the forwarding `Window` methods that AR-07 removes. `Driver` stores `observer` once in `Notes` (an `Rc` shared by presenters) instead of cloning it into every `Windows`.
- breaking: yes (`EmbedOptions.prefer_gpu` is public in kivact-types embed.rs:55; `GpuStart` gains a variant)   kpi: -   size: S
- confidence: high

### AR-17 Font knowledge is spread over four crates
- kind: STRUCT / DUP
- where: crates/arvane-asset/src/font.rs:93-110 (`GenericFamily::candidates`: per-OS tables including macOS, which has no backend), 142+ (`resolve_family`); crates/arvane/src/engine.rs:108-120 (comma-list and quoted-name special case before calling asset); crates/arvane-text/src/build.rs:233-240 (Parley fallback chain: system-ui, sans-serif, symbol and emoji fonts); crates/arvane-render/src/outline.rs, ink.rs, validate/fonts.rs (skrifa font reads); crates/arvane-asset/src/svg/fonts.rs (fontdb).
- evidence: "Which font draws this family" is answered by asset's table, by fontique's generic mapping inside Parley, and by the facade's comma check. Font-file reading lives in asset (`load_font`), render (`glyph_outline`, `glyph_ink_bounds`, maxp validation) and text.
- fix: arvane-text owns fonts: `TextEngine::resolve_family(&mut self, name) -> FamilyResolution` (moved from engine.rs + asset font.rs; generic families resolved through fontique's `GenericFamily` mapping, so the hand-written per-OS table is deleted), `load_font(&Asset)` (moved from asset), `glyph_outline(&GlyphRun)` and `glyph_ink_bounds(&GlyphRun)` (moved from render). arvane-render keeps only the glyph-count validation it needs to rasterize. Delete `Os`, `Os::MacOs` and `GenericFamily::candidates`.
- breaking: no   kpi: K6 (one table less)   size: M
- confidence: high

### AR-18 kivact-gpu-share: always built with import + export; three device modules
- kind: INEFF / STRUCT
- where: crates/arvane-compose/Cargo.toml:19-22 (`gpu = [.. "dep:kivact-gpu-share"]` with `features = ["import", "wgpu"]`; `wgpu` implies `export`); crates/kivact-gpu-share/src/lib.rs:157-192 (`vulkan_raw` + `vulkan/raw` re-export, `wgpu_device` + `wgpu/device` + `device`, public `d3d12_contract` / `vulkan_contract` next to `d3d12` / `vulkan`).
- evidence: Without `gpu-interop`, compose needs only `device::identity` (context.rs:155-160, adapter selection by `GpuPreference::adapter`). Every default build still compiles ash and the Direct3D 12 `windows` crate plus all pools. The device-opening code exists in three modules.
- fix: Move `identity(adapter) -> Option<AdapterIds>` into a featureless `kivact-gpu-share::adapter` module (wgpu-only, no ash/windows). compose `gpu` uses only that; `gpu-interop = ["gpu", "kivact-gpu-share/import", "kivact-gpu-share/wgpu"]`. Merge `wgpu_device.rs`, `wgpu/device.rs` and `device.rs` into `device.rs`. Fold `vulkan_raw.rs` into `vulkan/raw.rs` and `*_contract.rs` into `vulkan.rs` / `d3d12.rs` (public paths `vulkan::API_VERSION`, `vulkan::DEVICE_EXTENSIONS`, `d3d12::*` stay).
- breaking: yes for engine authors using `kivact_gpu_share::vulkan_contract::*` / `d3d12_contract::*` paths   kpi: K6 (compile time, binary)   size: S
- confidence: medium (the linker likely drops unused pool code; ash/windows static init and compile time are certain)

### AR-19 Test hooks in the release surface
- kind: DEAD / PROCESS
- where: crates/arvane/src/test_font.rs:8-14 (`include_bytes!` Tuffy, 172 KB, `pub fn test_font`); crates/arvane/src/availability.rs:3-9 (`gpu_unavailable()` builds a whole `GpuContext`, with device and 18 pipelines, to answer a yes/no); crates/arvane/src/spi.rs:10-12 (`pub fn lose_gpu_device`), presenter/frame_policy.rs:183-192; crates/arvane-shell/src/window.rs:56-59 (`keeps_frames`, `keep_frame` for test windows); crates/arvane-native/src/native_input/windows.rs:248, 531-570 (`ArvaneDebugDpiChange` registered message under `cfg!(debug_assertions)`); crates/arvane-render/src/text.rs:84-88 (`dilation(size, bucket, d_full, l_sat, s_full, s_none)`: constants passed as parameters only so the test can pass the same constants).
- fix: Gate `test_font`, `gpu_unavailable` (rewrite as `GpuDevice::probe()` that only requests an adapter) and `lose_gpu_device` behind an `arvane` feature `test`, enabled by `kivact-engine/testing`. Replace `keeps_frames`/`keep_frame` with `WindowSurface::frame_sink(&mut self) -> Option<&mut dyn FnMut(CpuFrame)>`, `None` for native windows. Move the debug DPI message to the `test` feature (the E2E driver that sends it is test-only). `dilation` uses the constants directly.
- breaking: no   kpi: K6 (minor)   size: S
- confidence: high

### AR-20 Small dead or over-general pieces
- kind: DEAD / DUP
- where and fix (each S, high confidence):
  - crates/arvane/src/presenter/frame_policy.rs:56, 362-368: `limits_noted: HashSet<String>` has one key ever (`"gpu_limit"`, line 427). Make it `gpu_limit_noted: bool` and make `RendererChange::LimitExceeded` a unit variant `GpuLimit`.
  - crates/arvane-native/src/wake.rs:1-33: table-driven `encode`/`decode` of a type that is already a `u8` bitset. Add `WakeReasons::bits(self) -> u8` / `from_bits(u8)` in arvane-shell types.rs and delete wake.rs.
  - crates/arvane-compose/src/lib.rs:16: `pub use arvane_render::RenderError as ComposeError` gives one type two names (VOC-3). Use `RenderError` everywhere.
  - crates/arvane-compose/src/gpu/state.rs:115-144: `DeviceLost { message: Mailbox<String>, access: Mutex<()> }` with take-and-put-back is `std::sync::OnceLock<String>` (`record` = `let _ = lock.set(m)`, `message` = `get().cloned()`).
  - crates/arvane-compose/src/gpu/state.rs:170-178: `GpuSurface { keep: Retain, last: Option<Texture> }` has "last only when keep" (S-3). Make it `kept: Option<Option<wgpu::Texture>>` or `enum Retain { Drop, Keep(Option<Texture>) }`.
  - crates/arvane-native/src/port/win32/dpi.rs:13-66: `GetProcAddress` + `transmute` for `SetThreadDpiAwarenessContext`/`GetWindowDpiAwarenessContext`/`GetDpiForWindow`, all in `windows-sys` `Win32_UI_HiDpi` (already enabled in Cargo.toml) since Windows 10 1607. Call them directly.
  - crates/arvane-native/src/port/x11/poll.rs:27-84: a timerfd only to give `poll` a timeout. Pass the timeout to `rustix::event::poll(&mut fds, Some(&ts))` and delete `Waiter.timer`.
  - crates/arvane/src/engine.rs:20-25: `RefCell<TextEngine>` only so `font_families` takes `&self`. Make it `&mut self` (disappears with AR-06).
  - crates/arvane-native/src/port/win32/window.rs:33-43 and 383-389: the embed `Environment` is assembled in two places. Use one `fn environment(shared: &Shared) -> Environment`.
- breaking: no   kpi: -   size: S each

### AR-21 Docs that contradict the code
- kind: CONTRA
- where and fix:
  - crates/arvane-native/src/lib.rs:36-37 says Linux "Mouse settings and reduced motion are not read from the desktop", but port/linux_settings.rs reads reduced motion from the XDG settings portal. Fix the sentence.
  - docs/for-kivact-developer/architecture.md:36 says arvane-render closes over "Vello と wgpu", but `GpuRaster::new(&wgpu::Device, &wgpu::Queue)`, `Canvas` and `RASTER_FORMAT: wgpu::TextureFormat` are public and arvane-compose and kivact-gpu-share depend on wgpu. Reword it as "vello is closed inside; wgpu is shared with arvane-compose, which owns the device".
  - crates/kivact-types/src/platform.rs:66-68: `PlatformRequest::ForeignChild` says "Create the native child window", but the child is created at window open (port/winit/open.rs:85, 107-118) and `perform` returns it (port/winit/window.rs:101-102). Delete the request; add `ShellWindow::foreign_child(&self) -> Option<NativeWindow>`.
  - crates/arvane/src/lib.rs:4-6 (see AR-06).
- breaking: no (ForeignChild is internal)   kpi: -   size: S
- confidence: high

### AR-22 Foreign windows get a presenter and can start a GPU device
- kind: INEFF
- where: crates/arvane/src/window.rs:234-249 (`Windows::open` makes a presenter and calls `begin` for any `WindowRequest`); content type at kivact-types `WindowContent::Foreign`.
- evidence: A window whose content another library draws never presents Kivact frames, yet with `GpuStart::AtLaunch` (or `ARVANE_RENDERER=gpu`) it starts a device worker for itself (today per window, AR-02).
- fix: `Windows::open` creates a presenter only for `request.content == WindowContent::Drawn`. `Windows::get` returns a `Window` without presenter methods for foreign windows (`presenter: Option<&mut Presenter>`, with `present` returning `Presented::NotDrawable`).
- breaking: no   kpi: K1, K2   size: S
- confidence: medium (engine never presents to foreign windows: not traced end to end)

### AR-23 `Gpu::State::Off` means three things
- kind: STATE
- where: crates/arvane/src/presenter/gpu.rs:257-276 (`Off`: CPU-only by choice, presenter not begun yet, or shut down), 286-296 (`shutdown` resets to `Off`), 387-397 (`begin` asserts `Off`).
- fix: `enum State { New, CpuOnly, Deferred{..}, Background(..), Failed, ShutDown }`. `begin` requires `New`; `shutdown` → `ShutDown`; any call in `ShutDown` is a `debug_assert!` violation. Implement this together with moving the machine to `Driver` (AR-02).
- breaking: no   kpi: -   size: S
- confidence: high

### AR-24 The font validation cache is a linear Vec scanned with `retain` per glyph run
- kind: INEFF
- where: crates/arvane-render/src/validate/fonts.rs:20-48 (each `glyphs()` call: `entries.retain(strong_count)` over up to 256, then `position` linear search, then `remove` + `push` shifting).
- fix: `HashMap<(usize /*ptr*/, usize /*len*/, u32 /*index*/), Entry>` (Entry keeps the `Weak`), pruning dead `Weak`s once per frame scope (AR-01), not per lookup.
- breaking: no   kpi: K4   size: S
- confidence: high

### AR-25 The accessibility tree is validated and stored again at the shell
- kind: DUP
- where: crates/arvane-shell/src/a11y/shared.rs:63-110 (`validation::canonicalize` on every replace, `AcceptedTree::apply` on every update, also while no assistive technology is active), state.rs (full node copy).
- evidence: The engine is the only producer of `SemanticsTree`, so its output is re-validated at the boundary, and a full second copy (`AcceptedTree`) is kept beside the engine's model and AccessKit's tree. The copy is needed only for AccessKit's synchronous initial-tree request and to check action targets.
- fix: Validation of engine output becomes `debug_assert!`-only (`#[cfg(debug_assertions)] validation::canonicalize`). Keep `AcceptedTree`; it is the activation snapshot (architecture.md "a11y の前回 accepted tree").
- breaking: no   kpi: K4, K2   size: S
- confidence: medium

### AR-26 Process noise in the area
- kind: PROCESS
- where: 136 `xtask-allow(state-fields)` comments in the area; 67 in arvane-text, e.g. crates/arvane-text/src/cache.rs:113-140, where 13 consecutive fields of a hash key `SpanBits` repeat "cache: one component of a cache key; equality of the whole key is what matters". Also `architecture/` diagram churn: the embedding PR rewrote `arvane_native-lib-normal.svg` (1 519 lines) and `-test.svg` (1 567 lines).
- fix: The state-fields check must skip types that derive `Hash + Eq` and are used only as map keys, and fields of `*Bits`/`*Key` structs. Delete the per-field comments. Stop committing generated diagrams (generate on demand in CI artefacts).
- breaking: no   kpi: -   size: S
- confidence: high

## 3. Target structure for this area

| crate | owns | boundary justification (inputs / outputs) |
|---|---|---|
| `arvane-shell` | `ShellWindow` (window services: perform, size, environment, title/min/resizable, cursor, `set_ime(Option<ImeRequest>)`, a11y replace/update, `surface()`), `WindowSurface` (present_with, with_gpu_target, recover, `drawable() -> Drawable`, frame_sink), `ShellLoop`, `ShellHandler`, `Backend`, `SteppedBackend`, `ShellChild`, `ShellEvent` (+`Visibility(bool)`), `WakeReasons` (+bits), `SurfacePresence`, input normalisation, accessibility acceptance | In: OS-free vocabulary; out: the SPI. No OS, no renderer. |
| `arvane-native` | winit port, Win32 embed, X11 embed, `ChildMachine` (one embed lifecycle), a11y (AccessKit), services, display; `test` feature: mirror + TestParent | Implements the SPI with OS crates. Embed stays a module (it shares a11y, services, native_input, present). |
| `arvane-virtual` | scripted stepped backend | Implements the SPI without an OS. |
| `arvane-text` | Parley shaping/measure, **all font knowledge** (registration, family resolution, generic families, `load_font`, `glyph_outline`, `glyph_ink_bounds`, deferred system scan, shared font source for SVG text); errors as `kivact_types::Failure` | In: text + `ComputedTextStyle` + font bytes; out: shaped snapshots, metrics, outlines. |
| `arvane-asset` | decode (premultiplied), SVG → `RenderScene` via usvg (no resvg/tiny-skia/fontdb system scan), cache + lazily started worker, fit | In: `Asset`; out: `Pixels`/`RenderScene`, `ImageState`. |
| `arvane-render` | vello raster of `RenderScene`, with `CpuRaster::frame(..)` / `GpuRaster::frame(..)` scopes, `raster_rects`, validation, stem darkening | In: scenes and a frame scope; out: pixels/encoded passes. |
| `arvane-compose` | plan (damage from `content_version`), CPU executor (direct root raster, pooled targets), GPU `Device` (process-wide: instance, device, lazy pipelines, raster, imports, export pools) + per-window `GpuContext` (caches, surface) | In: `Composition` + device; out: frames, `FrameReport`, export notices. |
| `arvane` (driver) | `Driver`, `Presenter` (frame policy, `FrameHealth`), the device `State` machine (one per Driver), `Shell::run` / `start_stepped` / embed `driver`, `DriverOptions` | In: SPI loop + compose; out: `Handler` callbacks with `Windows`/`Window`, `Presented`, `RendererChange`. |
| `kivact-gpu-share` | `adapter` (featureless identity), `device`, `vulkan`, `d3d12`, `wgpu`, `import`, `pool` | Public crate for external engines; no UI. |

Deleted: `arvane::{Engine, Rasterizer, asset, spi, embed::Embedded, embed::Embedder, ShellSetup, test_font (→ test feature), gpu_unavailable (→ test feature), availability.rs, env.rs's prefer_gpu}` and all `pub use` re-exports of other crates; `arvane-native/src/wake.rs`; `arvane-native/src/port/embed/surface.rs` (→ shell `SurfacePresence`); `arvane-virtual/src/surface.rs`; `CpuFormat`; `ShellWindow::{reduced_motion, cpu_format, focus, frame_period_ns, set_ime_* ×4, set_soft_keyboard, keeps_frames, keep_frame}`; `PlatformRequest::{RequestWindowSize, ForeignChild}`; `arvane-asset::{Os, GenericFamily::candidates, svg/fonts.rs system scan}`; resvg rendering + tiny-skia; `arvane-render/src/images.rs` CPU conversion cache; `arvane/src/presenter/exports.rs` `pools`; `ComposeError` alias; `kivact-gpu-share::{vulkan_raw, wgpu_device, *_contract}` as separate modules.

Merges and splits: no new crates. Do not split embed out of arvane-native: it reads a11y, services, native_input, present and module internals, so the boundary would be fake. Do not merge arvane-render into compose: render's input (one `RenderScene`) and output (pixels or encoded pass) are small, and vello stays closed there.

## 4. Public API

The area reaches app code only through kivact-core's launch options, `EmbedOptions`, `ARVANE_RENDERER` and `kivact-gpu-share`.

`prefer_gpu` is never `false` outside tests: kivact-core/src/embed/open.rs:75 sets `true`, and only kivact-engine/src/host/testing/test_host.rs:243 derives it from `GpuMode`. It is a second switch next to `GpuPreference.start`.

Before:
```rust
EmbedOptions { prefer_gpu: true, gpu_preference: GpuPreference::default(), .. } // kivact-types/src/embed.rs:55
App::new(root).gpu_preference(GpuPreference { start: GpuStart::OnDemand, ..Default::default() })
```
After (one switch):
```rust
EmbedOptions { gpu_preference: GpuPreference::default(), .. }
App::new(root).gpu_preference(GpuPreference { start: GpuStart::Never, ..Default::default() }) // CPU only
```
- Delete: `prefer_gpu` (every carrier), `EmbedOptions.prefer_gpu`.
- Add: `GpuStart::Never`.
- Rename: `ARVANE_RENDERER=cpu|gpu|auto` maps to `Never|AtLaunch|OnDemand` and keeps its spelling.
- `kivact_types::Pixels` states its alpha mode (AR-10), premultiplied by default.
- kivact-gpu-share: `vulkan_contract::*` → `vulkan::*`, `d3d12_contract::*` → `d3d12::*`.

## 5. Open questions for the owner

1. **Process-wide GPU device (AR-02).** Accept that one device failure moves *all* windows to the CPU for one restart, and that imported external images become drawable in every window? Recommended: yes.
2. **Stem darkening keys on the text colour only** (arvane-render text.rs). White text on a light background is darkened, and the rule also applies on Linux, though it was fitted to DirectWrite. It multiplies glyph atlas entries up to ×8 (eight luminance buckets). Keep 8 buckets, reduce to 2 (dark: 0, light: `D_FULL`), or apply it only on Windows?
3. **SVG as vector `RenderScene` (AR-11)** changes how SVG filters and other features resvg supports render: unsupported filter primitives would draw without the filter. Accept, or keep resvg only for SVGs that contain `<filter>`?
4. **`content_version` on `CompositeLayer` (AR-03)** makes the engine responsible for bumping versions correctly; a missed bump shows stale pixels. Accept, with a debug build that cross-checks the hash, or keep hashing and only fix the hash speed?
5. **Android-style `Suspended/Resumed/SurfaceDestroyed/SurfaceCreated/SaveState/KeyboardInset/MemoryWarning` events** have no emitter on any supported platform once embed uses `Visibility` (AR-08); only arvane-virtual scripts them. Delete them until a mobile backend exists (strategy says not implemented)?
