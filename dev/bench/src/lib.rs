//! Role: Shared helpers of the KPI probe binaries (timestamps, env options, percentiles).
//!
//! The binaries print `KEY value` lines on stderr; `dev/bench/run.sh` collects
//! them into one Markdown table. Nothing here is used by Kivact itself.

#![allow(
    clippy::print_stderr,
    reason = "the probes report their measurements on the console"
)]

use std::time::{SystemTime, UNIX_EPOCH};

/// Wall-clock nanoseconds since the Unix epoch (comparable with `KIVACT_FRAME_LOG`).
pub fn unix_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
}

/// Print one timestamp line: `T <tag> <unix_ns>`.
pub fn stamp(tag: &str) {
    eprintln!("T {tag} {}", unix_ns());
}

/// The environment variable `name` parsed as `T`, else `default`.
pub fn env_or<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// The `q` quantile (0..=1) of `samples` (sorted in place); 0 for none.
pub fn quantile(samples: &mut [f64], q: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.sort_by(f64::total_cmp);
    let index = ((samples.len() as f64) * q).floor() as usize;
    samples[index.min(samples.len() - 1)]
}

/// Print `name n=.. mean=.. p50=.. p95=.. p99=..` (milliseconds) on stderr.
pub fn report(name: &str, mut samples: Vec<f64>) {
    let n = samples.len().max(1) as f64;
    let mean = samples.iter().sum::<f64>() / n;
    let p50 = quantile(&mut samples, 0.5);
    let p95 = quantile(&mut samples, 0.95);
    let p99 = quantile(&mut samples, 0.99);
    eprintln!(
        "L {name} n={} mean={mean:.3} p50={p50:.3} p95={p95:.3} p99={p99:.3}",
        samples.len()
    );
}
