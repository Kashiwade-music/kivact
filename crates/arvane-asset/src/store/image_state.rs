//! Role: Defines the observable state of a loaded image.

use std::sync::Arc;

use crate::error::AssetError;
use kivact_types::{Repeat, Size};

/// The timing of an animated image: each frame's delay and how often the
/// animation repeats.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationInfo {
    /// Each frame's delay in ms, in frame order.
    pub delays_ms: Arc<[u32]>,
    /// How often the animation repeats.
    pub repeat: Repeat,
}

/// What a store knows about an image.
#[derive(Clone, Debug, PartialEq)]
pub enum ImageState {
    /// Loading; draw nothing, lay out with the node's own size hints.
    Pending,
    /// Loaded; `natural` is the natural size in logical px.
    Ready {
        /// The natural size in logical px.
        natural: Size,
    },
    /// Failed; the error is also reported once by `take_failures`.
    Failed(AssetError),
}
