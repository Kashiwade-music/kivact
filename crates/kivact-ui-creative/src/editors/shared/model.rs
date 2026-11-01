//! Role: Passive models shared by creative editor domains.
use kivact::style::{Color, rgb};
/// A visible time-editor row, identified independently of its display order.
/// Built with [`Lane::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Lane {
    pub id: u64,
    pub label: String,
}
impl Lane {
    pub fn new(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

/// A snapshot of the domain location that requested an editor context menu.
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct EditorContext {
    pub item: Option<u64>,
    pub lane: Option<u64>,
    pub time: f64,
}

/// An application clip in its own time units.
/// Built with [`Clip::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Clip {
    pub id: u64,
    pub lane: u64,
    pub start: f64,
    pub duration: f64,
    pub label: String,
    pub color: Color,
    pub muted: bool,
}
impl Clip {
    pub fn new(id: u64, lane: u64, start: f64, duration: f64, label: impl Into<String>) -> Self {
        Self {
            id,
            lane,
            start,
            duration,
            label: label.into(),
            color: rgb(0x8b7cff),
            muted: false,
        }
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

pub(crate) fn drawable(clip: &Clip) -> bool {
    clip.start.is_finite() && clip.duration.is_finite() && clip.duration > 0.0
}

/// The time and length an editor proposes for one item.
/// Built with [`ItemTime::new`].
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ItemTime {
    /// The application item ID.
    pub id: u64,
    /// The proposed start time.
    pub time: f64,
    /// The proposed duration.
    pub duration: f64,
}

impl ItemTime {
    /// The start `time` and `duration` of item `id`.
    pub const fn new(id: u64, time: f64, duration: f64) -> Self {
        ItemTime { id, time, duration }
    }
}

use kivact::Callback;
use kivact::interaction::EditSession;
use kivact::ui::EditPhase;

/// `session` reporting its phases to the kit's stored observer, if any.
pub(crate) fn observed<T: Clone + 'static>(
    session: EditSession<T>,
    observer: Option<Callback<EditPhase>>,
) -> EditSession<T> {
    match observer {
        Some(observer) => session.on_edit(move |phase| observer.call(phase)),
        None => session,
    }
}

/// One report from an editor surface. The editor never writes the application's items.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum EditorEvent<R> {
    /// A gesture starts. Requests until `Commit` or `Cancel` belong to one Undo step.
    Begin,
    /// An absolute proposal from the items at `Begin`, or one whole step outside a gesture.
    Request(R),
    /// The gesture ended and the application keeps the applied requests.
    Commit,
    /// Bound values are restored; the application restores its items from `Begin`.
    Cancel,
}
