//! Role: Platform-independent input vocabulary; pointer coordinates are logical pixels.

use kivact_types::{FileDrop, ImeInput, KeyInput, Modifiers, PointerSample, WheelInput};

/// Ordered shell events delivered on the UI thread.
#[derive(Clone, Debug, PartialEq)]
pub enum ShellEvent {
    /// Software keyboard occlusion in logical pixels.
    KeyboardInset(kivact_types::Insets),
    /// Ask the application to preserve state for its next launch.
    SaveState,
    /// CPU framebuffer should be drawn.
    Redraw,
    /// Worker wake or scheduled deadline.
    Wake(crate::types::WakeReasons),
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
    Pointer(PointerSample),
    /// A normalized wheel movement in logical window coordinates.
    Wheel(WheelInput),
    /// A normalized key press or release.
    Key(KeyInput),
    /// Files dragged from the operating system, with their window position.
    File(FileDrop),
    /// Keyboard modifier state changed.
    Modifiers(Modifiers),
    /// IME text and selection operations, with UTF-8 byte offsets.
    Ime(ImeInput),
    /// The window's environment may
    /// have changed: the theme, the reduce-motion preference, the refresh
    /// period (the window moved to another monitor), the safe area, the
    /// mouse settings or the system's locale. The scale changes with [`ShellEvent::Resized`].
    EnvironmentChanged,
    /// Assistive technology activated this window's accessibility tree.
    AccessibilityActivated,
    /// Assistive technology deactivated this window's accessibility tree.
    AccessibilityDeactivated,
    /// Native assistive technology requested an action.
    Accessibility {
        /// Stable semantics identity.
        node: u64,
        /// The request; a native click arrives as `Activate` and is resolved
        /// against the node's declared actions by the receiver.
        request: kivact_types::A11yRequest,
    },
}
