//! Role: Passive theme measurements of the touch date and time wheels.
/// Measurements of the rows visible around a wheel's selected value.
#[derive(Clone, Debug, PartialEq)]
pub struct DateInputTokens {
    pub wheel_row_height: f64,
    pub wheel_width: f64,
}
impl Default for DateInputTokens {
    fn default() -> Self {
        Self {
            wheel_row_height: 40.0,
            wheel_width: 80.0,
        }
    }
}
