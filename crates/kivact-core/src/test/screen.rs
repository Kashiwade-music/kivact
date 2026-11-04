//! Role: Test host for UI testing and automation.
//!
//! `Screen` and `Element`: a rendered App on the engine's test driver
//! (a real window, real layout, shaping, rendering and accessibility, a
//! virtual clock) and the handles its queries return. The implementation
//! is split by responsibility: `driver` (lifecycle, settling, virtual
//! time, waiting), `query` (queries, failure report, `Within`), `input`
//! (user input as real shell events, Playwright-like actionability),
//! `element` (reads on an `Element`), `state` (errors, trace, stats,
//! semantic trees), `controls` (environment and platform), `structure`
//! (the core's host nodes are the engine's tree) and `artifacts`
//! (screenshots, failure artifacts) and `layout` (the layout check).

mod artifacts;
mod controls;
mod driver;
mod element;
mod ime;
mod input;
mod input_platform;
mod layout;
mod model;
mod query;
mod results;
mod state;
mod structure;

#[cfg(test)]
mod asset_tests;
#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod disposal_tests;
#[cfg(test)]
mod gpu_tests;
#[cfg(test)]
mod host_sync_tests;
#[cfg(test)]
#[cfg(test)]
mod platform_tests;
#[cfg(test)]
mod real_window_tests;
#[cfg(test)]
mod support;
#[cfg(test)]
mod window_tests;

pub use artifacts::failure_dir;
pub use model::{Contact, Element, Screen, Within};
pub use results::{
    ImeText, LayoutCheck, LayoutLimit, LayoutRule, LayoutViolation, Screenshot, SemanticNode,
};

#[cfg(test)]
mod accessibility_tests;

#[cfg(test)]
mod scene_cache_tests;
