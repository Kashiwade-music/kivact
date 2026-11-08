//! Role: Generate the DevTools protocol JSON Schema and Python types from the protocol crate.

mod check;
mod cli;
mod json;
mod model;
mod python;
mod read;
mod schema;
#[cfg(test)]
mod tests;

pub(crate) use check::{check, generate};
pub(crate) use cli::run;
