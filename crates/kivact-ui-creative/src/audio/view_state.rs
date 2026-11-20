//! Role: Application-owned bindings for this component view state.
use kivact::state::Signal;
use kivact::state::signal;
/// The values a [`mixer_strip`](crate::mixer_strip) edits.
#[derive(Clone, Copy)]
pub struct ChannelState {
    /// The fader's gain, inside the strip's gain range.
    pub gain: Signal<f64>,
    /// The pan, from -100 (left) to 100 (right).
    pub pan: Signal<f64>,
    /// Whether the channel is muted.
    pub mute: Signal<bool>,
    /// Whether the channel is soloed.
    pub solo: Signal<bool>,
}

impl ChannelState {
    /// A gain of 0, centered, neither muted nor soloed. Setup only.
    #[track_caller]
    pub fn new() -> Self {
        Self {
            gain: signal(0.0),
            pan: signal(0.0),
            mute: signal(false),
            solo: signal(false),
        }
    }
}

impl Default for ChannelState {
    /// Same as [`ChannelState::new`]. Setup only.
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}
