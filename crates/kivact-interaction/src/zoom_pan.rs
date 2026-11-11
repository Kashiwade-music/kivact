//! Role: Shared view transforms and owner-scoped zoom and pan input.
//!
//! [`use_zoom_pan`] edits a caller's [`ViewTransform`] binding and shares
//! one input map: wheel zoom around the pointer, zoom and fit keys, a
//! middle-button or Space-plus-primary pan and a two-finger pinch (the
//! fingers' distance ratio scales, and the content point under their
//! starting midpoint follows the current midpoint). A plain primary press is
//! left to the application, for selection and gizmos. Each wheel notch or
//! key is one edit and each pan or pinch is one edit of an [`EditSession`];
//! the session is used directly rather than through a `DragApi`, since these
//! gestures do not fit one primary-button drag (see `crate::drag`). The
//! gesture is one state ([`ZoomPanApi::is_gesturing`]) ended in one place; the
//! gesture machine also holds Space ([`ZoomPanApi::is_space_armed`]): while not
//! panning, Space presses and releases update the state and reach
//! ancestors, returning `Handled::PreventDefault` so the focused part is not
//! activated; an application that binds Space to play or stop therefore
//! also runs it when a Space-plus-drag pan starts. Releasing Space alone
//! does not end a pan. Touchpad pinch events are not on the public input
//! path yet.
//!
//! The surface part opts into `Part::region_events`, so the unhandled
//! pointer, key and wheel input of a canvas's regions reaches it; a menu's
//! context trigger (`crate::menu`) uses the same opt-in, and a kit does not
//! deliver these events again itself.

mod data;
mod handle;
mod machine;
pub use data::ViewTransform;
pub use handle::{ZoomPanApi, use_zoom_pan};
