//! Role: Portable shell vocabulary and input normalization for Arvane backends.
//!
//! Backends deliver ordered [`ShellEvent`] values on their UI thread, using
//! logical window pixels for input coordinates and nanoseconds for deadlines.
//! [`ShellOptions`] configures a window; [`ShellControl`] chooses its next wake
//! and [`ShellWaker`] lets workers notify its event loop. [`ShellError`] separates
//! unsupported platform services from failures of an available service.
//!
//! The mouse, touch and pen arrive as one [`ShellEvent::Pointer`]. [`Mouse`]
//! makes complete mouse events (position and held buttons) from a backend's
//! separate messages. [`Contacts`] gives platform contacts stable pointer
//! identities and balances their beginnings and endings. [`ClickCounter`] fills click counts using the
//! backend's clock and double-click settings. Native event decoding stays in the
//! backend; native and injected input use these same normalization rules.
//!
//! [`ShellImeText`] supplies surrounding text in UTF-16 offsets and logical
//! character bounds. [`ShellFileDialog`] carries dialog options without opening
//! a dialog. [`DEFAULT_FRAME_PERIOD_NS`] is the fallback for unknown refresh
//! rates. [`Accessibility`] validates semantics transactionally. [`AcceptedTree`]
//! gives backends read access; [`PlatformTree`] tracks activation, pending updates
//! and accepted actions. [`PlatformUpdate`] is converted under its lock so native
//! readers never receive older node data after a newer activation snapshot.
//! [`ShellChild`] runs a child window in its host event loop.
//! The object-safe window and event-loop SPI has no OS implementation.
//! Backends implement its services; the facade retains rendering policy.
#![forbid(unsafe_code)]

mod a11y;
mod child;
mod clicks;
mod contacts;
mod events;
mod ime;
mod mouse;
mod services;
mod target;
mod types;
mod window;

pub use a11y::Accessibility;
pub use a11y::error::AccessibilityError;
pub use a11y::shared::{PlatformTree, PlatformUpdate};
pub use a11y::state::AcceptedTree;
pub use clicks::{ClickCounter, DEFAULT_DOUBLE_CLICK};
pub use contacts::Contacts;
pub use events::ShellEvent;
pub use ime::ShellImeText;
pub use kivact_types::FilePhase;
pub use mouse::Mouse;
pub use services::ShellFileDialog;
pub use types::{ShellControl, ShellError, ShellOptions, ShellWaker};

/// The frame period of a display whose refresh rate is unknown (60 Hz).
pub const DEFAULT_FRAME_PERIOD_NS: u64 = 16_666_667;

pub use target::{GpuTarget, GpuWindow};
pub use window::{ShellEventLoop, ShellHandler, ShellWindow};

pub use child::{ChildOptions, MakeHandler, PanicSlot, ShellChild};
