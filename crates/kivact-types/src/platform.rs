//! Role: Native platform requests, dialog options and classified failures.
use crate::Size;
use std::path::PathBuf;

/// Result of a platform request; cancellation is represented by an absent path.
#[derive(Clone, Debug, PartialEq)]
pub enum PlatformResult {
    Done,
    Text(String),
    Path(Option<PathBuf>),
    Paths(Option<Vec<PathBuf>>),
    Failed(Failure),
    Unsupported,
    /// The family names of a registered font file (`RegisterFont`).
    Fonts(Vec<String>),
}

/// Initial location, suggested name and filters for a file dialog.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FileDialog {
    /// Dialog title.
    pub title: String,
    /// Initial directory, when the platform supports it.
    pub directory: Option<PathBuf>,
    /// Suggested file name.
    pub file_name: Option<String>,
    /// (label, extensions without dot)
    pub filters: Vec<(String, Vec<String>)>,
}

/// A native platform action answered synchronously by its window.
#[derive(Clone, Debug, PartialEq)]
pub enum PlatformRequest {
    /// Persist application bytes for a future launch; unsupported platforms ignore it.
    SaveState {
        key: &'static str,
        bytes: Vec<u8>,
    },
    /// Request logical content dimensions.
    RequestWindowSize(Size),
    Fullscreen(bool),
    CopyText(String),
    ReadText,
    PickFile(FileDialog),
    PickFiles(FileDialog),
    SaveFile(FileDialog),
    /// Ask for a folder; the result is `Path` (`None` when cancelled).
    /// `file_name` and `filters` do not apply to a folder.
    PickFolder(FileDialog),
    /// Open a file or folder with the platform's default handler. The
    /// result is `Done` once the handler was started, `Failed` naming the
    /// path and the reason, or `Unsupported` on a platform without it.
    OpenPath(PathBuf),
    /// Show a file or folder selected in the platform's file manager;
    /// results as for `OpenPath`.
    RevealPath(PathBuf),
}

/// Stable classification of a boundary failure, independent of external libraries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FailureKind {
    /// Invalid application-supplied data; retrying the same input cannot succeed.
    Content,
    /// An external resource, such as an image or font, cannot be used.
    Resource,
    /// A platform adapter or rendering device failed.
    Backend,
    /// A window presentation surface failed and must be recreated.
    Surface,
    /// A required capability is unavailable in this environment.
    Unavailable,
}

/// Classified failure shared by platform services and rendering boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub kind: FailureKind,
    /// Human-readable context suitable for diagnostics.
    pub detail: String,
}

impl Failure {
    pub fn new(kind: FailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

impl std::error::Error for Failure {}
