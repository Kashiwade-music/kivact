//! Role: Easing curves and spring parameters of animations.

/// Where a discrete easing step jumps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPosition {
    Start,
    End,
}

/// Mapping from normalized elapsed time to animation progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// CSS-style cubic bezier (x1, y1, x2, y2).
    CubicBezier(f32, f32, f32, f32),
    /// Discrete steps. A zero count is treated as one.
    Steps {
        count: u32,
        position: StepPosition,
    },
}

impl Easing {
    /// `count` discrete steps that jump at the start of each step.
    pub fn steps_start(count: u32) -> Self {
        Easing::Steps {
            count,
            position: StepPosition::Start,
        }
    }

    /// `count` discrete steps that jump at the end of each step.
    pub fn steps_end(count: u32) -> Self {
        Easing::Steps {
            count,
            position: StepPosition::End,
        }
    }

    /// Evaluate progress after clamping normalized time to `0..=1`.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::CubicBezier(x1, y1, x2, y2) => bezier(t, x1, y1, x2, y2),
            Easing::Steps { count, position } => {
                let n = count.max(1) as f32;
                match position {
                    StepPosition::Start => ((t * n).floor() + 1.0).min(n) / n,
                    StepPosition::End => (t * n).floor() / n,
                }
            }
        }
    }
}

fn bezier(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let curve = |t: f32, a: f32, b: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    };
    // Solve x(t) = x by bisection (monotonic for valid curves).
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        if curve(mid, x1, x2) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    curve((lo + hi) / 2.0, y1, y2)
}

/// Spring parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Spring force per unit displacement.
    pub stiffness: f32,
    /// Resistance proportional to velocity.
    pub damping: f32,
    /// Mass used when integrating spring acceleration.
    pub mass: f32,
}

impl Spring {
    pub const GENTLE: Spring = Spring {
        stiffness: 120.0,
        damping: 14.0,
        mass: 1.0,
    };
    pub const WOBBLY: Spring = Spring {
        stiffness: 180.0,
        damping: 12.0,
        mass: 1.0,
    };
    pub const STIFF: Spring = Spring {
        stiffness: 210.0,
        damping: 20.0,
        mass: 1.0,
    };
    pub const DEFAULT: Spring = Spring {
        stiffness: 170.0,
        damping: 26.0,
        mass: 1.0,
    };
}
