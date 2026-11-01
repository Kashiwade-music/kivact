//! Role: How an image fits its box (CSS object-fit).

/// How an image fills its box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageFit {
    /// Scale to fit inside the box, keeping the aspect ratio.
    #[default]
    Contain,
    /// Scale to cover the box, keeping the aspect ratio.
    Cover,
    /// Stretch to the box.
    Fill,
    /// Natural size.
    None,
    /// `None` when the natural size fits the box, else `Contain`.
    ScaleDown,
}
