//! Role: The input a tool asks a loop to deliver, before normalization.

/// Input a tool (the DevTools agent) asks the loop to deliver, before the
/// platform-independent normalization.
#[derive(Clone, Debug, PartialEq)]
pub enum InjectedInput {
    /// Move the mouse to a logical window position.
    MouseMove(kivact_types::Point),
    /// Press a mouse button where the mouse is.
    MouseDown(kivact_types::MouseButton),
    /// Release a mouse button where the mouse is.
    MouseUp(kivact_types::MouseButton),
    /// Wheel lines at the current mouse position.
    Wheel { lines_x: f64, lines_y: f64 },
    /// Set the held modifier keys (delivered as `ShellEvent::Modifiers`, the
    /// way both backends report modifiers today). `kivact_types::Key` has no
    /// modifier keys, so an injected chord is: Modifiers(set), key down/up,
    /// Modifiers(empty).
    Modifiers(kivact_types::Modifiers),
    /// One raw key, IME or file-drop input (the `pipeline::RawInput` vocabulary).
    Raw(crate::pipeline::RawInput),
    /// Leave the window with the mouse.
    MouseLeave,
    /// An assistive-technology request for one node.
    Accessibility {
        /// Stable semantics identity.
        node: u64,
        /// The request to resolve against the node's declared actions.
        request: kivact_types::A11yRequest,
    },
}
