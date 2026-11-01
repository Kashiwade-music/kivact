//! Role: Benchmark result reporting.
//!
//! Measurement plumbing: the wall clock, phase marks on stdout, the host's
//! frame log (`KIVACT_FRAME_LOG`), percentiles and the one-line JSON
//! result.
//!
//! Every time here is the wall clock (`SystemTime`, ns since the Unix
//! epoch), the clock the hosts stamp their frame log lines with, so a
//! change the app makes can be matched with the frame that showed it.

use std::fmt::Write as _;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Wall clock in ns since the Unix epoch.
pub(crate) fn unix_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

static MAIN: OnceLock<u128> = OnceLock::new();

/// Remember when `main` started (the first call wins).
pub(crate) fn start_clock() {
    MAIN.get_or_init(unix_ns);
}

/// When `main` started.
pub(crate) fn main_ns() -> u128 {
    *MAIN.get_or_init(unix_ns)
}

/// The host configuration measured: `arvane` (the GPU, started with the
/// window), or `arvane-cpu` when the host is told to draw on the CPU
/// (`ARVANE_RENDERER=cpu`).
pub(crate) fn host_name() -> &'static str {
    match std::env::var("ARVANE_RENDERER").as_deref() {
        Ok("cpu") => "arvane-cpu",
        _ => "arvane",
    }
}

/// A phase mark for the runner, which samples the process (CPU time,
/// memory) when it reads the line: `MARK <name> <unix_ns>`.
pub(crate) fn mark(name: &str) -> u128 {
    let now = unix_ns();
    println!("MARK {name} {now}");
    now
}

/// One presented frame from the host's log.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Frame {
    /// Wall clock when the frame was handed to the platform.
    pub(crate) end: u128,
    /// Host UI-thread time of the frame (ns).
    pub(crate) work: u64,
    pub(crate) raster: Option<u64>,
    pub(crate) present: Option<u64>,
}

/// What the host's frame log holds so far.
#[derive(Debug, Default)]
pub(crate) struct FrameLog {
    pub(crate) frames: Vec<Frame>,
    /// The last `renderer=` note.
    pub(crate) renderer: Option<String>,
    /// Whether the host noted `accessibility=on`: a UI Automation client
    /// is attached (the host then keeps its accessibility tree up to date).
    pub(crate) accessibility: bool,
    /// Whether `KIVACT_FRAME_LOG` was set.
    pub(crate) enabled: bool,
}

/// Read the host's frame log (empty when `KIVACT_FRAME_LOG` is unset).
pub(crate) fn frame_log() -> FrameLog {
    let Some(path) = std::env::var_os("KIVACT_FRAME_LOG").filter(|p| !p.is_empty()) else {
        return FrameLog::default();
    };
    let mut log = FrameLog {
        enabled: true,
        ..FrameLog::default()
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return log;
    };
    for line in text.lines() {
        if let Some(note) = line.strip_prefix("# ") {
            if let Some(r) = note
                .split_whitespace()
                .find_map(|w| w.strip_prefix("renderer="))
            {
                log.renderer = Some(r.to_owned());
            }
            if note.split_whitespace().any(|w| w == "accessibility=on") {
                log.accessibility = true;
            }
            continue;
        }
        let mut parts = line.split_whitespace();
        if parts.next() != Some("F") {
            continue;
        }
        let mut next = || parts.next().and_then(|p| p.parse::<u128>().ok());
        let (Some(end), Some(work)) = (next(), next()) else {
            // A line still being written.
            continue;
        };
        let _scene = next();
        let raster = next().map(|v| v as u64);
        let present = next().map(|v| v as u64);
        log.frames.push(Frame {
            end,
            work: work as u64,
            raster,
            present,
        });
    }
    log
}

impl FrameLog {
    /// Frames handed to the platform in `[from, to]`.
    pub(crate) fn between(&self, from: u128, to: u128) -> Vec<Frame> {
        self.frames
            .iter()
            .filter(|f| f.end >= from && f.end <= to)
            .copied()
            .collect()
    }

    /// The first frame handed to the platform after `t`.
    pub(crate) fn first_after(&self, t: u128) -> Option<Frame> {
        self.frames.iter().find(|f| f.end > t).copied()
    }
}

