//! Role: Passive models shared by creative editor domains.
use kivact::style::Color;
/// A visible time-editor row, identified independently of its display order.
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditorContext {
    // xtask-allow(state-fields): Independent optional identities in an immutable context-menu location snapshot.
    pub item: Option<u64>,
    // xtask-allow(state-fields): Independent optional identities in an immutable context-menu location snapshot.
    pub lane: Option<u64>,
    pub time: f64,
}

/// An application clip in its own time units.
#[derive(Clone, Debug, PartialEq)]
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
            color: crate::color::default_item_color(),
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemTime {
    /// The application item ID.
    pub id: u64,
    /// The proposed start time.
    pub time: f64,
    /// The proposed duration.
    pub duration: f64,
}

use kivact::interaction::EditSession;
use kivact::state::Callback;
use kivact::view::EditPhase;

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

pub(crate) fn phase_observer<R: 'static>(
    observer: Option<Callback<EditorEvent<R>>>,
) -> Option<Callback<EditPhase>> {
    observer.map(|observer| {
        kivact::state::callback(move |phase| {
            observer.call(match phase {
                EditPhase::Begin => EditorEvent::Begin,
                EditPhase::Commit => EditorEvent::Commit,
                EditPhase::Cancel => EditorEvent::Cancel,
            })
        })
    })
}

pub(crate) fn canvas_size(
    rect: kivact::state::Value<Option<kivact::geometry::Rect>>,
) -> kivact::state::Value<kivact::geometry::Size> {
    kivact::state::IntoValue::into_value(kivact::state::memo(move || {
        let r = rect.get().unwrap_or_default();
        kivact::geometry::Size::new(r.width, r.height)
    }))
}
impl EditorContext {
    pub(crate) fn at_time(time: f64) -> Self {
        Self {
            item: None,
            lane: None,
            time,
        }
    }
}
