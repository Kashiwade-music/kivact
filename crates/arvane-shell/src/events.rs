//! Role: Platform-independent input vocabulary; pointer coordinates are logical pixels.

use kivact_types::{FilePhase, Key, KeyCode, Modifiers};

/// Ordered shell events delivered on the UI thread.
#[derive(Clone, Debug, PartialEq)]
pub enum ShellEvent {
    /// CPU framebuffer should be drawn.
    Redraw,
    /// Worker wake or scheduled deadline.
    Wake,
    /// Window content geometry or scale changed.
    Resized,
    /// User requested window closure.
    CloseRequested,
    /// Window activation changed.
    Focused(bool),
    /// The application was suspended: sent to the background (Android,
    /// iOS). Nothing is to be drawn until [`ShellEvent::Resumed`]; the
    /// application stays in memory and may come back. Desktops never
    /// suspend an application (the window starts resumed).
    Suspended,
    /// The suspended application is in the foreground again.
    Resumed,
    /// The window's presentation surface was destroyed (Android destroys it
    /// while the application is in the background): nothing can be drawn
    /// until [`ShellEvent::SurfaceCreated`], and frames are not presented.
    SurfaceDestroyed,
    /// The window has a new presentation surface (the next surface
    /// generation reported by the backend): draw the
    /// window again.
    SurfaceCreated,
    /// The platform is short of memory (Android, iOS): release what can be
    /// made again.
    MemoryWarning,
    /// One mouse, touch or pen event (Pointer Events): the mouse hovers
    /// (`Move`), leaves the window (`Leave`) and presses and releases its
    /// buttons (`Down`, `Up`); a touch or pen contact begins, moves and
    /// ends. Every event carries the pointer's position and held buttons.
    /// A platform's mouse emulation of touch is not reported as the mouse
    /// as well; a pen hovering without contact is not reported. The wheel
    /// is [`ShellEvent::Wheel`].
    Pointer(kivact_types::PointerInput),
    /// Wheel delta in logical pixels (positive scrolls content forward).
    Wheel {
        /// Horizontal logical delta.
        x: f32,
        /// Vertical logical delta.
        y: f32,
    },
    /// Logical key down.
    Key {
        /// Logical key identity (what the keyboard layout makes of it).
        key: Key,
        /// The letter (`a`..=`z`) the physical key has in the US layout
        /// (`KeyA`..=`KeyZ`), `None` for other keys: what a shortcut falls
        /// back to on a layout whose letters are not Latin.
        physical: Option<char>,
        /// Physical keyboard position.
        code: KeyCode,
        /// Whether this is an automatic repeat.
        repeat: bool,
        /// Printable keyboard text, independent of IME commits.
        text: Option<String>,
    },
    /// Logical key release; it never edits text or activates a control.
    KeyUp {
        /// Logical key.
        key: Key,
        /// Physical keyboard position.
        code: KeyCode,
    },
    /// Files dragged from the operating system, with logical position.
    File {
        /// Phase reported by the platform.
        phase: FilePhase,
        /// Paths supplied by the operating system.
        files: Vec<std::path::PathBuf>,
        /// Window logical coordinates.
        position: kivact_types::Point,
    },
    /// Keyboard modifier state changed.
    Modifiers(Modifiers),
    /// IME preedit with UTF-8 byte selection offsets.
    Preedit {
        /// Provisional text.
        text: String,
        /// UTF-8 selection offsets.
        selection: Option<(usize, usize)>,
    },
    /// IME committed text.
    Commit(String),
    /// The IME took this range of the focused text for reconversion, in
    /// UTF-16 code units of the text last given to
    /// the backend as [`crate::ShellImeText`]: it becomes the selection,
    /// which the reconversion's composition then replaces.
    ImeSelection {
        /// Start of the range.
        start: usize,
        /// End of the range.
        end: usize,
    },
    /// IME session ended.
    ImeDisabled,
    /// The window's environment may
    /// have changed: the theme, the reduce-motion preference, the refresh
    /// period (the window moved to another monitor), the safe area, the
    /// mouse settings or the system's locale. The scale changes with [`ShellEvent::Resized`].
    EnvironmentChanged,
    /// Native assistive technology requested an action.
    Accessibility {
        /// Stable semantics identity.
        node: u64,
        /// Requested portable action; a native click arrives as `Activate` and
        /// is resolved against the node's declared actions by the receiver.
        action: kivact_types::A11yAction,
        /// Optional requested text value.
        value: Option<String>,
    },
}
