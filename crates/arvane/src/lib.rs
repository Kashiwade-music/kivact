//! Role: Arvane: the public facade of the Arvane layer.

//!
//! Responsibility: the only crate outside Arvane may depend on (via
//! `kivact-engine`). It wires the adapter crates together; the
//! adapters share `kivact-types` values and portable `arvane-shell` contracts. Text is the [`Engine`]'s; windows ([`Window`]: the native
//! [`Window`] run by [`Shell`], including explicitly stepped backends) present scenes
//! through the shared presenter, which owns the renderers, the GPU life
//! cycle and the frame policy (`presenter.rs`).
//!
//! Invariants:
//! - Public signatures use Arvane contracts, neutral `kivact-types` values and
//!   standard library types.
//!   No type from winit, taffy, parley, vello_cpu, vello_hybrid, wgpu,
//!   softbuffer, accesskit, image, resvg or skrifa is reachable from it.
//! - Arvane depends on no Kivact type (the wire included); a host lowers its
//!   own values to Arvane's. OS differences end in the windows (`Window`,
//!   `arvane-native`'s shell), GPU and CPU differences in the presenter and
//!   `arvane-render`.
//! - Textures an application's render engine shares with a window
//!   (feature `gpu-interop`) are imported into that window's own GPU device
//!   ([`Window::import_external_image`]): an image belongs to one window,
//!   and another window's scene that names it draws nothing. The `unsafe`
//!   this needs is in `kivact-gpu-share`, below `arvane-render`.
//! - What a value means where it crosses (logical coordinates,
//!   straight-alpha input pixels, premultiplied frames, a text's scalar and
//!   glyph positions and its edit byte offsets, `Arc` identity) is written
//!   on its type.
//!
//! The adapters are `arvane-text` (shaping), `arvane-render` (drawing),
//! `arvane-asset` ([`asset`]: images, SVG, fonts) and `arvane-native`
//! (windows, input, accessibility). Portable shell vocabulary and input
//! normalization and validated accessibility trees live in `arvane-shell`;
//! native decoding stays in the backend. Inside each, the values shared by
//! several modules (this crate's renderer values in `presentation.rs`,
//! `kivact-types`' image data, text's errors and metrics, assets' limits and
//! image state, render's errors, gradients, GPU state and texture making,
//! the shell's values, native input state and the shared accessibility
//! tree) sit in modules of their own, which the modules doing the work
//! refer to.

#![forbid(unsafe_code)]

pub mod embed;

mod availability;
mod engine;
mod env;
mod error;
mod presentation;
mod presenter;
mod rasterizer;
pub mod spi;
mod test_font;
mod window;

/// Images, SVG and font files: loading, decoding, the image cache and its
/// worker, fit geometry and font family resolution.
pub mod asset;
pub use arvane_render::{MAX_SCENE_DEPTH, glyph_ink_bounds};
pub use arvane_shell::{
    Accessibility, AccessibilityError, FileDialog, ShellControl, ShellError, ShellEvent,
    ShellImeText, ShellOptions, ShellWaker, WakeReasons,
};
pub use engine::Engine;
pub use error::{Failure, FailureKind};
pub use presentation::{Renderer, RendererChange};
pub use presenter::{ExportNotice, FramePlan, Presented};
pub use rasterizer::Rasterizer;

pub use availability::gpu_unavailable;
pub use presentation::Observer;
pub use test_font::test_font;
pub use window::{Handler, Shell, ShellSetup, SteppedShell, Window};
