//! Role: Generate the DevTools JSON schemas from source; compare the committed ones.
//!
//! `KIVACT_WRITE_SCHEMA=1 cargo test -p kivact-devtools-protocol --test schema_generated`
//! rewrites `schema/*.json` after a change of the wire types.

mod committed;
mod json;
mod model;
mod read;
mod reader_tests;
mod schema;
