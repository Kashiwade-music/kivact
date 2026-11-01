//! Role: Shared device sample data and generation.

use kivact::prelude::ColorToken;

use crate::Rng;

/// Samples kept per device (one per tick).
pub const HISTORY: usize = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warning,
    Alarm,
    Offline,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::Warning => "WARN",
            Status::Alarm => "ALARM",
            Status::Offline => "OFFLINE",
        }
    }

    pub fn token(self) -> ColorToken {
        match self {
            Status::Ok => ColorToken::Success,
            Status::Warning => ColorToken::Warning,
            Status::Alarm => ColorToken::Danger,
            Status::Offline => ColorToken::TextMuted,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub id: u32,
    pub name: String,
    pub zone: &'static str,
    pub status: Status,
    /// °C, newest last.
    pub temp: Vec<f32>,
    /// V, newest last.
    pub volt: Vec<f32>,
}

impl Device {
    pub fn temp_now(&self) -> f32 {
        self.temp.last().copied().unwrap_or(0.0)
    }

    pub fn volt_now(&self) -> f32 {
        self.volt.last().copied().unwrap_or(0.0)
    }

    fn classify(&mut self) {
        let t = self.temp_now();
        let v = self.volt_now();
        self.status = if v < 1.0 {
            Status::Offline
        } else if t > 78.0 || v < 10.8 {
            Status::Alarm
        } else if t > 65.0 || v < 11.4 {
            Status::Warning
        } else {
            Status::Ok
        };
    }
}

const ZONES: [&str; 4] = ["North hall", "South hall", "Roof", "Basement"];

/// `n` devices with a full history.
pub fn fleet(seed: u64, n: usize) -> Vec<Device> {
    let mut rng = Rng::new(seed);
    (0..n)
        .map(|i| {
            let hot = rng.below(12) == 0;
            let base_t = if hot { 70.0 } else { 35.0 + rng.unit() * 20.0 };
            let dead = rng.below(40) == 0;
            let mut d = Device {
                id: i as u32,
                name: format!("sensor-{:04}", i),
                zone: ZONES[i % ZONES.len()],
                status: Status::Ok,
                temp: Vec::with_capacity(HISTORY),
                volt: Vec::with_capacity(HISTORY),
            };
            for k in 0..HISTORY {
                let wave = ((k as f32 + i as f32) * 0.35).sin() * 4.0;
                d.temp.push(base_t + wave + rng.unit() * 6.0);
                d.volt
                    .push(if dead { 0.0 } else { 11.2 + rng.unit() * 1.2 });
            }
            d.classify();
            d
        })
        .collect()
}

/// One live tick: every device gets a new sample.
pub fn tick(devices: &mut [Device], rng: &mut Rng) {
    for d in devices {
        let last_t = d.temp_now();
        let drift = (rng.unit() - 0.5) * 4.0;
        let t = (last_t + drift).clamp(20.0, 95.0);
        let v = if d.volt_now() < 1.0 {
            0.0
        } else {
            (11.2 + rng.unit() * 1.2).max(0.0)
        };
        for (hist, x) in [(&mut d.temp, t), (&mut d.volt, v)] {
            if hist.len() >= HISTORY {
                hist.remove(0);
            }
            hist.push(x);
        }
        d.classify();
    }
}
