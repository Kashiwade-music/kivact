//! Role: Keyboard focus: tab stops, Tab order, focus requests and automatic moves.
//!
//! Keyboard focus: tab stops and Tab order, focus requests (`Writer::focus`,
//! a press, assistive technology), the blur of a press on nothing
//! focusable ([`Model::press_blurs`]), and the moves the host makes on its own
//! when the focused node goes away or stops being a tab stop.
//!
//! The focus is one `FocusTarget { node, region }`: a scene region is
//! focused through it, with no focus manager of its own, and keys, focus
//! changes and accessibility keep that identity. Whether its indicator
//! shows is kept with it (`Focused::visibility`, so nothing shows without a
//! target): Tab and assistive technology show it, a pointer press hides
//! it, and kivact's request says which.
//!
//! Invariants (the native boundary contract):
//! - A tab stop is focusable by its own props (`Focusable`, neither
//!   disabled nor hidden) and shown: attached, with no hidden ancestor.
//!   Nothing under a hidden ancestor is a tab stop or a focus target, by
//!   any path (`Writer::focus`, Tab, the move into a modal, the move off a
//!   removed or invalid focus).
//! - While a modal overlay is shown, Tab cycles through the tab stops in
//!   it, and a request for a node outside it fails.
//! - Tab with no tab stop leaves the focus alone; static text is never a
//!   tab stop.
//! - Every change of the focus queues one `FocusChanged`; a move the host
//!   makes on its own (or a Tab) is scrolled into view after the next
//!   layout.
//! - A text input that loses the focus is not composing: a composition in
//!   progress is committed as it stands (`TextChanged` before the
//!   `FocusChanged`).

mod navigation;
pub(crate) mod state;
