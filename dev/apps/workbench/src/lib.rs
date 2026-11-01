//! Role: Workbench app with 100,000 records on the core virtual list.
//!
//! A 100,000-record workbench: the core `virtual_list` (fixed row height,
//! visible range computed by the core from host scroll reports) plus the
//! external `SearchBox` and `Timeline` from `kivact-external-component`.
//!
//! Run it with `cargo run -p kivact-workbench --bin workbench`; drive it
//! headless with `kivact::test::render(|| workbench(generate_records(100_000)))`.
#![warn(redundant_imports)]

pub mod components;
pub mod model;

pub use components::app::{BUCKETS, workbench};
pub use model::{Record, RecordKey, generate_records};

/// Window title (UIA harness and `gui_scenario.json`).
pub const TITLE: &str = "Kivact Workbench";
