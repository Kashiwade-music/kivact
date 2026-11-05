//! Role: Describe the observable loading state of an image asset.

/// The current loading result of an image node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageStatus {
    /// The image source has not finished loading.
    #[default]
    Loading,
    /// Decoded image pixels are available.
    Ready,
    /// The current image source failed to load.
    Failed,
}
