//! Role: Value lowering, resource resolution and local scene adaptation.
//!
//! Lowers wire styles, geometry, text and scenes into neutral drawing values,
//! using Arvane image and text services where resources are required. There is
//! no node tree, layout cache, input state, window loop or publication here.
//! Callers retain traversal, diagnostic location and error-priority policy.
//!
//! Sanitization corrects supported CSS semantics and gracefully adapts local
//! commands; it does not certify a complete scene. Arvane rendering validation
//! remains the authoritative scene boundary. Resource callbacks and borrowed
//! contexts are UI-thread-local and must not re-enter their owner.
//! Each output owner retains one [`SceneCache`] for cached payloads; draw
//! output is built in a [`SceneOut`], which is append-only and counts its open
//! groups. Resource revisions and per-hit resolution protect cached pixels;
//! failures and drops are not cached.
#![forbid(unsafe_code)]

pub mod assets;
pub mod border;
pub mod context;
pub mod conversion;
pub mod effects;
mod fill;
pub mod glyphs;
pub mod path;
pub mod sanitize;
pub mod scene;
mod scene_cache;
mod scene_out;
pub use scene_cache::SceneCache;
pub use scene_out::SceneOut;
pub mod scene_shape;
pub mod shape;
