//! Role: Read responsive breakpoints from the viewport and measured containers.
use kivact::platform::window_size;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::memo;
use kivact::view::NodeRef;

/// Named viewport-width thresholds, measured in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Breakpoint {
    /// Width threshold of 640 logical pixels.
    Sm,
    /// Width threshold of 768 logical pixels.
    Md,
    /// Width threshold of 1024 logical pixels.
    Lg,
    /// Width threshold of 1280 logical pixels.
    Xl,
    /// Width threshold of 1536 logical pixels.
    Xxl,
}

impl Breakpoint {
    /// Return this threshold in logical pixels.
    pub const fn px(self) -> f64 {
        match self {
            Self::Sm => 640.0,
            Self::Md => 768.0,
            Self::Lg => 1024.0,
            Self::Xl => 1280.0,
            Self::Xxl => 1536.0,
        }
    }
}

/// Track whether the viewport width is strictly below the given threshold.
pub fn viewport_below(breakpoint: Breakpoint) -> Value<bool> {
    memo(move || window_size().get().width < breakpoint.px())
}

/// Track whether the viewport width is at or above the given threshold.
pub fn viewport_from(breakpoint: Breakpoint) -> Value<bool> {
    memo(move || window_size().get().width >= breakpoint.px())
}

/// Track whether the viewport is narrower than 768 or shorter than 500 logical pixels.
pub fn is_compact() -> Value<bool> {
    memo(|| {
        let size = window_size().get();
        size.width < Breakpoint::Md.px() || size.height < 500.0
    })
}

/// Track whether a measured node is narrower than a reactive logical-pixel limit.
///
/// Returns `false` while the node has no measured rectangle. Reads both the node's
/// current width and the supplied limit reactively.
pub fn container_below(node: NodeRef, width: impl IntoValue<f64>) -> Value<bool> {
    let rect = node.rect();
    let width = width.into_value();
    memo(move || rect.get().is_some_and(|rect| rect.width < width.get()))
}
