//! Role: Shared logical geometry for views and drawing.

use std::panic::Location;

pub use kivact_types::{Edge, Insets, Matrix4, Point, Rect, Size};

use crate::Result;
use kivact_runtime::{ErrorKind, UiError};

/// Refuse a window size or minimum size that cannot form a drawable extent,
/// and a minimum larger than the size, as an error located at `at`; `names`
/// are the APIs to name (`<size api>`, `<min size api>`). The platform would
/// otherwise clamp or ignore it silently.
pub(crate) fn check_window_sizes(
    window: Option<Size>,
    min: Option<Size>,
    names: (&'static str, &'static str),
    at: &'static Location<'static>,
) -> Result<()> {
    let (size_api, min_api) = names;
    let fail = |message: String| Err(UiError::new(ErrorKind::Message, message, Some(at)));
    for (name, size) in [("window_size", window), ("min_window_size", min)] {
        if let Some(s) = size
            && !kivact_types::env::is_valid_window_size(s)
        {
            let api = if name == "window_size" {
                size_api
            } else {
                min_api
            };
            return fail(format!(
                "{api}: {} by {} must be finite, at least one logical pixel and fit the native window extent",
                s.width, s.height
            ));
        }
    }
    if let (Some(window), Some(min)) = (window, min)
        && (min.width > window.width || min.height > window.height)
    {
        return fail(format!(
            "{min_api}: {} by {} is larger than {size_api} {} by {}",
            min.width, min.height, window.width, window.height
        ));
    }
    Ok(())
}
