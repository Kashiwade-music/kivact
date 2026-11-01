//! Role: Application-owned bindings for this component view state.
use kivact::{Binding, signal};
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
    #[allow(
        clippy::new_without_default,
        reason = "creation makes signals under the current setup owner; a `Default` impl would hide that"
    )]
    pub fn new() -> Self {
        Self {
            gain: signal(0.0).into(),
            pan: signal(0.0).into(),
            mute: signal(false).into(),
            solo: signal(false).into(),
        }
    }
}
