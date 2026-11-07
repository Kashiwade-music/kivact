//! Role: Value lowering, resource resolution and local scene adaptation.
//!
//! Lowers shared styles, geometry, text and scenes into neutral drawing values,
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

pub(crate) mod assets;
pub(crate) mod border;
pub(crate) mod context;
pub(crate) mod conversion;
pub(crate) mod effects;
mod fill;
pub(crate) mod glyphs;
pub(crate) mod path;
pub(crate) mod sanitize;
pub(crate) mod scene;
mod scene_cache;
mod scene_out;
pub(crate) use scene_cache::SceneCache;
pub(crate) use scene_out::{LayerProps, SceneOut};
pub(crate) mod scene_shape;
pub(crate) mod shape;

mod asset_event;
pub(crate) use asset_event::AssetEvent;

pub(crate) mod input_paint;
