//! Role: Contract tests of the kivact::test host: caches, windows, GPU frames, sync.
#![allow(
    clippy::type_complexity,
    reason = "the tests store handler and effect closures as boxed trait objects, as the library does"
)]

mod accessibility;
mod context;
mod disposal;
mod editing_drop_probe;
mod gpu;
mod host_sync;
mod real_window;
mod scene_cache;
mod support;
mod window;
