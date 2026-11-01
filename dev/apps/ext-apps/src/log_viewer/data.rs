//! Role: Shared log records and generation.

use crate::Rng;

/// Minutes covered by the sample logs; the timeline has one bucket per 5 minutes.
pub const MINUTES: u16 = 240;
pub const BUCKET_MINUTES: u16 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn tag(self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogLine {
    pub id: u32,
    pub minute: u16,
    pub level: Level,
    pub source: &'static str,
    pub message: String,
}

impl LogLine {
    pub fn clock(&self) -> String {
        clock(self.minute)
    }

    /// Case-insensitive match on source and message (`needle` lower-cased).
    pub fn matches(&self, needle: &str) -> bool {
        needle.is_empty()
            || self.source.contains(needle)
            || self.message.to_lowercase().contains(needle)
    }
}

/// `hh:mm` of a minute offset from 08:00.
pub fn clock(minute: u16) -> String {
    let m = 8 * 60 + minute as u32;
    format!("{:02}:{:02}", m / 60 % 24, m % 60)
}

const SOURCES: [&str; 5] = ["gateway", "auth", "billing", "search", "storage"];
const INFO: [&str; 4] = [
    "request served in {} ms",
    "cache hit ratio {}%",
    "session {} refreshed",
    "job {} finished",
];
const WARN: [&str; 3] = [
    "slow response {} ms",
    "retrying upstream (attempt {})",
    "queue depth {}",
];
const ERROR: [&str; 3] = [
    "timeout after {} ms",
    "connection reset by peer {}",
    "disk quota exceeded on volume {}",
];

/// `n` deterministic lines spread over [`MINUTES`]; incidents (bursts of
/// warnings and errors) sit around `incident` minutes.
pub fn generate(seed: u64, n: usize, incident: u16) -> Vec<LogLine> {
    let mut rng = Rng::new(seed);
    let mut lines: Vec<LogLine> = (0..n)
        .map(|_| {
            let near = rng.below(4) == 0;
            let minute = if near {
                (incident as i64 + rng.below(30) as i64 - 15).clamp(0, MINUTES as i64 - 1) as u16
            } else {
                rng.below(MINUTES as u64) as u16
            };
            let roll = rng.below(100);
            let level = match (near, roll) {
                (true, 0..=39) | (false, 0..=2) => Level::Error,
                (true, 40..=69) | (false, 3..=10) => Level::Warn,
                _ => Level::Info,
            };
            let table: &[&str] = match level {
                Level::Info => &INFO,
                Level::Warn => &WARN,
                Level::Error => &ERROR,
            };
            let tpl = table[rng.below(table.len() as u64) as usize];
            LogLine {
                id: 0,
                minute,
                level,
                source: SOURCES[rng.below(SOURCES.len() as u64) as usize],
                message: tpl.replace(
                    "{}",
                    &(rng.below(if tpl.contains('%') { 100 } else { 2_000 }) + 1).to_string(),
                ),
            }
        })
        .collect();
    lines.sort_by_key(|l| l.minute);
    for (i, l) in lines.iter_mut().enumerate() {
        l.id = i as u32;
    }
    lines
}
