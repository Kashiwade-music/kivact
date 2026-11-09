//! Role: Application-owned bindings for this component view state.
use kivact::state::IntoBinding;
use kivact::state::{Binding, signal};
/// The values a [`mixer_strip`](crate::mixer_strip) edits.
#[derive(Clone, Copy)]
pub struct ChannelState {
    /// The fader's gain, inside the strip's gain range.
    pub gain: Binding<f64>,
    /// The pan, from -100 (left) to 100 (right).
    pub pan: Binding<f64>,
    /// Whether the channel is muted.
    pub mute: Binding<bool>,
    /// Whether the channel is soloed.
    pub solo: Binding<bool>,
}

impl ChannelState {
    /// A gain of 0, centered, neither muted nor soloed. Setup only.
    #[track_caller]
    pub fn new() -> Self {
        Self {
            gain: signal(0.0).into_binding(),
            pan: signal(0.0).into_binding(),
            mute: signal(false).into_binding(),
            solo: signal(false).into_binding(),
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
