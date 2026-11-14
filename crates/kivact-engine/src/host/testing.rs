//! Role: Drive production hosts through the stepped virtual backend.

mod display_probe;
#[cfg(debug_assertions)]
mod self_check;
mod test_frame;
mod test_host;
mod test_options;
#[cfg(debug_assertions)]
mod text_reference;

pub use arvane::{ShellEvent, ShellImeText};
pub use arvane_native::mirror::MirrorStats;
pub use arvane_native::native_window_alive;
pub use arvane_virtual::raw::{PlatformKey, RawDrop, RawIme, RawInput, RawKey, RawKeyPhase};
pub use arvane_virtual::{GpuMode, MirrorMode};
pub use kivact_types::render::{SemanticsNode, SemanticsTree};
pub use kivact_types::{
    A11yAction, A11yRequest, ButtonPress, CheckedState, Cursor, DismissReason, FilePhase, Insets,
    Key, KeyCode, LiveRegion, Modifiers, MouseButton, Orientation, Point, PointerId, PointerKind,
    PointerPhase, Role, SortDirection,
};
pub use kivact_types::{PointerSample, Semantics, Transform};

pub use display_probe::{display_stats, gpu_unavailable};
pub use test_frame::TestFrame;
pub use test_host::{ArvaneTestHost, Renderer};
pub use test_options::{TestFonts, TestHostOptions, default_font};

#[cfg(any(windows, target_os = "linux"))]
pub use arvane_native::embed::TestParent;
