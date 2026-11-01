//! Role: Optional GPU rasterization (feature `gpu`).
//! Handles remain private and
//! frames use core vocabulary.
//!
//! Responsibility: one wgpu device and vello_hybrid renderer
//! ([`GpuContext`]), presenting scenes to window surfaces ([`GpuSurface`],
//! `surface.rs`) without reading pixels back, and reading frames back only
//! where pixels are needed (tests, screenshots).
//!
//! vello_hybrid is vello_cpu's pipeline with its last stage on the GPU: the
//! CPU flattens paths and makes sparse strips exactly as vello_cpu does
//! (`draw.rs` records the same calls for both), and ordinary render
//! pipelines (no compute shaders) fill the strips' pixels into the target.
//! Its GPU memory follows the window and the scene (strips, a depth
//! texture, glyph atlas pages once text is drawn, a texture per image,
//! layer textures once a layer is used), with no fixed working buffers.
//!
//! Adapter choice: hardware adapters only, through the backends that start
//! fastest first (Vulkan, then DirectX 12 on Windows; the platform's
//! primary backends elsewhere), or the backend the application asks for
//! (`GpuPreference`; `WGPU_BACKEND` overrides both); among them the
//! adapter the application names (feature `gpu-interop`, looked up by
//! LUID or UUIDs), else the one its power preference picks
//! (`WGPU_POWER_PREF` overrides). With `gpu-interop`, a Vulkan device is
//! opened with the extensions shared textures need when the adapter has
//! them, and the context reports its adapter (`interop_adapter`) when it
//! can import. Software adapters
//! (WARP, lavapipe) are refused: vello_hybrid runs on WARP, but a screen of
//! text takes 2.5 times as long as vello_cpu takes for it and the adapter
//! adds ~90 MB. The facade falls back to the CPU renderer instead (on
//! Linux the GPU is a hardware Vulkan adapter; WSLg offers only lavapipe,
//! without D3D12's Vulkan driver dozen, so it draws on the CPU). wgpu is
//! built with the Vulkan, DirectX 12 and Metal backends only; Vulkan
//! drivers cache the compiled pipelines between runs. The device asks for
//! `MemoryHints::MemoryUsage` (smaller allocation blocks: a UI needs
//! little GPU memory), and a new context draws one small blank frame, so
//! the renderer's first buffers are made then rather than in a window's
//! first frame. How devices are made is read from the environment once
//! ([`GpuEnv::from_env`]).
//!
//! The feature `gpu` of `kivact` (on by default) is forwarded through
//! `kivact-core`, `kivact-engine` and `arvane` to this crate's; without it
//! only the CPU renderer exists.
//!
//! Invariants:
//! - GPU resources are serially owned by their context; a context may be
//!   created on a worker thread and moved to the presenting thread.
//! - Validation errors and device loss never panic: they are recorded and
//!   reported by [`GpuContext::failure`] and by the next call, so the
//!   facade can recreate the device or fall back to the CPU.
//! - Readback waits are bounded.

mod context;
mod encode;
pub(crate) mod external;
mod offscreen;
pub(crate) mod state;
mod surface;
mod texture;
