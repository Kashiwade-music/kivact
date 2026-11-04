//! Role: Native platform requests and their results.
use crate::{Asset, Size};
use std::path::PathBuf;

/// Result of a platform request; cancellation is represented by an absent path.
#[derive(Clone, Debug, PartialEq)]
pub enum PlatformResult {
    Done,
    Text(String),
    Path(Option<PathBuf>),
    Paths(Option<Vec<PathBuf>>),
    Failed(String),
    Unsupported,
    /// The family names of a registered font file (`RegisterFont`).
    Fonts(Vec<String>),
}

/// Initial location, suggested name and filters for a file dialog.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileDialogRequest {
    /// Dialog title.
    pub title: String,
    /// Initial directory, when the platform supports it.
    pub directory: Option<PathBuf>,
    /// Suggested file name.
    pub file_name: Option<String>,
    /// (label, extensions without dot)
    pub filters: Vec<(String, Vec<String>)>,
}

/// A requested native platform action, completed through host events.
#[derive(Clone, Debug, PartialEq)]
pub enum PlatformRequest {
    /// Request logical content dimensions.
    RequestWindowSize(Size),
    Fullscreen(bool),
    /// Override the window's reduced-motion preference, or follow the system.
    ReducedMotion(Option<bool>),
    CopyText(String),
    ReadText,
    PickFile(FileDialogRequest),
    PickFiles(FileDialogRequest),
    SaveFile(FileDialogRequest),
    /// Ask for a folder; the result is `Path` (`None` when cancelled).
    /// `file_name` and `filters` do not apply to a folder.
    PickFolder(FileDialogRequest),
    /// Open a file or folder with the platform's default handler. The
    /// result is `Done` once the handler was started, `Failed` naming the
    /// path and the reason, or `Unsupported` on a platform without it.
    OpenPath(PathBuf),
    /// Show a file or folder selected in the platform's file manager;
    /// results as for `OpenPath`.
    RevealPath(PathBuf),
    /// Register the fonts of a TTF, OTF or TTC file with the host's text
    /// system (always available). The result is
    /// `PlatformResult::Fonts` with the family names in the file, or
    /// `Failed` with a message naming the source and the reason. The
    /// engine handles it when it is written: the next layout already
    /// measures with the font, and text measured before is measured and
    /// drawn again.
    RegisterFont(Asset),
}
