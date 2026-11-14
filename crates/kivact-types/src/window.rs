//! Role: Window identities, native handles and the request to open one.

use crate::Size;
use std::num::NonZeroIsize;
use std::num::NonZeroU32;
use std::num::NonZeroU64;

/// The identity of one window of an App. The App's first window is `MAIN`;
/// the core numbers further windows 2, 3, ... and never reuses a number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(NonZeroU64);

impl WindowId {
    /// The App's first window.
    pub const MAIN: WindowId = WindowId(NonZeroU64::MIN);

    /// The id with number `raw`, or `None` for 0.
    pub fn new(raw: u64) -> Option<WindowId> {
        NonZeroU64::new(raw).map(WindowId)
    }

    /// The number of this id.
    pub fn get(self) -> u64 {
        self.0.get()
    }
}

/// What kind of window a window is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WindowMode {
    /// An ordinary window with its own taskbar entry.
    #[default]
    Normal,
    /// A utility window that stays above its parent and has no taskbar entry.
    Tool,
    /// A window that blocks input to its parent while it is open.
    Modal,
}

/// Where a window opens, relative to its parent window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WindowPlacement {
    /// Where the platform puts it (the main window: centered on the primary
    /// monitor's work area, as today).
    #[default]
    Auto,
    /// Centered over the parent window.
    CenterOnParent,
    /// Next to the parent window: to its right, or to its left when the right
    /// side has no room, or `Auto` when neither has room.
    NextToParent,
}

/// Where a window's content comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WindowContent {
    /// Kivact draws the window's view tree.
    #[default]
    Drawn,
    /// The whole content area is one native child window for UI another
    /// library draws, such as a plugin editor.
    Foreign,
}

/// A raw native window value; this type never dereferences it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeWindow {
    /// A Win32 `HWND`.
    Win32(NonZeroIsize),
    /// An X11 window id.
    X11(NonZeroU32),
}

/// The measured logical size, physical rendering extent and scale of a
/// surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSize {
    pub logical: Size,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
}

/// A request to open one window.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowRequest {
    pub id: WindowId,
    pub title: String,
    /// Logical size.
    pub size: Size,
    pub min_size: Option<Size>,
    pub mode: WindowMode,
    pub placement: WindowPlacement,
    /// The window the request was declared in; `None` only for the main window.
    pub parent: Option<WindowId>,
    /// Where the content comes from.
    pub content: WindowContent,
}

impl WindowRequest {
    /// The request for the App's first window: `WindowId::MAIN`, a `Normal`
    /// window at the platform's default place, without a parent, drawn by
    /// Kivact.
    pub fn main(title: impl Into<String>, size: Size) -> WindowRequest {
        WindowRequest {
            id: WindowId::MAIN,
            title: title.into(),
            size,
            min_size: None,
            mode: WindowMode::Normal,
            placement: WindowPlacement::Auto,
            parent: None,
            content: WindowContent::Drawn,
        }
    }
}
