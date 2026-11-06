//! Role: Host environment, capabilities and lifecycle events.
use crate::cell::Generation;
use crate::defaults::{DEFAULT_WINDOW_SIZE, DOUBLE_CLICK, FRAME_PERIOD_NS};
use crate::{Insets, Size};

/// Whether logical window dimensions are finite, at least one logical pixel,
/// and fit a native unsigned extent. Backends also validate the scaled size.
pub fn is_valid_window_size(size: Size) -> bool {
    [size.width, size.height]
        .into_iter()
        .all(|value| value.is_finite() && value >= 1.0 && value <= f64::from(u32::MAX))
}

/// System preference for a light or dark color scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeHint {
    #[default]
    Light,
    Dark,
}

/// Current window and platform surroundings, shared unchanged across layers.
#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    /// Read-only state supplied by the platform at launch; saving does not mutate this snapshot.
    pub restored_state: std::collections::BTreeMap<String, Vec<u8>>,
    /// Physical pixels per logical pixel.
    pub scale_factor: f64,
    /// The system asks for reduced motion.
    pub reduced_motion: bool,
    /// The system's current app color mode; light when it cannot be queried.
    pub theme_hint: ThemeHint,
    /// Display refresh period (ns); used for `FrameRate::Display`.
    pub frame_period_ns: u64,
    /// The part of each window edge (logical pixels) system UI may cover
    /// (a notch, a status bar); zero on desktops. The host makes a negative
    /// or non-finite side zero.
    pub safe_area: Insets,
    /// Insets occupied by the software keyboard, in logical pixels.
    pub keyboard_inset: Insets,
    /// The window's size in logical pixels.
    pub window_size: Size,
    /// The system's language and region as a BCP 47 tag (`"ja-JP"`), or
    /// `None` where the host cannot tell.
    pub locale: Option<String>,
    /// The window is borderless fullscreen.
    /// The window can enter fullscreen.
    pub fullscreen: bool,
    /// System pointer preferences, expressed in logical pixels and nanoseconds.
    pub pointer: PointerSettings,
    /// Generation of the available font collection.
    pub fonts: Generation,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            restored_state: std::collections::BTreeMap::new(),
            scale_factor: 1.0,
            reduced_motion: false,
            theme_hint: ThemeHint::default(),
            frame_period_ns: FRAME_PERIOD_NS,
            safe_area: Insets::default(),
            keyboard_inset: Insets::default(),
            window_size: DEFAULT_WINDOW_SIZE,
            locale: None,
            fullscreen: false,
            pointer: PointerSettings::default(),
            fonts: Generation::default(),
        }
    }
}

/// Precision of the platform's primary pointing device.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PointerClass {
    /// A mouse, trackpad or similarly precise pointing device.
    #[default]
    Fine,
    /// A primary device intended for direct, less precise contact.
    Coarse,
}

/// Platform pointer preferences; positions and distances are logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerSettings {
    /// Maximum interval between presses counted as one double click, in nanoseconds.
    pub double_click_ns: u64,
    /// Maximum double-click displacement on each axis, in logical pixels.
    pub double_click_distance: (f64, f64),
    /// Lines scrolled by one vertical wheel notch.
    pub wheel_lines: u32,
    /// Characters scrolled by one horizontal wheel notch.
    pub wheel_chars: u32,
    /// Default movement threshold for new drag interactions, in logical pixels.
    pub drag_slop: f64,
    /// Precision of the primary pointing device.
    pub primary: PointerClass,
    /// Fraction of fling velocity retained per millisecond.
    pub fling_deceleration: f64,
    /// Fling stop threshold, in logical pixels per millisecond.
    pub fling_min_velocity: f64,
}

impl Default for PointerSettings {
    fn default() -> Self {
        Self {
            double_click_ns: DOUBLE_CLICK.0,
            double_click_distance: DOUBLE_CLICK.1,
            wheel_lines: 3,
            wheel_chars: 3,
            drag_slop: 4.0,
            primary: PointerClass::Fine,
            fling_deceleration: 0.998,
            fling_min_velocity: 0.05,
        }
    }
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
    /// Application state can be saved for the next launch.
    pub state_restoration: bool,
    /// The platform can read and write clipboard text.
    pub clipboard_text: bool,
    /// The platform can show file and folder dialogs.
    pub file_dialogs: bool,
    /// The platform can open files and folders.
    pub open_path: bool,
    /// The platform can reveal paths in its file manager.
    pub reveal_path: bool,
    pub fullscreen: bool,
    /// The window can enforce a minimum content size.
    pub min_size: bool,
    /// The modifier used for platform editing commands.
    pub command_key: CommandKey,
    /// The pointer transition used to open context menus.
    pub context_menu_on: ContextMenuTrigger,
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
/// Surface availability belongs to the host's lifecycle state, outside [`Environment`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// The platform asks the application to save state for its next launch.
    SaveState,
    /// Application execution has been suspended by the shell.
    Suspended,
    /// The shell resumed application execution, possibly without a surface yet.
    Resumed,
    /// The platform asks the application to release rebuildable resources.
    MemoryWarning,
}

/// Key used for platform editing commands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CommandKey {
    #[default]
    Ctrl,
    Command,
}

/// Pointer transition that opens a context menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContextMenuTrigger {
    Press,
    #[default]
    Release,
}