/// Percentiles of a sample (nearest rank).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Stats {
    pub(crate) p50: f64,
    pub(crate) p95: f64,
    pub(crate) p99: f64,
    pub(crate) max: f64,
    pub(crate) mean: f64,
}

pub(crate) fn stats(mut values: Vec<f64>) -> Option<Stats> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let at = |q: f64| {
        let rank = (q * values.len() as f64).ceil() as usize;
        values[rank.clamp(1, values.len()) - 1]
    };
    Some(Stats {
        p50: at(0.50),
        p95: at(0.95),
        p99: at(0.99),
        max: values[values.len() - 1],
        mean: values.iter().sum::<f64>() / values.len() as f64,
    })
}

pub(crate) fn ms(ns: u128) -> f64 {
    ns as f64 / 1e6
}

/// A flat JSON object, written by hand (the bench has no dependencies).
pub(crate) struct Json(String);

impl Json {
    /// `{"host":..,"renderer":..,"scenario":..`.
    pub(crate) fn new(scenario: &str) -> Json {
        let log = frame_log();
        let renderer = log.renderer.unwrap_or_else(|| "unknown".into());
        let mut j = Json(String::from("{"));
        j.str("host", host_name());
        j.str("renderer", &renderer);
        j.str(
            "accessibility",
            if log.accessibility { "on" } else { "off" },
        );
        j.str("scenario", scenario);
        j
    }

    fn key(&mut self, key: &str) {
        if self.0.len() > 1 {
            self.0.push(',');
        }
        let _ = write!(self.0, "\"{key}\":");
    }

    pub(crate) fn str(&mut self, key: &str, value: &str) -> &mut Self {
        self.key(key);
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        let _ = write!(self.0, "\"{escaped}\"");
        self
    }

    pub(crate) fn num(&mut self, key: &str, value: f64) -> &mut Self {
        self.key(key);
        if value.is_finite() {
            let _ = write!(self.0, "{:.3}", value);
        } else {
            self.0.push_str("null");
        }
        self
    }

    pub(crate) fn int(&mut self, key: &str, value: u128) -> &mut Self {
        self.key(key);
        let _ = write!(self.0, "{value}");
        self
    }

    /// `"key":{"p50":..,"p95":..,"p99":..,"max":..,"mean":..}` or `null`.
    pub(crate) fn stats(&mut self, key: &str, s: Option<Stats>) -> &mut Self {
        self.key(key);
        match s {
            Some(s) => {
                let _ = write!(
                    self.0,
                    "{{\"p50\":{:.3},\"p95\":{:.3},\"p99\":{:.3},\"max\":{:.3},\"mean\":{:.3}}}",
                    s.p50, s.p95, s.p99, s.max, s.mean
                );
            }
            None => self.0.push_str("null"),
        }
        self
    }

    /// The presentation of the frames in `[from, to]`: count, rate,
    /// intervals between frames and the host's per-frame times (ms).
    pub(crate) fn frames(&mut self, from: u128, to: u128) -> &mut Self {
        let log = frame_log();
        let frames = log.between(from, to);
        let intervals = frames.windows(2).map(|w| ms(w[1].end - w[0].end)).collect();
        let fps = match (frames.first(), frames.last()) {
            (Some(a), Some(b)) if b.end > a.end => {
                (frames.len() - 1) as f64 / (ms(b.end - a.end) / 1e3)
            }
            _ => 0.0,
        };
        self.int("frames", frames.len() as u128);
        self.num("fps", fps);
        self.num("phase_ms", ms(to.saturating_sub(from)));
        self.stats("interval_ms", stats(intervals));
        self.stats(
            "work_ms",
            stats(frames.iter().map(|f| f.work as f64 / 1e6).collect()),
        );
        let part = |pick: fn(&Frame) -> Option<u64>| {
            stats(
                frames
                    .iter()
                    .filter_map(|f| pick(f).map(|v| v as f64 / 1e6))
                    .collect(),
            )
        };
        self.stats("raster_ms", part(|f| f.raster));
        self.stats("present_ms", part(|f| f.present));
        self
    }

    /// Print the result line and the `end` mark, then exit after a short
    /// pause (the runner samples the process memory at the mark).
    pub(crate) fn finish(&mut self) {
        self.0.push('}');
        println!("RESULT {}", self.0);
        mark("end");
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(400));
            std::process::exit(0);
        });
    }
}
