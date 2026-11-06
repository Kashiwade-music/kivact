//! Role: Calendar measurements that kits can override in the theme.
/// Theme measurements of calendar dates and panels.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarTokens {
    pub cell_size: f64,
    pub month_gap: f64,
    pub week_gap: f64,
}
impl Default for CalendarTokens {
    fn default() -> Self {
        Self {
            cell_size: 32.0,
            month_gap: 16.0,
            week_gap: 8.0,
        }
    }
}

/// Passive drawing projection for one calendar event mark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarEventVisual {
    pub color: kivact_core::style::Color,
    pub selected: bool,
}
