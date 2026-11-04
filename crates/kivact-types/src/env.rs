//! Role: Shared env values.
use crate::Size;

/// System preference for a light or dark color scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeHint {
    #[default]
    Light,
    Dark,
}

/// The window's environment as the client sees it: the whole value, at
/// start (`HostInfo::env`) and again whenever any of it changes
/// (`HostEvent::EnvChanged`). The host keeps one and sends a new one only
/// when it differs from the last it sent.
#[derive(Clone, Debug, PartialEq)]
pub struct HostEnv {
    /// Physical pixels per logical pixel.
    pub scale: f32,
    /// The system asks for reduced motion.
    pub reduced_motion: bool,
    /// Light at start; the host follows the system's theme from its first
    /// change.
    pub theme_hint: ThemeHint,
    /// Display refresh period (ns); used for `FrameRate::Display`.
    pub frame_period_ns: u64,
    /// The part of each window edge (logical pixels) system UI may cover
    /// (a notch, a status bar); zero on desktops. The host makes a negative
    /// or non-finite side zero.
    pub safe_area: crate::Insets,
    /// The window's size in logical pixels.
    pub window: Size,
    /// The system's language and region as a BCP 47 tag (`"ja-JP"`), or
    /// `None` where the host cannot tell.
    pub locale: Option<String>,
    /// The window is borderless fullscreen.
    pub fullscreen: bool,
    /// Frames can be drawn: the application is in the foreground and the
    /// window has a surface and is not empty. While it is `false` no frame
    /// ticks are sent.
    pub surface_active: bool,
}

/// What the backend a window runs on can do, reported once, when the
/// window exists: requests it cannot act on and events it never delivers
/// can be diagnosed instead of being ignored.
///
/// Each platform's values are decided by the window that reports them:
/// `arvane-native`'s native window by what winit 0.30 and its Windows
/// subclass deliver on that platform, the `arvane` facade adding the GPU;
/// a test window stands for a platform that can do everything, but the GPU
/// when it is not asked for and native accessibility without a real
/// window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// Touch contacts arrive as touch pointers.
    pub touch: bool,
    /// Pen contacts arrive as pen pointers (not as touch or mouse).
    pub pen: bool,
    /// A request to show or hide the software keyboard is acted on.
    pub soft_keyboard: bool,
    /// The environment's safe area is the platform's (else always zero).
    pub safe_area: bool,
    /// The platform's memory warnings are delivered.
    pub memory_warnings: bool,
    /// The platform IME's reconversion and text queries are answered
    /// from the focused input's text.
    pub ime_reconversion: bool,
    /// The accessibility tree reaches the platform's assistive
    /// technology.
    pub native_accessibility: bool,
    /// Frames can be drawn on the GPU (GPU support is built in; which
    /// renderer draws them is reported as it changes); else on the CPU
    /// only.
    pub gpu: bool,
}

/// An actual application lifecycle transition or platform memory warning.
/// Surface availability and visibility remain current state in the host's environment (`HostEnv`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// Application execution has been suspended by the shell.
    Suspended,
    /// The shell resumed application execution, possibly without a surface yet.
    Resumed,
    /// The platform asks the application to release rebuildable resources.
    MemoryWarning,
}
