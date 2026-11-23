//! Role: Compare two run.sh KPI reports row by row under dev/bench/kpi.toml.

mod cli;
mod config;
mod judge;
mod report;
#[cfg(test)]
mod tests;

pub(crate) use cli::run;
