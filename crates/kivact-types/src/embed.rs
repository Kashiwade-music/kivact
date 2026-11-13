//! Role: Pure options and parent identities for embedded windows.

use crate::defaults::DEFAULT_WINDOW_SIZE;
use crate::{GpuPreference, Size};

/// Whether an embedded window is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Visibility {
    /// The window can draw; the default, and what `App::embed` opens.
    #[default]
    Shown,
    /// The window has no surface until `Embedded::show`: nothing is drawn and
    /// frame loops do not tick, while tasks, timers and signals keep running.
    Hidden,
}

/// What the host did with a size the editor asked for
/// (`Parent::on_resize_request`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeAnswer {
    /// The host accepted now. Kivact resizes the window itself; a later
    /// `Embedded::set_size` still wins. The request resolves at once.
    Accepted,
    /// The host decides later and confirms with `Embedded::set_size`. The
    /// window keeps its size until then; the request resolves when the next
    /// host size arrives, is cancelled by a newer request, or times out.
    Pending,
    /// The host refused; the request fails with a platform error.
    Refused,
}

/// Whether key presses the editor does not handle go back to the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum UnhandledKeys {
    /// The editor keeps them.
    #[default]
    Keep,
    /// Give them, and their releases, back to the parent window
    /// (`Parent::forward_unhandled_keys`).
    ToParent,
}

/// Initial embedded-window values; callbacks belong to the host's hooks.
#[derive(Clone, Debug, PartialEq)]
pub struct EmbedOptions {
    /// Initial dimensions in logical pixels.
    pub size: Size,
    /// Physical pixels per logical pixel, or the operating system's scale.
    pub scale: Option<f64>,
    /// Whether the window opens shown.
    pub visibility: Visibility,
    /// What happens to key presses the editor does not handle.
    pub unhandled_keys: UnhandledKeys,
    /// Prefer GPU rendering when the platform supports it.
    pub prefer_gpu: bool,
    pub gpu_preference: GpuPreference,
}

impl Default for EmbedOptions {
    fn default() -> Self {
        Self {
            size: DEFAULT_WINDOW_SIZE,
            scale: None,
            visibility: Visibility::default(),
            unhandled_keys: UnhandledKeys::default(),
            prefer_gpu: true,
            gpu_preference: GpuPreference::default(),
        }
    }
}
