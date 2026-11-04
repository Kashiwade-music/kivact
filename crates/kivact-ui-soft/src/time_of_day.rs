//! Role: Validated wall-clock time independent of any picker surface.

/// A time on a 24-hour clock with second precision.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
}

impl TimeOfDay {
    pub fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        (hour < 24 && minute < 60 && second < 60).then_some(Self {
            hour,
            minute,
            second,
        })
    }

    pub fn hour(self) -> u8 {
        self.hour
    }
    pub fn minute(self) -> u8 {
        self.minute
    }
    pub fn second(self) -> u8 {
        self.second
    }

    pub fn adjust_minutes(self, delta: i32) -> Self {
        let minutes = (i32::from(self.hour) * 60 + i32::from(self.minute) + delta.rem_euclid(1440))
            .rem_euclid(1440);
        Self {
            hour: (minutes / 60) as u8,
            minute: (minutes % 60) as u8,
            second: self.second,
        }
    }
}
