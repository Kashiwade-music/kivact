//! Role: Test host for UI testing and automation.
//!
//! `Screen` and `Locator`: a rendered App on the engine's test driver
//! (a real window, real layout, shaping, rendering and accessibility, a
//! virtual clock) and the handles its queries return. The implementation
//! is split by responsibility: `driver` (lifecycle, settling, virtual
//! time, waiting), `query` (queries, failure report, `Within`), `input`
//! (user input as real shell events, Playwright-like actionability),
//! `element` (reads on an `Locator`), `state` (errors, trace, stats,
//! semantic trees), `controls` (environment and platform), `artifacts`
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

pub use artifacts::failure_dir;
pub use model::{Contact, Locator, Screen, Within};
pub use results::{
    ImeText, LayoutCheck, LayoutLimit, LayoutRule, LayoutViolation, RasterDifference, Screenshot,
    SemanticNode,
};
