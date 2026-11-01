//! Role: Defines the observable state of a loaded image.

use crate::error::AssetError;
use ui_types::Size;

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
